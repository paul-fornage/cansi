use super::*;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

/// Parses the text and returns each formatted slice in order.
/// The ANSI escape codes are not included in the text slices.
///
/// Each different text slice is returned in order such that the text without the escape characters can be reconstructed.
/// There is a helper function (`construct_text_no_codes`) on `CategorisedSlices` for this.
pub fn categorise_text(text: &str) -> CategorisedSlices {
    crate::new_approach::parse(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use colored::Colorize;

    #[test]
    fn cat_and_matches_len() {
        let txt = "hello";
        let matches = parse(&txt);
        let cat = categorise_text(&txt);
        assert!(matches.len() + 1 >= cat.len());

        let txt = "hello".bright_green();
        let matches = parse(&txt);
        let cat = categorise_text(&txt);
        assert!(matches.len() + 1 >= cat.len());

        let txt = format!("{}{}{}", "hello".bright_green(), "world".red(), "whatever");
        let matches = parse(&txt);
        let cat = categorise_text(&txt);
        assert!(matches.len() + 1 >= cat.len());
    }

    #[test]
    fn malformed_escapes() {
        let x = categorise_text("oops\x1b[\n");
        assert_eq!(
            x,
            vec![CategorisedSlice {
                text: "oops\x1b[\n",
                start: 0,
                end: 7,
                fg: None,
                bg: None,
                blink: None,
                underline: None,
                hidden: None,
                intensity: None,
                italic: None,
                reversed: None,
                strikethrough: None
            }]
        );
    }
}
