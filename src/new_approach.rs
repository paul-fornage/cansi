use crate::CategorisedSlice;



const CSI: &str = "\x1b[";
const SEPARATOR: char = ';';
#[inline(always)]
fn terminated_byte(byte: u8) -> bool {
    (0x40..=0x7e).contains(&byte)
}

pub fn parse(text: &str) -> Vec<CategorisedSlice> {
    let mut v: Vec<CategorisedSlice> = Vec::new();
    // SAFE TO USE .bytes():
    // UTF-8 guarantees ASCII bytes (0x00-0x7F) never appear as parts of multi-byte characters.
    // Since `\x1b` (0x1B) and `[` (0x5B) are ASCII, their byte values uniquely identify them.
    let bytes = text.bytes();
    todo!()
}

fn my_function(my_str: &str) -> Option<Vec<u8>> { None }

#[cfg(test)]
mod tests {
    use super::*;

    // --- 1. THE "IMPLICIT ZERO" TESTS ---
    #[test]
    fn test_implicit_zeros() {
        // Empty sequence implies 0
        assert_eq!(my_function("\x1b[m"), Some(vec![0]));

        // Semicolon with no numbers implies two 0s
        assert_eq!(my_function("\x1b[;m"), Some(vec![0, 0]));

        // Leading implicit 0
        assert_eq!(my_function("\x1b[;45m"), Some(vec![0, 45]));

        // Trailing implicit 0
        assert_eq!(my_function("\x1b[31;m"), Some(vec![31, 0]));

        // Interleaved implicit 0
        assert_eq!(my_function("\x1b[31;;45m"), Some(vec![31, 0, 45]));

        // Multiple adjacent implicit zeros
        assert_eq!(my_function("\x1b[;;;m"), Some(vec![0, 0, 0, 0]));
    }

    // --- 2. NUMBER PARSING EDGE CASES ---
    #[test]
    fn test_number_formatting() {
        // Leading zeros should be parsed normally
        assert_eq!(my_function("\x1b[031;0045m"), Some(vec![31, 45]));

        // Multiple leading zeros for 0
        assert_eq!(my_function("\x1b[000;m"), Some(vec![0, 0]));
    }

    // --- 3. ADVERSARIAL & MALFORMED SEQUENCES (EXPECTING NONE) ---
    #[test]
    fn test_malformed_expecting_none() {
        // --- Garbage data in parameters ---
        assert_eq!(my_function("\x1b[31;x;45m"), None); // Letters instead of numbers
        assert_eq!(my_function("\x1b[31;-45m"), None);  // Negative signs are not valid in ECMA-48 parameters
        assert_eq!(my_function("\x1b[31.5m"), None);    // Decimals are invalid
        assert_eq!(my_function("\x1b[+31m"), None);     // Plus signs are invalid

        // --- Space / Intermediate Byte Injection ---
        // According to ECMA-48, spaces (0x20) are "Intermediate Bytes" and change the meaning
        // of the sequence. If a space is present, it is NO LONGER an SGR sequence.
        assert_eq!(my_function("\x1b[ 31m"), None);
        assert_eq!(my_function("\x1b[31 m"), None);
        assert_eq!(my_function("\x1b[31; 45m"), None);

        // Other intermediate bytes like '!' or '?' change the control function entirely
        assert_eq!(my_function("\x1b[?31m"), None); // This is a DEC Private Mode sequence, not SGR
        assert_eq!(my_function("\x1b[!31m"), None);

        // --- Truncated Sequences ---
        assert_eq!(my_function("\x1b["), None);
        assert_eq!(my_function("\x1b[31"), None);
        assert_eq!(my_function("\x1b[31;"), None);

        // --- Nested or Double Escapes ---
        assert_eq!(my_function("\x1b[\x1b[31m"), None); // Escape inside escape
        assert_eq!(my_function("\x1b[[31m"), None);     // Double bracket

        // --- The "Other Final Byte" trick ---
        // 'M' is not 'm'. 'M' is Delete Line (DL), not SGR.
        assert_eq!(my_function("\x1b[31M"), None);
    }

    // --- 4. OVERFLOW AND DOS ATTACKS ---
    #[test]
    fn test_overflow_attacks() {

        assert_eq!(my_function("\x1b[999999999999999999999999999m"), None);
        assert_eq!(my_function("\x1b[31;999999999999999999999999999;45m"), None);

        let mut huge_seq = String::from("\x1b[");
        for _ in 0..10_000 {
            huge_seq.push_str("1;");
        }
        huge_seq.pop().unwrap();
        huge_seq.push_str("m");

        let result = my_function(&huge_seq);

        assert!(result.unwrap().iter().all(|&x| x == 1));
    }

    // --- 5. SUBTLE SPEC VIOLATIONS ---
    #[test]
    fn test_colon_separators() {
        // not supported, not planned. But should not crash
        assert_eq!(my_function("\x1b[38:2::255:0:0m"), None);
    }
}