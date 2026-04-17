//! [![Latest Version](https://img.shields.io/crates/v/cansi.svg)](https://crates.io/crates/cansi)
//! [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/cansi)
//!
//! # cansi — ANSI escape code parser
//!
//! Parses text containing ANSI escape sequences and returns it split into styled segments.
//! Only [CSI SGR](https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters)
//! sequences are interpreted; other CSI sequences (cursor movement, etc.) are stripped silently.
//! cansi does not *produce* coloured text — for that, see crates like
//! [`colored`](https://crates.io/crates/colored).
//!
//! ## Usage
//!
//! ```rust
//! # use cansi::*;
//! # use colored::Colorize;
//! # use std::io::Write;
//! # colored::control::set_override(true);
//!
//! let v = &mut Vec::new();
//! write!(
//!   v,
//!   "Hello, {}{}{}{}{}{}",
//!   "w".white().on_red(),
//!   "o".cyan().on_green(),
//!   "r".magenta().on_yellow(),
//!   "l".blue().on_white(),
//!   "d".yellow().on_bright_cyan(),
//!   "!".bright_red().on_bright_yellow(),
//! )
//! .unwrap();
//!
//! let text = String::from_utf8_lossy(&v);
//! let result = categorise_text(&text);
//!
//! assert_eq!(result.len(), 7); // seven differently styled segments
//! assert_eq!("Hello, world!", &construct_text_no_codes(&result));
//!
//! // "Hello, " has no styling
//! assert_eq!(result[0].text, "Hello, ");
//! assert_eq!(result[0].fg, None);
//! assert_eq!(result[0].bg, None);
//!
//! // "w" is white on red
//! assert_eq!(result[1].text, "w");
//! assert_eq!(result[1].fg, Some(Color::White));
//! assert_eq!(result[1].bg, Some(Color::Red));
//! ```
//!
//! ## Style fields
//!
//! Every field on [`CategorisedSlice`] is `Option<T>`. `None` means the attribute was not set
//! by any escape sequence in the current run (i.e. it inherits the terminal default).
//! `Some(v)` means it was explicitly set to `v`.
//!
//! ## no_std support
//!
//! The `std` feature (enabled by default) can be replaced with `alloc` for no_std targets:
//!
//! ```toml
//! [dependencies]
//! cansi = { version = "2.2", default-features = false, features = ["alloc"] }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

#[macro_use]
mod logging;
mod parser;

#[cfg(test)]
mod tests;

/// The SGR (Select Graphic Rendition) state accumulated from escape sequences.
/// [spec](https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters)
#[derive(Clone, Copy, Default)]
#[allow(clippy::upper_case_acronyms)]
struct SGR {
    fg: Option<Color>,
    bg: Option<Color>,
    intensity: Option<Intensity>,
    italic: Option<bool>,
    underline: Option<bool>,
    blink: Option<bool>,
    reversed: Option<bool>,
    hidden: Option<bool>,
    strikethrough: Option<bool>,
}

/// The emphasis (bold, faint) states.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Intensity {
    /// Normal intensity (no emphasis).
    Normal,
    /// Bold.
    Bold,
    /// Faint.
    Faint,
}

/// The 16 standard ANSI colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

/// A contiguous run of text that shares the same SGR styling.
///
/// All style fields are `Option<T>`: `None` indicates the attribute was not set
/// by any escape sequence (terminal default applies); `Some(v)` means it was
/// explicitly set to `v`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CategorisedSlice<'text> {
    /// The text content (escape codes excluded).
    pub text: &'text str,
    /// Inclusive starting byte position in the original string.
    pub start: usize,
    /// Exclusive ending byte position in the original string.
    pub end: usize,

    /// Foreground (text) colour.
    pub fg: Option<Color>,
    /// Background colour.
    pub bg: Option<Color>,

    /// Bold / faint / normal emphasis.
    pub intensity: Option<Intensity>,

    /// Italic text.
    pub italic: Option<bool>,
    /// Underlined text.
    pub underline: Option<bool>,

    /// Blinking text.
    pub blink: Option<bool>,
    /// Reversed colours. See [reverse video](https://en.wikipedia.org/wiki/Reverse_video).
    pub reversed: Option<bool>,
    /// Invisible (concealed) text.
    pub hidden: Option<bool>,
    /// Struck-through text.
    pub strikethrough: Option<bool>,
}

impl<'text> CategorisedSlice<'text> {
    pub(crate) const fn with_sgr(sgr: SGR, text: &'text str, start: usize, end: usize) -> Self {
        let SGR {
            fg,
            bg,
            intensity,
            italic,
            underline,
            blink,
            reversed,
            hidden,
            strikethrough,
        } = sgr;

        Self {
            text,
            start,
            end,
            fg,
            bg,
            intensity,
            italic,
            underline,
            blink,
            reversed,
            hidden,
            strikethrough,
        }
    }

    const fn clone_style(&self, text: &'text str, start: usize, end: usize) -> Self {
        let mut c = *self;
        c.text = text;
        c.start = start;
        c.end = end;
        c
    }

    /// Converts this slice back into a [`colored::ColoredString`] with the same styling applied.
    ///
    /// Only styles explicitly set to an active value are emitted:
    /// - `fg`/`bg`: the corresponding colour code
    /// - `intensity`: bold or faint (normal / unset → no code)
    /// - boolean attributes (`italic`, `underline`, `blink`, `reversed`, `hidden`,
    ///   `strikethrough`): emitted only when `Some(true)`
    ///
    /// Requires the `colorized` feature.
    #[cfg(feature = "colorized")]
    pub fn as_colorized(&self) -> colored::ColoredString {
        use colored::Colorize as _;

        let mut s: colored::ColoredString = self.text.into();

        if let Some(fg) = self.fg {
            s = s.color(colored::Color::from(fg));
        }
        if let Some(bg) = self.bg {
            s = s.on_color(colored::Color::from(bg));
        }
        match self.intensity {
            Some(Intensity::Bold) => s = s.bold(),
            Some(Intensity::Faint) => s = s.dimmed(),
            _ => {}
        }
        if self.italic == Some(true) {
            s = s.italic();
        }
        if self.underline == Some(true) {
            s = s.underline();
        }
        if self.blink == Some(true) {
            s = s.blink();
        }
        if self.reversed == Some(true) {
            s = s.reversed();
        }
        if self.hidden == Some(true) {
            s = s.hidden();
        }
        if self.strikethrough == Some(true) {
            s = s.strikethrough();
        }
        s
    }
}

#[cfg(feature = "colorized")]
impl From<Color> for colored::Color {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => colored::Color::Black,
            Color::Red => colored::Color::Red,
            Color::Green => colored::Color::Green,
            Color::Yellow => colored::Color::Yellow,
            Color::Blue => colored::Color::Blue,
            Color::Magenta => colored::Color::Magenta,
            Color::Cyan => colored::Color::Cyan,
            Color::White => colored::Color::White,
            Color::BrightBlack => colored::Color::BrightBlack,
            Color::BrightRed => colored::Color::BrightRed,
            Color::BrightGreen => colored::Color::BrightGreen,
            Color::BrightYellow => colored::Color::BrightYellow,
            Color::BrightBlue => colored::Color::BrightBlue,
            Color::BrightMagenta => colored::Color::BrightMagenta,
            Color::BrightCyan => colored::Color::BrightCyan,
            Color::BrightWhite => colored::Color::BrightWhite,
        }
    }
}

/// A collection of [`CategorisedSlice`]s covering a parsed string.
pub type CategorisedSlices<'text> = Vec<CategorisedSlice<'text>>;

/// A single line's worth of [`CategorisedSlice`]s, as yielded by [`CategorisedLineIterator`].
///
/// The type alias is the same as [`CategorisedSlices`], so functions such as
/// [`construct_text_no_codes`] work on it directly.
pub type CategorisedLine<'text> = Vec<CategorisedSlice<'text>>;

/// Parses text containing ANSI escape sequences and returns each styled segment in order.
///
/// Escape codes are not included in the returned text slices. The original text (without
/// escape codes) can be reconstructed with [`construct_text_no_codes`].
///
/// # Example
/// ```rust
/// use cansi::*;
/// let result = categorise_text("\x1b[31mHello\x1b[0m, world!");
/// assert_eq!(result[0].text, "Hello");
/// assert_eq!(result[0].fg, Some(Color::Red));
/// assert_eq!(result[1].text, ", world!");
/// assert_eq!(result[1].fg, None);
/// ```
pub fn categorise_text(text: &str) -> CategorisedSlices<'_> {
    parser::parse(text)
}

/// Constructs a string from the categorised slices, with all ANSI escape codes removed.
///
/// # Example
/// ```rust
/// use cansi::*;
/// let categorised = categorise_text("\x1b[30mH\x1b[31me\x1b[32ml\x1b[33ml\x1b[34mo");
/// assert_eq!("Hello", &construct_text_no_codes(&categorised));
/// ```
pub fn construct_text_no_codes(categorised_slices: &CategorisedSlices) -> String {
    let mut s = String::with_capacity(
        categorised_slices
            .iter()
            .map(|x| x.text.len())
            .sum::<usize>(),
    );
    for sl in categorised_slices {
        s.push_str(sl.text);
    }
    s
}

/// Returns an iterator over lines of [`CategorisedSlice`]s, splitting on `\n` or `\r\n`.
///
/// Slices that span a line boundary are split and yielded with the same style on each side.
///
/// # Example
/// ```rust
/// # use colored::Colorize;
/// # use cansi::*;
/// # colored::control::set_override(true);
///
/// let s = format!("{}{}\nhow are you\r\ntoday", "hello, ".green(), "world".red());
/// let cat = categorise_text(&s);
/// let mut iter = line_iter(&cat);
///
/// let first = iter.next().unwrap();
/// assert_eq!(first[0].text, "hello, ");
/// assert_eq!(first[0].fg, Some(Color::Green));
/// assert_eq!(first[1].text, "world");
/// assert_eq!(first[1].fg, Some(Color::Red));
///
/// assert_eq!(&construct_text_no_codes(&iter.next().unwrap()), "how are you");
/// assert_eq!(&construct_text_no_codes(&iter.next().unwrap()), "today");
/// assert_eq!(iter.next(), None);
/// ```
pub fn line_iter<'text, 'iter>(
    categorised_slices: &'iter CategorisedSlices<'text>,
) -> CategorisedLineIterator<'text, 'iter> {
    CategorisedLineIterator {
        slices: categorised_slices,
        idx: 0,
        prev: None,
    }
}

/// Iterator over lines of [`CategorisedSlice`]s, splitting on `\n` or `\r\n`.
///
/// Constructed with [`line_iter`].
pub struct CategorisedLineIterator<'text, 'iter> {
    slices: &'iter CategorisedSlices<'text>,
    idx: usize,
    prev: Option<CategorisedSlice<'text>>,
}

impl<'text, 'iter> Iterator for CategorisedLineIterator<'text, 'iter> {
    type Item = CategorisedLine<'text>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut v = Vec::new();

        if let Some(prev) = &self.prev {
            let (first, remainder) = split_on_new_line(prev.text);

            v.push(prev.clone_style(&prev.text[..first], prev.start, prev.start + first));

            if let Some(remainder) = remainder {
                self.prev = Some(prev.clone_style(
                    &prev.text[remainder..],
                    prev.start + remainder,
                    prev.end,
                ));
                return Some(v);
            }

            self.prev = None;
        }

        while let Some(slice) = self.slices.get(self.idx) {
            self.idx += 1;

            let (first, remainder) = split_on_new_line(slice.text);

            if first > 0 || v.is_empty() {
                v.push(slice.clone_style(
                    &slice.text[..first],
                    slice.start,
                    slice.start + first,
                ));
            }

            if let Some(remainder) = remainder {
                if !slice.text[remainder..].is_empty() {
                    self.prev = Some(slice.clone_style(
                        &slice.text[remainder..],
                        slice.start + remainder,
                        slice.end,
                    ));
                }
                break;
            }
        }

        if v.is_empty() && self.idx >= self.slices.len() {
            None
        } else {
            Some(v)
        }
    }
}

/// Splits on the first `\r\n` or `\n`.
///
/// Returns `(exclusive_end_of_first, inclusive_start_of_remainder)`.
/// Both parts may be empty (e.g. `"\nHello"` gives `(0, Some(1))`).
fn split_on_new_line(txt: &str) -> (usize, Option<usize>) {
    let cr = txt.find('\r');
    let nl = txt.find('\n');

    match (cr, nl) {
        (None, None) => (txt.len(), None),
        (Some(_), None) => (txt.len(), None), // lone CR, no newline
        (None, Some(nl)) => (nl, Some(nl + 1)),
        (Some(cr), Some(nl)) => {
            if nl.saturating_sub(1) == cr {
                (cr, Some(nl + 1))
            } else {
                (nl, Some(nl + 1))
            }
        }
    }
}
