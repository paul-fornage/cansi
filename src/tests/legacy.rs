#![allow(deprecated)]


use crate::*;
use colored::Colorize;
use std::io::Write;


#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

fn print_bytes(bytes: &[u8]) {
    for ch in String::from_utf8_lossy(bytes).chars() {
        print!("{} ", ch);
    }
    println!();
    for byte in bytes {
        print!("{} ", byte);
    }
    println!();
}


#[test]
fn cover_other_parameters() {
    // colored doesn't always test all match arms, so i test here

    // no escape sequences
    let text = "test";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 0,
            end: 4,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );

    // empty sequences
    let text = "\x1b[;mtest";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );

    // empty text - doesn't add it
    let text = "\x1b[;mtest\x1b[;m\x1b[;m";
    assert_eq!(categorise_text(&text[..]).len(), 1);
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );

    // 22
    let text = "\x1b[1;22mtest";
    assert_eq!(
        categorise_text(&text[..])[0],
        CategorisedSlice {
            text: "test",
            start: 7,
            end: 11,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
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
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
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
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
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
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
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
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
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
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
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
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );
}



#[test]
fn line_iter_test() {
    let mut green = CategorisedSlice::default_style("", 0, 0);
    let mut red = CategorisedSlice::default_style("", 0, 0);
    green.fg_colour = Color::Green;
    red.fg_colour = Color::Red;

    let cat = categorise_text("hello, world");
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("hello, world", 0, 12)])
    );
    assert_eq!(iter.next(), None);

    let cat = categorise_text("hello, world\nhow are you");
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("hello, world", 0, 12)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("how are you", 13, 24)])
    );
    assert_eq!(iter.next(), None);

    let s = format!("{}{}\nhow are you", "hello, ".green(), "world".red());
    let cat = categorise_text(&s);
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![
            green.clone_style("hello, ", 5, 12),
            red.clone_style("world", 21, 26)
        ])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("how are you", 31, 42)])
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
            green.clone_style("hello, ", 5, 12),
            red.clone_style("world", 21, 26)
        ])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("how are you", 31, 42)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("today", 44, 49)])
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
        Some(vec![CategorisedSlice::default_style("", 0, 0)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("", 1, 1)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("", 2, 2)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("", 3, 3)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("", 4, 4)])
    );
    assert_eq!(iter.next(), None);

    let cat = categorise_text("\r\n\r\n\r\n\r\n");
    let mut iter = line_iter(&cat);
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("", 0, 0)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("", 2, 2)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("", 4, 4)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("", 6, 6)])
    );
    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("", 8, 8)])
    );
    assert_eq!(iter.next(), None);
}

#[test]
fn line_iter_newline_starts_with_esc() {
    let mut green = CategorisedSlice::default_style("", 0, 0);
    green.fg_colour = Color::Green;

    let s = format!("hello\n{}", "world".green());
    let cat = categorise_text(&s);
    let mut iter = line_iter(&cat);

    assert_eq!(
        iter.next(),
        Some(vec![CategorisedSlice::default_style("hello", 0, 5)])
    );
    assert_eq!(iter.next(), Some(vec![green.clone_style("world", 11, 16)]));
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

    let mut cyan = CategorisedSlice::default_style("", 0, 0);
    cyan.fg_colour = Color::Cyan;

    assert_eq!(
        iter.next(),
        Some(vec![
            cyan.clone_style("papyrus", 5, 12),
            CategorisedSlice::default_style("=> 5+6", 16, 22)
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
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        })
    );

    assert_eq!(
        c.next(),
        Some(CategorisedSlice {
            text: "🌍",
            start: 13,
            end: 17,
            fg_colour: Color::Red,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: true,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        })
    );

    assert_eq!(
        c.next(),
        Some(CategorisedSlice {
            text: "!",
            start: 21,
            end: 22,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        })
    );

    assert_eq!(c.next(), None);
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
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );

    assert_eq!(
        result[1],
        CategorisedSlice {
            text: "w",
            start: 15,
            end: 16,
            fg_colour: Color::White,
            bg_colour: Color::Red,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );

    assert_eq!(
        result[2],
        CategorisedSlice {
            text: "o",
            start: 28,
            end: 29,
            fg_colour: Color::Cyan,
            bg_colour: Color::Green,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );

    assert_eq!(
        result[3],
        CategorisedSlice {
            text: "r",
            start: 41,
            end: 42,
            fg_colour: Color::Magenta,
            bg_colour: Color::Yellow,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );

    assert_eq!(
        result[4],
        CategorisedSlice {
            text: "l",
            start: 54,
            end: 55,
            fg_colour: Color::Blue,
            bg_colour: Color::White,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );

    assert_eq!(
        result[5],
        CategorisedSlice {
            text: "d",
            start: 68,
            end: 69,
            fg_colour: Color::Yellow,
            bg_colour: Color::BrightCyan,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );

    assert_eq!(
        result[6],
        CategorisedSlice {
            text: "!",
            start: 82,
            end: 83,
            fg_colour: Color::BrightRed,
            bg_colour: Color::BrightYellow,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        }
    );
}

#[test]
fn test_colored_function() {
    let test_string: &str = "test";
    let v = &mut Vec::new();

    write!(v, "{}", "test".black()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::Black,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "black()"
    );
    v.clear();
    write!(v, "{}", "test".red()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::Red,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "red()"
    );
    v.clear();
    write!(v, "{}", "test".green()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::Green,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "green()"
    );
    v.clear();
    write!(v, "{}", "test".yellow()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::Yellow,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "yellow()"
    );
    v.clear();
    write!(v, "{}", "test".blue()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::Blue,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "blue()"
    );
    v.clear();
    write!(v, "{}", "test".magenta()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::Magenta,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "magenta()"
    );
    v.clear();
    write!(v, "{}", "test".purple()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::Magenta,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "purple()"
    );
    v.clear();
    write!(v, "{}", "test".cyan()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::Cyan,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "cyan()"
    );
    v.clear();
    write!(v, "{}", "test".white()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "white()"
    );
    v.clear();
    write!(v, "{}", "test".bright_black()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::BrightBlack,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bright_black()"
    );
    v.clear();
    write!(v, "{}", "test".bright_red()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::BrightRed,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bright_red()"
    );
    v.clear();
    write!(v, "{}", "test".bright_green()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::BrightGreen,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bright_green()"
    );
    v.clear();
    write!(v, "{}", "test".bright_yellow()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::BrightYellow,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bright_yellow()"
    );
    v.clear();
    write!(v, "{}", "test".bright_blue()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::BrightBlue,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bright_blue()"
    );
    v.clear();
    write!(v, "{}", "test".bright_magenta()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::BrightMagenta,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bright_magenta()"
    );
    v.clear();
    write!(v, "{}", "test".bright_purple()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::BrightMagenta,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bright_purple()"
    );
    v.clear();
    write!(v, "{}", "test".bright_cyan()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::BrightCyan,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bright_cyan()"
    );
    v.clear();
    write!(v, "{}", "test".bright_white()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::BrightWhite,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bright_white()"
    );
    v.clear();
    write!(v, "{}", "test".on_black()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_black()"
    );
    v.clear();
    write!(v, "{}", "test".on_red()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::Red,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_red()"
    );
    v.clear();
    write!(v, "{}", "test".on_green()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::Green,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_green()"
    );
    v.clear();
    write!(v, "{}", "test".on_yellow()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::Yellow,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_yellow()"
    );
    v.clear();
    write!(v, "{}", "test".on_blue()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::Blue,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_blue()"
    );
    v.clear();
    write!(v, "{}", "test".on_magenta()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::Magenta,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_magenta()"
    );
    v.clear();
    write!(v, "{}", "test".on_purple()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::Magenta,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_purple()"
    );
    v.clear();
    write!(v, "{}", "test".on_cyan()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::Cyan,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_cyan()"
    );
    v.clear();
    write!(v, "{}", "test".on_white()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 5,
            end: 9,
            fg_colour: Color::White,
            bg_colour: Color::White,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_white()"
    );
    v.clear();
    write!(v, "{}", "test".on_bright_black()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 6,
            end: 10,
            fg_colour: Color::White,
            bg_colour: Color::BrightBlack,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_bright_black()"
    );
    v.clear();
    write!(v, "{}", "test".on_bright_red()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 6,
            end: 10,
            fg_colour: Color::White,
            bg_colour: Color::BrightRed,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_bright_red()"
    );
    v.clear();
    write!(v, "{}", "test".on_bright_green()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 6,
            end: 10,
            fg_colour: Color::White,
            bg_colour: Color::BrightGreen,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_bright_green()"
    );
    v.clear();
    write!(v, "{}", "test".on_bright_yellow()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 6,
            end: 10,
            fg_colour: Color::White,
            bg_colour: Color::BrightYellow,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_bright_yellow()"
    );
    v.clear();
    write!(v, "{}", "test".on_bright_blue()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 6,
            end: 10,
            fg_colour: Color::White,
            bg_colour: Color::BrightBlue,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_bright_blue()"
    );
    v.clear();
    write!(v, "{}", "test".on_bright_magenta()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 6,
            end: 10,
            fg_colour: Color::White,
            bg_colour: Color::BrightMagenta,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_bright_magenta()"
    );
    v.clear();
    write!(v, "{}", "test".on_bright_purple()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 6,
            end: 10,
            fg_colour: Color::White,
            bg_colour: Color::BrightMagenta,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_bright_purple()"
    );
    v.clear();
    write!(v, "{}", "test".on_bright_cyan()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 6,
            end: 10,
            fg_colour: Color::White,
            bg_colour: Color::BrightCyan,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_bright_cyan()"
    );
    v.clear();
    write!(v, "{}", "test".on_bright_white()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 6,
            end: 10,
            fg_colour: Color::White,
            bg_colour: Color::BrightWhite,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "on_bright_white()"
    );
    v.clear();
    write!(v, "{}", "test".clear()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 0,
            end: 4,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "clear()"
    );
    v.clear();
    write!(v, "{}", "test".normal()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 0,
            end: 4,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "normal()"
    );
    v.clear();
    write!(v, "{}", "test".bold()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Bold,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "bold()"
    );
    v.clear();
    write!(v, "{}", "test".dimmed()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Faint,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "dimmed()"
    );
    v.clear();
    write!(v, "{}", "test".italic()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: true,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "italic()"
    );
    v.clear();
    write!(v, "{}", "test".underline()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: true,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "underline()"
    );
    v.clear();
    write!(v, "{}", "test".blink()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: true,
            reversed: false,
            hidden: false,
            strikethrough: false
        },
        "blink()"
    );
    v.clear();
    write!(v, "{}", "test".reverse()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: true,
            hidden: false,
            strikethrough: false
        },
        "reverse()"
    );
    v.clear();
    write!(v, "{}", "test".reversed()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: true,
            hidden: false,
            strikethrough: false
        },
        "reversed()"
    );
    v.clear();
    write!(v, "{}", "test".hidden()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: true,
            strikethrough: false
        },
        "hidden()"
    );
    v.clear();
    write!(v, "{}", "test".strikethrough()).unwrap();
    print_bytes(&v);
    assert_eq!(
        categorise_text(&String::from_utf8_lossy(&v))[0],
        CategorisedSlice {
            text: test_string,
            start: 4,
            end: 8,
            fg_colour: Color::White,
            bg_colour: Color::Black,
            intensity: Intensity::Normal,
            italic: false,
            underline: false,
            blink: false,
            reversed: false,
            hidden: false,
            strikethrough: true
        },
        "strikethrough()"
    );
    v.clear();
}