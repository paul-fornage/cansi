pub mod legacy;

use super::v3::{categorise_text, construct_text_no_codes, line_iter, CategorisedSlice};
use super::{parse, split_on_new_line, Color, Intensity};
use colored::Colorize;
use std::io::Write;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

const DEFAULT_STYLE: CategorisedSlice = CategorisedSlice {
    text: "",
    start: 0,
    end: 0,
    fg: None,
    bg: None,
    intensity: None,
    italic: None,
    underline: None,
    blink: None,
    reversed: None,
    hidden: None,
    strikethrough: None,
};

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const UNDERLINE: &str = "\x1b[4m";

fn clone_style<'text>(
    slice: CategorisedSlice<'text>,
    text: &'text str,
    start: usize,
    end: usize,
) -> CategorisedSlice<'text> {
    CategorisedSlice {
        text,
        start,
        end,
        ..slice
    }
}

#[test]
fn cover_other_parameters() {

    // no escape sequences
    let text = "test";
    assert_eq!(categorise_text(&text[..])[0], CategorisedSlice{
        text,
        start: 0,
        end: 4,
        ..DEFAULT_STYLE
    });

    // empty sequences
    let text = "\x1b[;mtest";
    assert_eq!(categorise_text(&text[..])[0], CategorisedSlice{
        text: "test",
        start: 4,
        end: 8,
        ..DEFAULT_STYLE
    });

    // empty text - doesn't add it
    let text = "\x1b[;mtest\x1b[;m\x1b[;m";
    assert_eq!(categorise_text(&text[..]).len(), 1);
    assert_eq!(categorise_text(&text[..])[0], CategorisedSlice{
        text: "test",
        start: 4,
        end: 8,
        ..DEFAULT_STYLE
    });


    // 22
    let text = "\x1b[1;22mtest";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 7,
            end: 11,
            intensity: Some(Intensity::Normal),
            ..DEFAULT_STYLE
        }
    );

    // 23
    let text = "\x1b[3;23mtest";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 7,
            end: 11,
            italic: Some(false),
            ..DEFAULT_STYLE
        }
    );

    // 24
    let text = "\x1b[4;24mtest";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 7,
            end: 11,
            underline: Some(false),
            ..DEFAULT_STYLE
        }
    );

    // 25
    let text = "\x1b[5;25mtest";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 7,
            end: 11,
            blink: Some(false),
            ..DEFAULT_STYLE
        }
    );

    // 27
    let text = "\x1b[7;27mtest";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 7,
            end: 11,
            reversed: Some(false),
            ..DEFAULT_STYLE
        }
    );

    // 28
    let text = "\x1b[8;28mtest";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 7,
            end: 11,
            hidden: Some(false),
            ..DEFAULT_STYLE
        }
    );

    // 29
    let text = "\x1b[9;29mtest";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 7,
            end: 11,
            strikethrough: Some(false),
            ..DEFAULT_STYLE
        }
    );
}

#[test]
fn split_on_new_line_tests() {
    fn fn_as_str(s: &str) -> (&str, Option<&str>) {
        let (first, remainder) = split_on_new_line(s);
        (&s[..first], remainder.map(|i| &s[i..]))
    }

    // no remainder
    let (first, remainder) = fn_as_str("Hello worlds");
    assert_eq!(first, "Hello worlds");
    assert_eq!(remainder, None);

    let (first, remainder) = fn_as_str("Hello worlds\n");
    assert_eq!(first, "Hello worlds");
    assert_eq!(remainder, Some(""));

    let (first, remainder) = fn_as_str("Hello worlds\r\n");
    assert_eq!(first, "Hello worlds");
    assert_eq!(remainder, Some(""));

    // some remainder
    let (first, remainder) = fn_as_str("Hello worlds\none two three");
    assert_eq!(first, "Hello worlds");
    assert_eq!(remainder, Some("one two three"));

    let (first, remainder) = fn_as_str("Hello worlds\r\none two three");
    assert_eq!(first, "Hello worlds");
    assert_eq!(remainder, Some("one two three"));

    let (first, remainder) = fn_as_str("Hello worlds\r\none\ntwo\nthree\n");
    assert_eq!(first, "Hello worlds");
    assert_eq!(remainder, Some("one\ntwo\nthree\n"));

    // no first
    let (first, remainder) = fn_as_str("\r\nHello worlds\none two three");
    assert_eq!(first, "");
    assert_eq!(remainder, Some("Hello worlds\none two three"));

    let (first, remainder) = fn_as_str("\nHello worlds\r\none two three");
    assert_eq!(first, "");
    assert_eq!(remainder, Some("Hello worlds\r\none two three"));

    let (first, remainder) = fn_as_str("\r\n");
    assert_eq!(first, "");
    assert_eq!(remainder, Some(""));
}

#[test]
fn clone_style_test() {
    use colored::*;
    colored::control::set_override(true);
    let s = "hello".green();
    let c = categorise_text(&s);
    let d = clone_style(c[0], "why", 0, 0);

    assert_eq!(d.text, "why");

    let e = clone_style(d, "hello", 0, 5);

    assert_eq!(c[0], e);
}

#[test]
fn line_iter_test() {
    colored::control::set_override(true);

    let green = CategorisedSlice{
        text: "",
        start: 0,
        end: 0,
        fg: Some(Color::Green),
        ..DEFAULT_STYLE
    };
    let red = CategorisedSlice{
        text: "",
        start: 0,
        end: 0,
        fg: Some(Color::Red),
        ..DEFAULT_STYLE
    };

    let cat = categorise_text("hello, world");
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            text: "hello, world",
            start: 0,
            end: 12,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(iter.next(), None);

    let cat = categorise_text("hello, world\nhow are you");
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            text: "hello, world",
            start: 0,
            end: 12,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            text: "how are you",
            start: 13,
            end: 24,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(iter.next(), None);

    let s = format!("{}{}\nhow are you", "hello, ".green(), "world".red());
    let cat = categorise_text(&s);
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![
            clone_style(green, "hello, ", 5, 12),
            clone_style(red, "world", 21, 26)
        ])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            text: "how are you",
            start: 31,
            end: 42,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(&s[5..12], "hello, ");
    assert_eq!(&s[21..26], "world");
    assert_eq!(&s[31..42], "how are you");
    assert_eq!(iter.next(), None);

    let s = format!(
        "{}{}\nhow are you\r\ntoday",
        "hello, ".green(),
        "world".red()
    );
    let cat = categorise_text(&s);
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![
            clone_style(green, "hello, ", 5, 12),
            clone_style(red, "world", 21, 26)
        ])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            text: "how are you",
            start: 31,
            end: 42,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            text: "today",
            start: 44,
            end: 49,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(iter.next(), None);
    assert_eq!(&s[5..12], "hello, ");
    assert_eq!(&s[21..26], "world");
    assert_eq!(&s[31..42], "how are you");
    assert_eq!(&s[44..49], "today");

    let cat = categorise_text("\n\n\n\n");
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 0,
            end: 0,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 1,
            end: 1,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 2,
            end: 2,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 3,
            end: 3,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 4,
            end: 4,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(iter.next(), None);

    let cat = categorise_text("\r\n\r\n\r\n\r\n");
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 0,
            end: 0,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 2,
            end: 2,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 4,
            end: 4,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 6,
            end: 6,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            start: 8,
            end: 8,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(iter.next(), None);
}

#[test]
fn line_iter_newline_starts_with_esc() {
    colored::control::set_override(true);

    let green = CategorisedSlice{
        text: "",
        start: 0,
        end: 0,
        fg: Some(Color::Green),
        ..DEFAULT_STYLE
    };

    let s = format!("hello\n{}", "world".green());
    let cat = categorise_text(&s);
    let mut iter = line_iter(&cat);

    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice{
            text: "hello",
            start: 0,
            end: 5,
            ..DEFAULT_STYLE
        }])
    );
    assert_eq!(iter.next(), Some(vec![clone_style(green, "world", 11, 16)]));
    assert_eq!(&s[11..16], "world");
}

#[test]
fn line_iter_bugs() {
    let bug_str = "\u{1b}[36mpapyrus\u{1b}[0m=> 5+6\n\u{1b}[36mpapyrus\u{1b}[0m \u{1b}[92m[out0]\u{1b}[0m: 11                                            \n\u{1b}[36mpapyrus\u{1b}[0m=>
                              \n                                                                                \n                                                                                \n
                     \n                                                                                \n                                                                                \n
            \n                                                                                \n                                                                                \n
   \n                                                                                \n                                                                                \n                                                                                \n
                                                                            \n                                                                                \n                                                                                \n
                                                                   \n                                                                                \n                                                                                \n
                                                          \n                                                                                \n                                                                                \n";

    let cat = categorise_text(bug_str);
    let mut iter = line_iter(&cat);

    let cyan = CategorisedSlice{
        fg: Some(Color::Cyan),
        ..DEFAULT_STYLE
    };

    assert_eq!(
        iter.next(),
        Some(vec![
            clone_style(cyan, "papyrus", 5, 12),
            CategorisedSlice{
                text: "=> 5+6",
                start: 16,
                end: 22,
                ..DEFAULT_STYLE
            }
        ])
    );

    assert_eq!(&bug_str[5..12], "papyrus");
    assert_eq!(&bug_str[16..22], "=> 5+6");
}

#[test]
fn byte_bug() {
    let s = "ﾮ";
    let matches = parse(s);
    assert_eq!(matches, vec![]);

    let x = categorise_text(&s);
    let c = construct_text_no_codes(&x);
    assert_eq!(s, c);
}

#[test]
fn colouring_with_emojis() {
    let t = "👋, \x1b[31;4m🌍\x1b[0m!";
    let c = categorise_text(t);
    assert_eq!(construct_text_no_codes(&c), "👋, 🌍!");

    let mut c = c.into_iter();

    assert_eq!(
        c.next(),
        Some(CategorisedSlice {
            text: "👋, ",
            start: 0,
            end: 6,
            ..DEFAULT_STYLE
        })
    );

    assert_eq!(
        c.next(),
        Some(CategorisedSlice {
            text: "🌍",
            start: 13,
            end: 17,
            fg: Some(Color::Red),
            underline: Some(true),
            ..DEFAULT_STYLE
        })
    );

    assert_eq!(
        c.next(),
        Some(CategorisedSlice {
            text: "!",
            start: 21,
            end: 22,
            ..DEFAULT_STYLE
        })
    );

    assert_eq!(c.next(), None);
}



#[test]
fn test_echo_command() {
    let command = "mycommand";
    let input = format!("{CYAN}{BOLD}[echo]{RESET} {UNDERLINE}{GREEN}<<<{ITALIC}{command}{UNDERLINE}>>>{RESET}");
    let slices = categorise_text(&input);

    assert_eq!(slices.len(), 5);

    assert_eq!(
        slices[0],
        CategorisedSlice {
            text: "[echo]",
            start: 9,
            end: 15,
            fg: Some(Color::Cyan),
            intensity: Some(Intensity::Bold),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[1],
        CategorisedSlice {
            text: " ",
            start: 19,
            end: 20,
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[2],
        CategorisedSlice {
            text: "<<<",
            start: 29,
            end: 32,
            fg: Some(Color::Green),
            underline: Some(true),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[3],
        CategorisedSlice {
            text: command,
            start: 36,
            end: 45,
            fg: Some(Color::Green),
            italic: Some(true),
            underline: Some(true),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[4],
        CategorisedSlice {
            text: ">>>",
            start: 49,
            end: 52,
            fg: Some(Color::Green),
            italic: Some(true),
            underline: Some(true),
            ..DEFAULT_STYLE
        }
    );
}

#[test]
fn test_heartbeat() {
    let ms = "9876";
    let input = format!("{GREEN}{BOLD}[heartbeat]{RESET}{DIM} alive t_ms={RESET}{ms}");
    let slices = categorise_text(&input);

    assert_eq!(slices.len(), 3);

    assert_eq!(
        slices[0],
        CategorisedSlice {
            text: "[heartbeat]",
            start: 9,
            end: 20,
            fg: Some(Color::Green),
            intensity: Some(Intensity::Bold),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[1],
        CategorisedSlice {
            text: " alive t_ms=",
            start: 24,
            end: 35,
            intensity: Some(Intensity::Faint),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[2],
        CategorisedSlice {
            text: ms,
            start: 39,
            end: 43,
            ..DEFAULT_STYLE
        }
    );
}

#[test]
fn test_boot_message() {
    let input = format!("{MAGENTA}[boot]{RESET} usb echo firmware ready");
    let slices = categorise_text(&input);

    assert_eq!(slices.len(), 2);

    assert_eq!(
        slices[0],
        CategorisedSlice {
            text: "[boot]",
            start: 5,
            end: 11,
            fg: Some(Color::Magenta),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[1],
        CategorisedSlice {
            text: " usb echo firmware ready",
            start: 15,
            end: 39,
            ..DEFAULT_STYLE
        }
    );
}

#[test]
fn test_format_styles() {
    let input = format!("{BOLD}[format]{RESET} {BOLD}bold {DIM}dim {ITALIC}italic {UNDERLINE}underline {RESET}");
    let slices = categorise_text(&input);

    assert_eq!(slices.len(), 6);

    assert_eq!(
        slices[0],
        CategorisedSlice {
            text: "[format]",
            start: 4,
            end: 12,
            intensity: Some(Intensity::Bold),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[1],
        CategorisedSlice {
            text: " ",
            start: 16,
            end: 17,
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[2],
        CategorisedSlice {
            text: "bold ",
            start: 21,
            end: 26,
            intensity: Some(Intensity::Bold),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[3],
        CategorisedSlice {
            text: "dim ",
            start: 30,
            end: 34,
            intensity: Some(Intensity::Faint),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[4],
        CategorisedSlice {
            text: "italic ",
            start: 38,
            end: 45,
            intensity: Some(Intensity::Faint),
            italic: Some(true),
            ..DEFAULT_STYLE
        }
    );
    assert_eq!(
        slices[5],
        CategorisedSlice {
            text: "underline ",
            start: 49,
            end: 59,
            intensity: Some(Intensity::Faint),
            italic: Some(true),
            underline: Some(true),
            ..DEFAULT_STYLE
        }
    );
}



#[test]
fn test_readme_code() {
    let v = &mut Vec::new();
    write!(
        v,
        "Hello, {}{}{}{}{}{}",
        "w".white().on_red(),
        "o".cyan().on_green(),
        "r".magenta().on_yellow(),
        "l".blue().on_white(),
        "d".yellow().on_bright_cyan(),
        "!".bright_red().on_bright_yellow(),
    )
        .unwrap();

    let text = String::from_utf8_lossy(&v);
    let result = categorise_text(&text); // cansi function

    assert_eq!(result.len(), 7); // there should be seven differently styled components

    assert_eq!("Hello, world!", &construct_text_no_codes(&result));

    // 'Hello, ' is just defaults
    assert_eq!(
        result[0],
        CategorisedSlice {
            text: "Hello, ",
            start: 0,
            end: 7,
            ..DEFAULT_STYLE
        }
    );

    assert_eq!(
        result[1],
        CategorisedSlice {
            text: "w",
            start: 15,
            end: 16,
            fg: Some(Color::White),
            bg: Some(Color::Red),
            ..DEFAULT_STYLE
        }
    );

    assert_eq!(
        result[2],
        CategorisedSlice {
            text: "o",
            start: 28,
            end: 29,
            fg: Some(Color::Cyan),
            bg: Some(Color::Green),
            ..DEFAULT_STYLE
        }
    );

    assert_eq!(
        result[3],
        CategorisedSlice {
            text: "r",
            start: 41,
            end: 42,
            fg: Some(Color::Magenta),
            bg: Some(Color::Yellow),
            ..DEFAULT_STYLE
        }
    );

    assert_eq!(
        result[4],
        CategorisedSlice {
            text: "l",
            start: 54,
            end: 55,
            fg: Some(Color::Blue),
            bg: Some(Color::White),
            ..DEFAULT_STYLE
        }
    );

    assert_eq!(
        result[5],
        CategorisedSlice {
            text: "d",
            start: 68,
            end: 69,
            fg: Some(Color::Yellow),
            bg: Some(Color::BrightCyan),
            ..DEFAULT_STYLE
        }
    );

    assert_eq!(
        result[6],
        CategorisedSlice {
            text: "!",
            start: 82,
            end: 83,
            fg: Some(Color::BrightRed),
            bg: Some(Color::BrightYellow),
            ..DEFAULT_STYLE
        }
    );
}

#[test]
fn test_colored_function() {
    colored::control::set_override(true);

    let test_string: &str = "test";
    let v = &mut Vec::new();

    macro_rules! assert_colored {
        ($expr:expr, $expected:expr, $name:literal) => {{
            v.clear();
            write!(v, "{}", $expr).unwrap();
            assert_eq!(categorise_text(&String::from_utf8_lossy(v))[0], $expected, $name);
        }};
    }

    assert_colored!("test".black(), CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::Black), ..DEFAULT_STYLE }, "black()");
    assert_colored!("test".red(), CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::Red), ..DEFAULT_STYLE }, "red()");
    assert_colored!("test".green(), CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::Green), ..DEFAULT_STYLE }, "green()");
    assert_colored!("test".yellow(), CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::Yellow), ..DEFAULT_STYLE }, "yellow()");
    assert_colored!("test".blue(), CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::Blue), ..DEFAULT_STYLE }, "blue()");
    assert_colored!("test".magenta(), CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::Magenta), ..DEFAULT_STYLE }, "magenta()");
    assert_colored!("test".purple(), CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::Magenta), ..DEFAULT_STYLE }, "purple()");
    assert_colored!("test".cyan(), CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::Cyan), ..DEFAULT_STYLE }, "cyan()");
    assert_colored!("test".white(), CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::White), ..DEFAULT_STYLE }, "white()");
    assert_colored!(
        "test".bright_black(),
        CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::BrightBlack), ..DEFAULT_STYLE },
        "bright_black()"
    );
    assert_colored!(
        "test".bright_red(),
        CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::BrightRed), ..DEFAULT_STYLE },
        "bright_red()"
    );
    assert_colored!(
        "test".bright_green(),
        CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::BrightGreen), ..DEFAULT_STYLE },
        "bright_green()"
    );
    assert_colored!(
        "test".bright_yellow(),
        CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::BrightYellow), ..DEFAULT_STYLE },
        "bright_yellow()"
    );
    assert_colored!(
        "test".bright_blue(),
        CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::BrightBlue), ..DEFAULT_STYLE },
        "bright_blue()"
    );
    assert_colored!(
        "test".bright_magenta(),
        CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::BrightMagenta), ..DEFAULT_STYLE },
        "bright_magenta()"
    );
    assert_colored!(
        "test".bright_purple(),
        CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::BrightMagenta), ..DEFAULT_STYLE },
        "bright_purple()"
    );
    assert_colored!(
        "test".bright_cyan(),
        CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::BrightCyan), ..DEFAULT_STYLE },
        "bright_cyan()"
    );
    assert_colored!(
        "test".bright_white(),
        CategorisedSlice { text: test_string, start: 5, end: 9, fg: Some(Color::BrightWhite), ..DEFAULT_STYLE },
        "bright_white()"
    );

    assert_colored!("test".on_black(), CategorisedSlice { text: test_string, start: 5, end: 9, bg: Some(Color::Black), ..DEFAULT_STYLE }, "on_black()");
    assert_colored!("test".on_red(), CategorisedSlice { text: test_string, start: 5, end: 9, bg: Some(Color::Red), ..DEFAULT_STYLE }, "on_red()");
    assert_colored!("test".on_green(), CategorisedSlice { text: test_string, start: 5, end: 9, bg: Some(Color::Green), ..DEFAULT_STYLE }, "on_green()");
    assert_colored!(
        "test".on_yellow(),
        CategorisedSlice { text: test_string, start: 5, end: 9, bg: Some(Color::Yellow), ..DEFAULT_STYLE },
        "on_yellow()"
    );
    assert_colored!("test".on_blue(), CategorisedSlice { text: test_string, start: 5, end: 9, bg: Some(Color::Blue), ..DEFAULT_STYLE }, "on_blue()");
    assert_colored!(
        "test".on_magenta(),
        CategorisedSlice { text: test_string, start: 5, end: 9, bg: Some(Color::Magenta), ..DEFAULT_STYLE },
        "on_magenta()"
    );
    assert_colored!(
        "test".on_purple(),
        CategorisedSlice { text: test_string, start: 5, end: 9, bg: Some(Color::Magenta), ..DEFAULT_STYLE },
        "on_purple()"
    );
    assert_colored!("test".on_cyan(), CategorisedSlice { text: test_string, start: 5, end: 9, bg: Some(Color::Cyan), ..DEFAULT_STYLE }, "on_cyan()");
    assert_colored!("test".on_white(), CategorisedSlice { text: test_string, start: 5, end: 9, bg: Some(Color::White), ..DEFAULT_STYLE }, "on_white()");
    assert_colored!(
        "test".on_bright_black(),
        CategorisedSlice { text: test_string, start: 6, end: 10, bg: Some(Color::BrightBlack), ..DEFAULT_STYLE },
        "on_bright_black()"
    );
    assert_colored!(
        "test".on_bright_red(),
        CategorisedSlice { text: test_string, start: 6, end: 10, bg: Some(Color::BrightRed), ..DEFAULT_STYLE },
        "on_bright_red()"
    );
    assert_colored!(
        "test".on_bright_green(),
        CategorisedSlice { text: test_string, start: 6, end: 10, bg: Some(Color::BrightGreen), ..DEFAULT_STYLE },
        "on_bright_green()"
    );
    assert_colored!(
        "test".on_bright_yellow(),
        CategorisedSlice { text: test_string, start: 6, end: 10, bg: Some(Color::BrightYellow), ..DEFAULT_STYLE },
        "on_bright_yellow()"
    );
    assert_colored!(
        "test".on_bright_blue(),
        CategorisedSlice { text: test_string, start: 6, end: 10, bg: Some(Color::BrightBlue), ..DEFAULT_STYLE },
        "on_bright_blue()"
    );
    assert_colored!(
        "test".on_bright_magenta(),
        CategorisedSlice { text: test_string, start: 6, end: 10, bg: Some(Color::BrightMagenta), ..DEFAULT_STYLE },
        "on_bright_magenta()"
    );
    assert_colored!(
        "test".on_bright_purple(),
        CategorisedSlice { text: test_string, start: 6, end: 10, bg: Some(Color::BrightMagenta), ..DEFAULT_STYLE },
        "on_bright_purple()"
    );
    assert_colored!(
        "test".on_bright_cyan(),
        CategorisedSlice { text: test_string, start: 6, end: 10, bg: Some(Color::BrightCyan), ..DEFAULT_STYLE },
        "on_bright_cyan()"
    );
    assert_colored!(
        "test".on_bright_white(),
        CategorisedSlice { text: test_string, start: 6, end: 10, bg: Some(Color::BrightWhite), ..DEFAULT_STYLE },
        "on_bright_white()"
    );

    assert_colored!("test".clear(), CategorisedSlice { text: test_string, start: 0, end: 4, ..DEFAULT_STYLE }, "clear()");
    assert_colored!("test".normal(), CategorisedSlice { text: test_string, start: 0, end: 4, ..DEFAULT_STYLE }, "normal()");
    assert_colored!(
        "test".bold(),
        CategorisedSlice { text: test_string, start: 4, end: 8, intensity: Some(Intensity::Bold), ..DEFAULT_STYLE },
        "bold()"
    );
    assert_colored!(
        "test".dimmed(),
        CategorisedSlice { text: test_string, start: 4, end: 8, intensity: Some(Intensity::Faint), ..DEFAULT_STYLE },
        "dimmed()"
    );
    assert_colored!(
        "test".italic(),
        CategorisedSlice { text: test_string, start: 4, end: 8, italic: Some(true), ..DEFAULT_STYLE },
        "italic()"
    );
    assert_colored!(
        "test".underline(),
        CategorisedSlice { text: test_string, start: 4, end: 8, underline: Some(true), ..DEFAULT_STYLE },
        "underline()"
    );
    assert_colored!(
        "test".blink(),
        CategorisedSlice { text: test_string, start: 4, end: 8, blink: Some(true), ..DEFAULT_STYLE },
        "blink()"
    );
    assert_colored!(
        "test".reverse(),
        CategorisedSlice { text: test_string, start: 4, end: 8, reversed: Some(true), ..DEFAULT_STYLE },
        "reverse()"
    );
    assert_colored!(
        "test".reversed(),
        CategorisedSlice { text: test_string, start: 4, end: 8, reversed: Some(true), ..DEFAULT_STYLE },
        "reversed()"
    );
    assert_colored!(
        "test".hidden(),
        CategorisedSlice { text: test_string, start: 4, end: 8, hidden: Some(true), ..DEFAULT_STYLE },
        "hidden()"
    );
    assert_colored!(
        "test".strikethrough(),
        CategorisedSlice { text: test_string, start: 4, end: 8, strikethrough: Some(true), ..DEFAULT_STYLE },
        "strikethrough()"
    );
}