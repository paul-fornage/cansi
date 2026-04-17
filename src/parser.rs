use crate::{CategorisedSlice, Color, Intensity, SGR};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

/// Errors that can occur when parsing a CSI SGR sequence.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// The sequence was cut off before a final byte was found.
    #[error("CSI sequence truncated before final byte")]
    Truncated,
    /// A byte was encountered that is not valid in a CSI parameter string.
    #[error("invalid byte 0x{0:02x} in CSI parameter string")]
    InvalidByte(u8),
    /// A numeric parameter overflowed u8.
    #[error("SGR parameter overflowed u8")]
    Overflow,
    /// The final byte was not 'm', so this is a valid CSI but not SGR.
    #[error("valid CSI but not SGR (final byte 0x{0:02x})")]
    NotSgr(u8),
    /// The sequence uses a private/experimental prefix (e.g. `ESC[?`).
    #[error("private/experimental CSI sequence")]
    PrivateSequence,
}

/// Apply a single SGR parameter value to the SGR state (GRCM cumulative).
/// Unknown parameter values are ignored per ECMA-48.
#[inline]
fn apply_sgr_param(sgr: &mut SGR, param: u8) {
    match param {
        0 => *sgr = SGR::default(),
        1 => sgr.intensity = Some(Intensity::Bold),
        2 => sgr.intensity = Some(Intensity::Faint),
        3 => sgr.italic = Some(true),
        4 => sgr.underline = Some(true),
        5 => sgr.blink = Some(true),
        7 => sgr.reversed = Some(true),
        8 => sgr.hidden = Some(true),
        9 => sgr.strikethrough = Some(true),
        22 => sgr.intensity = Some(Intensity::Normal),
        23 => sgr.italic = Some(false),
        24 => sgr.underline = Some(false),
        25 => sgr.blink = Some(false),
        27 => sgr.reversed = Some(false),
        28 => sgr.hidden = Some(false),
        29 => sgr.strikethrough = Some(false),
        30 => sgr.fg = Some(Color::Black),
        31 => sgr.fg = Some(Color::Red),
        32 => sgr.fg = Some(Color::Green),
        33 => sgr.fg = Some(Color::Yellow),
        34 => sgr.fg = Some(Color::Blue),
        35 => sgr.fg = Some(Color::Magenta),
        36 => sgr.fg = Some(Color::Cyan),
        37 => sgr.fg = Some(Color::White),
        40 => sgr.bg = Some(Color::Black),
        41 => sgr.bg = Some(Color::Red),
        42 => sgr.bg = Some(Color::Green),
        43 => sgr.bg = Some(Color::Yellow),
        44 => sgr.bg = Some(Color::Blue),
        45 => sgr.bg = Some(Color::Magenta),
        46 => sgr.bg = Some(Color::Cyan),
        47 => sgr.bg = Some(Color::White),
        90 => sgr.fg = Some(Color::BrightBlack),
        91 => sgr.fg = Some(Color::BrightRed),
        92 => sgr.fg = Some(Color::BrightGreen),
        93 => sgr.fg = Some(Color::BrightYellow),
        94 => sgr.fg = Some(Color::BrightBlue),
        95 => sgr.fg = Some(Color::BrightMagenta),
        96 => sgr.fg = Some(Color::BrightCyan),
        97 => sgr.fg = Some(Color::BrightWhite),
        100 => sgr.bg = Some(Color::BrightBlack),
        101 => sgr.bg = Some(Color::BrightRed),
        102 => sgr.bg = Some(Color::BrightGreen),
        103 => sgr.bg = Some(Color::BrightYellow),
        104 => sgr.bg = Some(Color::BrightBlue),
        105 => sgr.bg = Some(Color::BrightMagenta),
        106 => sgr.bg = Some(Color::BrightCyan),
        107 => sgr.bg = Some(Color::BrightWhite),
        _ => { trace!("cansi: unknown SGR parameter {param}, ignoring") }
    }
}

/// Parse one CSI sequence from `bytes` starting after the `ESC[`.
///
/// On `Ok`, applies all SGR params to `sgr` and returns the byte index after the final `m`.
/// On `Err`, `sgr` is not modified.
fn parse_csi_sgr(bytes: &[u8], csi_start: usize, sgr: &mut SGR) -> Result<usize, ParseError> {
    let len = bytes.len();
    let mut i = csi_start + 2;

    // ECMA-48 §5.4: if the first parameter byte is in 0x3C..=0x3F, it's private use.
    if i < len && (0x3C..=0x3F).contains(&bytes[i]) {
        return Err(ParseError::PrivateSequence);
    }

    // Parse into a scratch copy so we only commit on success.
    let mut scratch = *sgr;
    let mut num: u8 = 0;

    while i < len {
        let b = bytes[i];
        match b {
            b'0'..=b'9' => {
                num = num
                    .checked_mul(10)
                    .and_then(|n| n.checked_add(b - b'0'))
                    .ok_or(ParseError::Overflow)?;
            }
            b';' => {
                apply_sgr_param(&mut scratch, num);
                num = 0;
            }
            b'm' => {
                apply_sgr_param(&mut scratch, num);
                *sgr = scratch;
                return Ok(i + 1);
            }
            0x40..=0x7E => return Err(ParseError::NotSgr(b)),
            _ => return Err(ParseError::InvalidByte(b)),
        }
        i += 1;
    }

    Err(ParseError::Truncated)
}

/// Skip forward from a CSI start to find the end of the sequence (byte after the final byte).
/// Used to strip non-SGR CSI sequences (e.g. cursor movement) without keeping them as text.
/// Returns `len` if no final byte is found (truncated sequence).
fn skip_csi(bytes: &[u8], csi_start: usize) -> usize {
    let mut j = csi_start + 2;
    while j < bytes.len() && !(0x40..=0x7E).contains(&bytes[j]) {
        j += 1;
    }
    if j < bytes.len() { j + 1 } else { bytes.len() }
}

enum Phase {
    /// Scanning printable text. The current run started at `text_start`.
    Text,
    /// The previous byte was ESC at `esc_pos`.
    Escape { esc_pos: usize },
}

struct Parser {
    sgr: SGR,
    /// Byte offset where the current pending text run begins.
    /// Text is not emitted until something forces a flush (an SGR sequence or end of input),
    /// so we lazily accumulate by just tracking the start position.
    text_start: usize,
    phase: Phase,
}

impl Parser {
    fn new() -> Self {
        Self {
            sgr: SGR::default(),
            phase: Phase::Text,
            text_start: 0
        }
    }

    /// Emit the accumulated text run `text[start..end]` tagged with `sgr`.
    /// Called when an SGR sequence (or end of input) closes the current run.
    /// No-ops on empty ranges so callers don't need to guard.
    fn flush_text<'t>(
        slices: &mut Vec<CategorisedSlice<'t>>,
        sgr: SGR,
        text: &'t str,
        start: usize,
        end: usize,
    ) {
        if start < end {
            slices.push(CategorisedSlice::with_sgr(sgr, &text[start..end], start, end));
        }
    }
}

/// Single-pass state-machine parser: walks the input byte-by-byte, interprets CSI SGR
/// sequences cumulatively, strips other CSI sequences, and returns styled text segments.
///
/// Malformed or truncated sequences are treated as literal text.
pub fn parse(text: &'_ str) -> Vec<CategorisedSlice<'_>> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut slices: Vec<CategorisedSlice> = Vec::new();
    let mut p = Parser::new();

    trace!("cansi::parse: {} byte input", len);

    let mut i: usize = 0;
    while i < len {
        match p.phase {
            Phase::Text => {
                match text[i..].find('\x1B') {
                    Some(rel) => {
                        let esc_pos = i + rel;
                        p.phase = Phase::Escape { esc_pos };
                        i = esc_pos + 1;
                    }
                    None => i = len,
                }
            }

            Phase::Escape { esc_pos } => {
                // `i` is the byte right after ESC. CSI requires ESC followed by '['.
                if bytes[i] != b'[' {
                    // Not CSI — ESC is just text. Don't advance; this byte could be ESC.
                    trace!("cansi::parse: bare ESC at byte {esc_pos} (next byte 0x{:02x}), not CSI, treating as text", bytes[i]);
                    p.phase = Phase::Text;
                    continue;
                }

                // We have ESC[ at esc_pos. Try to parse a CSI SGR sequence.
                let sgr_before = p.sgr;
                match parse_csi_sgr(bytes, esc_pos, &mut p.sgr) {
                    Ok(seq_end) => {
                        // Flush the run that ended just before ESC with its pre-sequence SGR,
                        // then start a new run after the sequence under the updated SGR.
                        trace!("cansi::parse: SGR sequence at {}..{}", esc_pos, seq_end);
                        Parser::flush_text(&mut slices, sgr_before, text, p.text_start, esc_pos);
                        i = seq_end;
                    }
                    Err(_e @ ParseError::Truncated) => {
                        // Input ended inside a sequence — treat everything from text_start
                        // to end-of-input as literal text (ESC[ included).
                        trace!("cansi::parse: {_e} at byte {esc_pos}, treating remainder as literal");
                        p.sgr = sgr_before;
                        i = len;
                        p.phase = Phase::Text;
                        continue;
                    }
                    Err(_e @ (ParseError::NotSgr(_) | ParseError::PrivateSequence)) => {
                        // Valid CSI but not SGR (e.g. cursor movement) — strip it silently,
                        // flushing the text before it and resuming after it.
                        trace!("cansi::parse: {_e} at byte {esc_pos}, stripping");
                        p.sgr = sgr_before;
                        Parser::flush_text(&mut slices, p.sgr, text, p.text_start, esc_pos);
                        i = skip_csi(bytes, esc_pos);
                    }
                    Err(_e @ (ParseError::InvalidByte(_) | ParseError::Overflow)) => {
                        // Malformed sequence — not a real escape, keep ESC as literal text
                        // and re-scan from the byte after ESC.
                        warn!("cansi::parse: {_e} at byte {esc_pos}, treating as literal text");
                        p.sgr = sgr_before;
                        i = esc_pos + 1;
                        p.phase = Phase::Text;
                        continue;
                    }
                }
                p.phase = Phase::Text;
                // Advance text_start past the consumed sequence so the next run begins cleanly.
                p.text_start = i;
            }
        }
    }

    Parser::flush_text(&mut slices, p.sgr, text, p.text_start, len);

    slices
}
