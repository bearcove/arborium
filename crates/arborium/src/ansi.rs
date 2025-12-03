//! ANSI terminal rendering for syntax-highlighted code.
//!
//! This module renders tree-sitter highlighted code with ANSI escape sequences
//! for terminal output. Uses 24-bit true color by default.
//!
//! # Example
//!
//! ```rust,ignore
//! use arborium_highlight::{ansi, HIGHLIGHT_NAMES, Highlighter, HighlightConfiguration};
//!
//! let mut highlighter = Highlighter::new();
//! let mut output = Vec::new();
//! ansi::render(&mut output, &mut highlighter, &config, "fn main() {}", |_| None).unwrap();
//! print!("{}", String::from_utf8_lossy(&output));
//! ```

use std::io::{self, Write};

use crate::tree_sitter_highlight::{
    Highlight, HighlightConfiguration, HighlightEvent, Highlighter,
};

/// RGB color for ANSI output.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse from hex string like "cba6f7" or "#cba6f7"
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }
}

/// Style for a highlight category.
#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            fg: None,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }

    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub const fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Write the ANSI escape sequence to enable this style.
    pub fn write_start(&self, w: &mut dyn Write) -> io::Result<()> {
        let mut codes = Vec::new();

        if self.bold {
            codes.push("1".to_string());
        }
        if self.italic {
            codes.push("3".to_string());
        }
        if self.underline {
            codes.push("4".to_string());
        }
        if self.strikethrough {
            codes.push("9".to_string());
        }
        if let Some(color) = self.fg {
            codes.push(format!("38;2;{};{};{}", color.r, color.g, color.b));
        }

        if !codes.is_empty() {
            write!(w, "\x1b[{}m", codes.join(";"))?;
        }
        Ok(())
    }

    /// Write the ANSI reset sequence.
    pub fn write_end(&self, w: &mut dyn Write) -> io::Result<()> {
        if self.fg.is_some() || self.bold || self.italic || self.underline || self.strikethrough {
            write!(w, "\x1b[0m")?;
        }
        Ok(())
    }
}

/// A theme mapping highlight indices to styles.
pub struct Theme {
    styles: [Style; 34],
}

impl Theme {
    /// Create a theme with all default (unstyled) entries.
    pub const fn empty() -> Self {
        Self {
            styles: [Style::new(); 34],
        }
    }

    /// Get the style for a highlight index.
    pub fn style(&self, index: usize) -> Option<&Style> {
        self.styles.get(index)
    }
}

// Catppuccin Mocha colors
const MAUVE: Color = Color::new(0xcb, 0xa6, 0xf7);
const BLUE: Color = Color::new(0x89, 0xb4, 0xfa);
const SKY: Color = Color::new(0x89, 0xdc, 0xeb);
const TEAL: Color = Color::new(0x94, 0xe2, 0xd5);
const GREEN: Color = Color::new(0xa6, 0xe3, 0xa1);
const YELLOW: Color = Color::new(0xf9, 0xe2, 0xaf);
const PEACH: Color = Color::new(0xfa, 0xb3, 0x87);
const RED: Color = Color::new(0xf3, 0x8b, 0xa8);
const PINK: Color = Color::new(0xf5, 0xc2, 0xe7);
const OVERLAY0: Color = Color::new(0x6c, 0x70, 0x86);
const OVERLAY2: Color = Color::new(0x93, 0x99, 0xb2);
const TEXT: Color = Color::new(0xcd, 0xd6, 0xf4);

/// Default theme based on Catppuccin Mocha.
pub const CATPPUCCIN_MOCHA: Theme = Theme {
    styles: [
        Style::new().fg(YELLOW),           // 0:  attribute
        Style::new().fg(PEACH),            // 1:  constant
        Style::new().fg(SKY),              // 2:  function.builtin
        Style::new().fg(BLUE),             // 3:  function
        Style::new().fg(MAUVE),            // 4:  keyword
        Style::new().fg(TEAL),             // 5:  operator
        Style::new().fg(BLUE),             // 6:  property
        Style::new().fg(OVERLAY2),         // 7:  punctuation
        Style::new().fg(OVERLAY2),         // 8:  punctuation.bracket
        Style::new().fg(OVERLAY2),         // 9:  punctuation.delimiter
        Style::new().fg(GREEN),            // 10: string
        Style::new().fg(RED),              // 11: string.special
        Style::new().fg(BLUE),             // 12: tag
        Style::new().fg(YELLOW),           // 13: type
        Style::new().fg(YELLOW),           // 14: type.builtin
        Style::new().fg(TEXT),             // 15: variable
        Style::new().fg(RED),              // 16: variable.builtin
        Style::new().fg(PEACH),            // 17: variable.parameter
        Style::new().fg(OVERLAY0),         // 18: comment
        Style::new().fg(TEAL),             // 19: macro
        Style::new().fg(PINK),             // 20: label
        Style::new().fg(GREEN),            // 21: diff.addition
        Style::new().fg(RED),              // 22: diff.deletion
        Style::new().fg(PEACH),            // 23: number
        Style::new().fg(TEXT),             // 24: text.literal
        Style::new().italic(),             // 25: text.emphasis
        Style::new().bold(),               // 26: text.strong
        Style::new().fg(BLUE).underline(), // 27: text.uri
        Style::new().fg(SKY),              // 28: text.reference
        Style::new().fg(PEACH),            // 29: string.escape
        Style::new().fg(MAUVE).bold(),     // 30: text.title
        Style::new().fg(PINK),             // 31: punctuation.special
        Style::new().strikethrough(),      // 32: text.strikethrough
        Style::new().fg(TEXT),             // 33: spell
    ],
};

/// Catppuccin Latte (light theme).
pub const CATPPUCCIN_LATTE: Theme = Theme {
    styles: [
        Style::new().fg(Color::new(0xdf, 0x8e, 0x1d)), // 0:  attribute - yellow
        Style::new().fg(Color::new(0xfe, 0x64, 0x0b)), // 1:  constant - peach
        Style::new().fg(Color::new(0x04, 0xa5, 0xe5)), // 2:  function.builtin - sky
        Style::new().fg(Color::new(0x1e, 0x66, 0xf5)), // 3:  function - blue
        Style::new().fg(Color::new(0x88, 0x39, 0xef)), // 4:  keyword - mauve
        Style::new().fg(Color::new(0x17, 0x92, 0x99)), // 5:  operator - teal
        Style::new().fg(Color::new(0x1e, 0x66, 0xf5)), // 6:  property - blue
        Style::new().fg(Color::new(0x7c, 0x7f, 0x93)), // 7:  punctuation - overlay2
        Style::new().fg(Color::new(0x7c, 0x7f, 0x93)), // 8:  punctuation.bracket
        Style::new().fg(Color::new(0x7c, 0x7f, 0x93)), // 9:  punctuation.delimiter
        Style::new().fg(Color::new(0x40, 0xa0, 0x2b)), // 10: string - green
        Style::new().fg(Color::new(0xd2, 0x0f, 0x39)), // 11: string.special - red
        Style::new().fg(Color::new(0x1e, 0x66, 0xf5)), // 12: tag - blue
        Style::new().fg(Color::new(0xdf, 0x8e, 0x1d)), // 13: type - yellow
        Style::new().fg(Color::new(0xdf, 0x8e, 0x1d)), // 14: type.builtin
        Style::new().fg(Color::new(0x4c, 0x4f, 0x69)), // 15: variable - text
        Style::new().fg(Color::new(0xd2, 0x0f, 0x39)), // 16: variable.builtin - red
        Style::new().fg(Color::new(0xfe, 0x64, 0x0b)), // 17: variable.parameter - peach
        Style::new().fg(Color::new(0x9c, 0xa0, 0xb0)), // 18: comment - overlay0
        Style::new().fg(Color::new(0x17, 0x92, 0x99)), // 19: macro - teal
        Style::new().fg(Color::new(0xea, 0x76, 0xcb)), // 20: label - pink
        Style::new().fg(Color::new(0x40, 0xa0, 0x2b)), // 21: diff.addition - green
        Style::new().fg(Color::new(0xd2, 0x0f, 0x39)), // 22: diff.deletion - red
        Style::new().fg(Color::new(0xfe, 0x64, 0x0b)), // 23: number - peach
        Style::new().fg(Color::new(0x4c, 0x4f, 0x69)), // 24: text.literal
        Style::new().italic(),                         // 25: text.emphasis
        Style::new().bold(),                           // 26: text.strong
        Style::new().fg(Color::new(0x1e, 0x66, 0xf5)).underline(), // 27: text.uri
        Style::new().fg(Color::new(0x04, 0xa5, 0xe5)), // 28: text.reference - sky
        Style::new().fg(Color::new(0xfe, 0x64, 0x0b)), // 29: string.escape - peach
        Style::new().fg(Color::new(0x88, 0x39, 0xef)).bold(), // 30: text.title - mauve
        Style::new().fg(Color::new(0xea, 0x76, 0xcb)), // 31: punctuation.special - pink
        Style::new().strikethrough(),                  // 32: text.strikethrough
        Style::new().fg(Color::new(0x4c, 0x4f, 0x69)), // 33: spell
    ],
};

/// Dracula theme.
pub const DRACULA: Theme = Theme {
    styles: [
        Style::new().fg(Color::new(0x50, 0xfa, 0x7b)), // 0:  attribute - green
        Style::new().fg(Color::new(0xbd, 0x93, 0xf9)), // 1:  constant - purple
        Style::new().fg(Color::new(0x8b, 0xe9, 0xfd)), // 2:  function.builtin - cyan
        Style::new().fg(Color::new(0x50, 0xfa, 0x7b)), // 3:  function - green
        Style::new().fg(Color::new(0xff, 0x79, 0xc6)), // 4:  keyword - pink
        Style::new().fg(Color::new(0xff, 0x79, 0xc6)), // 5:  operator - pink
        Style::new().fg(Color::new(0x8b, 0xe9, 0xfd)), // 6:  property - cyan
        Style::new().fg(Color::new(0xf8, 0xf8, 0xf2)), // 7:  punctuation - fg
        Style::new().fg(Color::new(0xf8, 0xf8, 0xf2)), // 8:  punctuation.bracket
        Style::new().fg(Color::new(0xf8, 0xf8, 0xf2)), // 9:  punctuation.delimiter
        Style::new().fg(Color::new(0xf1, 0xfa, 0x8c)), // 10: string - yellow
        Style::new().fg(Color::new(0xff, 0x55, 0x55)), // 11: string.special - red
        Style::new().fg(Color::new(0xff, 0x79, 0xc6)), // 12: tag - pink
        Style::new().fg(Color::new(0x8b, 0xe9, 0xfd)), // 13: type - cyan
        Style::new().fg(Color::new(0x8b, 0xe9, 0xfd)), // 14: type.builtin
        Style::new().fg(Color::new(0xf8, 0xf8, 0xf2)), // 15: variable - fg
        Style::new().fg(Color::new(0xbd, 0x93, 0xf9)), // 16: variable.builtin - purple
        Style::new().fg(Color::new(0xff, 0xb8, 0x6c)), // 17: variable.parameter - orange
        Style::new().fg(Color::new(0x62, 0x72, 0xa4)), // 18: comment
        Style::new().fg(Color::new(0x50, 0xfa, 0x7b)), // 19: macro - green
        Style::new().fg(Color::new(0x8b, 0xe9, 0xfd)), // 20: label - cyan
        Style::new().fg(Color::new(0x50, 0xfa, 0x7b)), // 21: diff.addition - green
        Style::new().fg(Color::new(0xff, 0x55, 0x55)), // 22: diff.deletion - red
        Style::new().fg(Color::new(0xbd, 0x93, 0xf9)), // 23: number - purple
        Style::new().fg(Color::new(0xf8, 0xf8, 0xf2)), // 24: text.literal
        Style::new().italic(),                         // 25: text.emphasis
        Style::new().bold(),                           // 26: text.strong
        Style::new().fg(Color::new(0x8b, 0xe9, 0xfd)).underline(), // 27: text.uri
        Style::new().fg(Color::new(0x8b, 0xe9, 0xfd)), // 28: text.reference
        Style::new().fg(Color::new(0xff, 0x79, 0xc6)), // 29: string.escape - pink
        Style::new().fg(Color::new(0xbd, 0x93, 0xf9)).bold(), // 30: text.title
        Style::new().fg(Color::new(0xff, 0x79, 0xc6)), // 31: punctuation.special
        Style::new().strikethrough(),                  // 32: text.strikethrough
        Style::new().fg(Color::new(0xf8, 0xf8, 0xf2)), // 33: spell
    ],
};

/// Tokyo Night theme.
pub const TOKYO_NIGHT: Theme = Theme {
    styles: [
        Style::new().fg(Color::new(0xe0, 0xaf, 0x68)), // 0:  attribute - yellow
        Style::new().fg(Color::new(0xff, 0x9e, 0x64)), // 1:  constant - orange
        Style::new().fg(Color::new(0x7d, 0xcf, 0xff)), // 2:  function.builtin - sky
        Style::new().fg(Color::new(0x7a, 0xa2, 0xf7)), // 3:  function - blue
        Style::new().fg(Color::new(0xbb, 0x9a, 0xf7)), // 4:  keyword - purple
        Style::new().fg(Color::new(0x89, 0xdd, 0xff)), // 5:  operator - cyan
        Style::new().fg(Color::new(0x7a, 0xa2, 0xf7)), // 6:  property - blue
        Style::new().fg(Color::new(0xa9, 0xb1, 0xd6)), // 7:  punctuation - fg
        Style::new().fg(Color::new(0xa9, 0xb1, 0xd6)), // 8:  punctuation.bracket
        Style::new().fg(Color::new(0xa9, 0xb1, 0xd6)), // 9:  punctuation.delimiter
        Style::new().fg(Color::new(0x9e, 0xce, 0x6a)), // 10: string - green
        Style::new().fg(Color::new(0xf7, 0x76, 0x8e)), // 11: string.special - red
        Style::new().fg(Color::new(0xf7, 0x76, 0x8e)), // 12: tag - red
        Style::new().fg(Color::new(0x2a, 0xc3, 0xde)), // 13: type - cyan
        Style::new().fg(Color::new(0x2a, 0xc3, 0xde)), // 14: type.builtin
        Style::new().fg(Color::new(0xc0, 0xca, 0xf5)), // 15: variable - text
        Style::new().fg(Color::new(0xf7, 0x76, 0x8e)), // 16: variable.builtin - red
        Style::new().fg(Color::new(0xe0, 0xaf, 0x68)), // 17: variable.parameter - yellow
        Style::new().fg(Color::new(0x56, 0x5f, 0x89)), // 18: comment
        Style::new().fg(Color::new(0x7d, 0xcf, 0xff)), // 19: macro - sky
        Style::new().fg(Color::new(0xff, 0x9e, 0x64)), // 20: label - orange
        Style::new().fg(Color::new(0x9e, 0xce, 0x6a)), // 21: diff.addition - green
        Style::new().fg(Color::new(0xf7, 0x76, 0x8e)), // 22: diff.deletion - red
        Style::new().fg(Color::new(0xff, 0x9e, 0x64)), // 23: number - orange
        Style::new().fg(Color::new(0xa9, 0xb1, 0xd6)), // 24: text.literal
        Style::new().italic(),                         // 25: text.emphasis
        Style::new().bold(),                           // 26: text.strong
        Style::new().fg(Color::new(0x7a, 0xa2, 0xf7)).underline(), // 27: text.uri
        Style::new().fg(Color::new(0x7d, 0xcf, 0xff)), // 28: text.reference
        Style::new().fg(Color::new(0x89, 0xdd, 0xff)), // 29: string.escape
        Style::new().fg(Color::new(0xbb, 0x9a, 0xf7)).bold(), // 30: text.title
        Style::new().fg(Color::new(0x89, 0xdd, 0xff)), // 31: punctuation.special
        Style::new().strikethrough(),                  // 32: text.strikethrough
        Style::new().fg(Color::new(0xa9, 0xb1, 0xd6)), // 33: spell
    ],
};

/// Nord theme.
pub const NORD: Theme = Theme {
    styles: [
        Style::new().fg(Color::new(0x8f, 0xbc, 0xbb)), // 0:  attribute - nord7
        Style::new().fg(Color::new(0xb4, 0x8e, 0xad)), // 1:  constant - nord15
        Style::new().fg(Color::new(0x88, 0xc0, 0xd0)), // 2:  function.builtin - nord8
        Style::new().fg(Color::new(0x88, 0xc0, 0xd0)), // 3:  function - nord8
        Style::new().fg(Color::new(0x81, 0xa1, 0xc1)), // 4:  keyword - nord9
        Style::new().fg(Color::new(0x81, 0xa1, 0xc1)), // 5:  operator - nord9
        Style::new().fg(Color::new(0x88, 0xc0, 0xd0)), // 6:  property - nord8
        Style::new().fg(Color::new(0xec, 0xef, 0xf4)), // 7:  punctuation - nord6
        Style::new().fg(Color::new(0xec, 0xef, 0xf4)), // 8:  punctuation.bracket
        Style::new().fg(Color::new(0xec, 0xef, 0xf4)), // 9:  punctuation.delimiter
        Style::new().fg(Color::new(0xa3, 0xbe, 0x8c)), // 10: string - nord14
        Style::new().fg(Color::new(0xbf, 0x61, 0x6a)), // 11: string.special - nord11
        Style::new().fg(Color::new(0x81, 0xa1, 0xc1)), // 12: tag - nord9
        Style::new().fg(Color::new(0x8f, 0xbc, 0xbb)), // 13: type - nord7
        Style::new().fg(Color::new(0x8f, 0xbc, 0xbb)), // 14: type.builtin
        Style::new().fg(Color::new(0xec, 0xef, 0xf4)), // 15: variable - nord6
        Style::new().fg(Color::new(0xbf, 0x61, 0x6a)), // 16: variable.builtin - nord11
        Style::new().fg(Color::new(0xd0, 0x87, 0x70)), // 17: variable.parameter - nord12
        Style::new().fg(Color::new(0x61, 0x6e, 0x88)), // 18: comment - nord3 bright
        Style::new().fg(Color::new(0x5e, 0x81, 0xac)), // 19: macro - nord10
        Style::new().fg(Color::new(0xd0, 0x87, 0x70)), // 20: label - nord12
        Style::new().fg(Color::new(0xa3, 0xbe, 0x8c)), // 21: diff.addition - nord14
        Style::new().fg(Color::new(0xbf, 0x61, 0x6a)), // 22: diff.deletion - nord11
        Style::new().fg(Color::new(0xb4, 0x8e, 0xad)), // 23: number - nord15
        Style::new().fg(Color::new(0xec, 0xef, 0xf4)), // 24: text.literal
        Style::new().italic(),                         // 25: text.emphasis
        Style::new().bold(),                           // 26: text.strong
        Style::new().fg(Color::new(0x88, 0xc0, 0xd0)).underline(), // 27: text.uri
        Style::new().fg(Color::new(0x88, 0xc0, 0xd0)), // 28: text.reference
        Style::new().fg(Color::new(0xeb, 0xcb, 0x8b)), // 29: string.escape - nord13
        Style::new().fg(Color::new(0x81, 0xa1, 0xc1)).bold(), // 30: text.title
        Style::new().fg(Color::new(0xd0, 0x87, 0x70)), // 31: punctuation.special
        Style::new().strikethrough(),                  // 32: text.strikethrough
        Style::new().fg(Color::new(0xec, 0xef, 0xf4)), // 33: spell
    ],
};

/// One Dark theme.
pub const ONE_DARK: Theme = Theme {
    styles: [
        Style::new().fg(Color::new(0xd1, 0x9a, 0x66)), // 0:  attribute - orange
        Style::new().fg(Color::new(0xd1, 0x9a, 0x66)), // 1:  constant - orange
        Style::new().fg(Color::new(0x56, 0xb6, 0xc2)), // 2:  function.builtin - cyan
        Style::new().fg(Color::new(0x61, 0xaf, 0xef)), // 3:  function - blue
        Style::new().fg(Color::new(0xc6, 0x78, 0xdd)), // 4:  keyword - purple
        Style::new().fg(Color::new(0x56, 0xb6, 0xc2)), // 5:  operator - cyan
        Style::new().fg(Color::new(0xe0, 0x6c, 0x75)), // 6:  property - red
        Style::new().fg(Color::new(0xab, 0xb2, 0xbf)), // 7:  punctuation - fg
        Style::new().fg(Color::new(0xab, 0xb2, 0xbf)), // 8:  punctuation.bracket
        Style::new().fg(Color::new(0xab, 0xb2, 0xbf)), // 9:  punctuation.delimiter
        Style::new().fg(Color::new(0x98, 0xc3, 0x79)), // 10: string - green
        Style::new().fg(Color::new(0xe0, 0x6c, 0x75)), // 11: string.special - red
        Style::new().fg(Color::new(0xe0, 0x6c, 0x75)), // 12: tag - red
        Style::new().fg(Color::new(0xe5, 0xc0, 0x7b)), // 13: type - yellow
        Style::new().fg(Color::new(0xe5, 0xc0, 0x7b)), // 14: type.builtin
        Style::new().fg(Color::new(0xab, 0xb2, 0xbf)), // 15: variable - fg
        Style::new().fg(Color::new(0xe0, 0x6c, 0x75)), // 16: variable.builtin - red
        Style::new().fg(Color::new(0xd1, 0x9a, 0x66)), // 17: variable.parameter - orange
        Style::new().fg(Color::new(0x5c, 0x63, 0x70)), // 18: comment
        Style::new().fg(Color::new(0x56, 0xb6, 0xc2)), // 19: macro - cyan
        Style::new().fg(Color::new(0xe0, 0x6c, 0x75)), // 20: label - red
        Style::new().fg(Color::new(0x98, 0xc3, 0x79)), // 21: diff.addition - green
        Style::new().fg(Color::new(0xe0, 0x6c, 0x75)), // 22: diff.deletion - red
        Style::new().fg(Color::new(0xd1, 0x9a, 0x66)), // 23: number - orange
        Style::new().fg(Color::new(0xab, 0xb2, 0xbf)), // 24: text.literal
        Style::new().italic(),                         // 25: text.emphasis
        Style::new().bold(),                           // 26: text.strong
        Style::new().fg(Color::new(0x61, 0xaf, 0xef)).underline(), // 27: text.uri
        Style::new().fg(Color::new(0x56, 0xb6, 0xc2)), // 28: text.reference
        Style::new().fg(Color::new(0x56, 0xb6, 0xc2)), // 29: string.escape
        Style::new().fg(Color::new(0xe0, 0x6c, 0x75)).bold(), // 30: text.title
        Style::new().fg(Color::new(0xc6, 0x78, 0xdd)), // 31: punctuation.special
        Style::new().strikethrough(),                  // 32: text.strikethrough
        Style::new().fg(Color::new(0xab, 0xb2, 0xbf)), // 33: spell
    ],
};

/// GitHub Dark theme.
pub const GITHUB_DARK: Theme = Theme {
    styles: [
        Style::new().fg(Color::new(0x79, 0xc0, 0xff)), // 0:  attribute - blue
        Style::new().fg(Color::new(0x79, 0xc0, 0xff)), // 1:  constant - blue
        Style::new().fg(Color::new(0xd2, 0xa8, 0xff)), // 2:  function.builtin - purple
        Style::new().fg(Color::new(0xd2, 0xa8, 0xff)), // 3:  function - purple
        Style::new().fg(Color::new(0xff, 0x7b, 0x72)), // 4:  keyword - red
        Style::new().fg(Color::new(0xff, 0x7b, 0x72)), // 5:  operator - red
        Style::new().fg(Color::new(0x79, 0xc0, 0xff)), // 6:  property - blue
        Style::new().fg(Color::new(0xc9, 0xd1, 0xd9)), // 7:  punctuation - fg
        Style::new().fg(Color::new(0xc9, 0xd1, 0xd9)), // 8:  punctuation.bracket
        Style::new().fg(Color::new(0xc9, 0xd1, 0xd9)), // 9:  punctuation.delimiter
        Style::new().fg(Color::new(0xa5, 0xd6, 0xff)), // 10: string - light blue
        Style::new().fg(Color::new(0xff, 0x7b, 0x72)), // 11: string.special - red
        Style::new().fg(Color::new(0x7e, 0xe7, 0x87)), // 12: tag - green
        Style::new().fg(Color::new(0xff, 0xa6, 0x57)), // 13: type - orange
        Style::new().fg(Color::new(0xff, 0xa6, 0x57)), // 14: type.builtin
        Style::new().fg(Color::new(0xc9, 0xd1, 0xd9)), // 15: variable - fg
        Style::new().fg(Color::new(0xff, 0xa6, 0x57)), // 16: variable.builtin - orange
        Style::new().fg(Color::new(0xff, 0xa6, 0x57)), // 17: variable.parameter - orange
        Style::new().fg(Color::new(0x8b, 0x94, 0x9e)), // 18: comment
        Style::new().fg(Color::new(0x79, 0xc0, 0xff)), // 19: macro - blue
        Style::new().fg(Color::new(0xd2, 0xa8, 0xff)), // 20: label - purple
        Style::new().fg(Color::new(0x7e, 0xe7, 0x87)), // 21: diff.addition - green
        Style::new().fg(Color::new(0xff, 0x7b, 0x72)), // 22: diff.deletion - red
        Style::new().fg(Color::new(0x79, 0xc0, 0xff)), // 23: number - blue
        Style::new().fg(Color::new(0xc9, 0xd1, 0xd9)), // 24: text.literal
        Style::new().italic(),                         // 25: text.emphasis
        Style::new().bold(),                           // 26: text.strong
        Style::new().fg(Color::new(0x58, 0xa6, 0xff)).underline(), // 27: text.uri
        Style::new().fg(Color::new(0xd2, 0xa8, 0xff)), // 28: text.reference
        Style::new().fg(Color::new(0x79, 0xc0, 0xff)), // 29: string.escape
        Style::new().fg(Color::new(0xd2, 0xa8, 0xff)).bold(), // 30: text.title
        Style::new().fg(Color::new(0x79, 0xc0, 0xff)), // 31: punctuation.special
        Style::new().strikethrough(),                  // 32: text.strikethrough
        Style::new().fg(Color::new(0xc9, 0xd1, 0xd9)), // 33: spell
    ],
};

/// Render highlighted code to ANSI-colored terminal output.
///
/// # Arguments
///
/// * `w` - Writer to output ANSI text to
/// * `highlighter` - A mutable `Highlighter` instance (reuse for performance)
/// * `config` - Configured `HighlightConfiguration` for the source language
/// * `source` - Source code to highlight
/// * `injection_callback` - Callback for language injection
pub fn render<'a>(
    w: &mut dyn Write,
    highlighter: &'a mut Highlighter,
    config: &'a HighlightConfiguration,
    source: &'a str,
    injection_callback: impl FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
) -> io::Result<()> {
    render_with_theme(
        w,
        highlighter,
        config,
        source,
        &CATPPUCCIN_MOCHA,
        injection_callback,
    )
}

/// Render highlighted code with a custom theme.
///
/// # Arguments
///
/// * `w` - Writer to output ANSI text to
/// * `highlighter` - A mutable `Highlighter` instance (reuse for performance)
/// * `config` - Configured `HighlightConfiguration` for the source language
/// * `source` - Source code to highlight
/// * `theme` - Theme to use for styling
/// * `injection_callback` - Callback for language injection
pub fn render_with_theme<'a>(
    w: &mut dyn Write,
    highlighter: &'a mut Highlighter,
    config: &'a HighlightConfiguration,
    source: &'a str,
    theme: &Theme,
    injection_callback: impl FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
) -> io::Result<()> {
    let highlights = highlighter
        .highlight(config, source.as_bytes(), None, injection_callback)
        .map_err(|e| io::Error::other(e.to_string()))?;

    // Stack to track active styles
    let mut style_stack: Vec<&Style> = Vec::new();

    for event in highlights {
        let event = event.map_err(|e| io::Error::other(e.to_string()))?;
        match event {
            HighlightEvent::Source { start, end } => {
                write!(w, "{}", &source[start..end])?;
            }
            HighlightEvent::HighlightStart(Highlight(i)) => {
                if let Some(style) = theme.style(i) {
                    style.write_start(w)?;
                    style_stack.push(style);
                }
            }
            HighlightEvent::HighlightEnd => {
                if let Some(style) = style_stack.pop() {
                    style.write_end(w)?;
                    // Re-apply parent styles if nested
                    for s in &style_stack {
                        s.write_start(w)?;
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let c = Color::from_hex("#cba6f7").unwrap();
        assert_eq!(c.r, 0xcb);
        assert_eq!(c.g, 0xa6);
        assert_eq!(c.b, 0xf7);

        let c = Color::from_hex("89b4fa").unwrap();
        assert_eq!(c.r, 0x89);
        assert_eq!(c.g, 0xb4);
        assert_eq!(c.b, 0xfa);
    }

    #[test]
    fn test_style_write() {
        let style = Style::new().fg(Color::new(255, 0, 128)).bold();
        let mut out = Vec::new();
        style.write_start(&mut out).unwrap();
        assert_eq!(String::from_utf8_lossy(&out), "\x1b[1;38;2;255;0;128m");

        out.clear();
        style.write_end(&mut out).unwrap();
        assert_eq!(String::from_utf8_lossy(&out), "\x1b[0m");
    }
}
