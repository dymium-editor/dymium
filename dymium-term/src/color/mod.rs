//! Terminal colors
//!
//! This module is extracted out from [`style`](crate::style) because it is complex enough to
//! warrant a separate place to gather all that complexity together.

use std::str::FromStr;
use thiserror::Error;

mod css_names;
mod vim_names;

/// Representation of a color that can be displayed in the terminal
///
/// Typical users should not directly construct this value. It's expected that you will instead
/// make use of the provided methods (e.g., [`green`], [`fixed`], or [`rgb`]) or trait
/// implementations (e.g., `FromStr`).
///
/// [`green`]: Self::green
/// [`fixed`]: Self::fixed
/// [`rgb`]: Self::rgb
#[derive(Debug, Copy, Clone)]
pub enum Color {
    /// A 3-4-bit or 8-bit color
    ///
    /// Because 8-bit terminal colors use the first 16 values to represent the *original* 8 -- and
    /// then 16 -- colors (i.e., the ones that use `ESC[<N>m`), we can represent them here inline,
    /// with output using the more compact version.
    ///
    /// ## Output
    ///
    /// The rest of the 256 colors are emitted as `ESC[38:5:<N>m` (foreground) and `ESC[48:5:<N>m`
    /// (background), while the first 16 are `ESC[<N: 30-37, 90-97>m` (foreground) and
    /// `ESC[<N: 40-47, 100-107>m`(background)
    ///
    /// ## Parsing
    ///
    /// Some fixed colors are named, like "green" or "bright yellow". These can be provided in any
    /// mix of upper and lower case.
    ///
    /// The rest of the 256 colors can be provided with `@<N>` (e.g., `@171`).
    Fixed(u8),

    /// 24-bit colors specified by separate red, green, and blue values
    ///
    /// ## Output
    ///
    /// These colors are emitted as `ESC[38;2;<R>;<G>;<B>m` (foreground) and
    /// `ESC[48;2;<R>;<G>;<B>m` (background).
    ///
    /// ## Parsing
    ///
    /// RGB colors are provided with hex color strings, like `#bade1f` or `#8b4ca1`. The hex digits
    /// 'a' through 'f' may be provided in any mix of upper and lower case.
    ///
    /// Also, color names from both CSS and Vim can be used as `css:<NAME>` and `vim:<NAME>`. The
    /// definitions for these colors are reproduced locally as `css_names` and `vim_names`
    /// respectively.
    Rgb(u8, u8, u8),
}

impl Color {
    /// Produces a `Color` with the fixed, 8-bit value
    ///
    /// **Note:** If the value is less than 16, it will be reinterpreted on output as the
    /// equivalent 4-bit named color.
    ///
    /// For a color chart, see: <https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg>
    pub fn fixed(value: u8) -> Self {
        Self::Fixed(value)
    }

    /// Produces a `Color` where the values of the red, green, and blue channels have been
    /// explicitly provided
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::Rgb(red, green, blue)
    }
}

macro_rules! named_color_methods {
    (
        $(
        $(#[$attrs:meta])*
        pub fn $method_name:ident($name:literal) => $value:expr;
        )*
    ) => {
        $(
        #[doc = concat!("Produces the named color '", $name, "'")]
        #[doc = ""]
        $(#[$attrs])*
        #[doc = ""]
        #[doc = "For more information, see: <https://en.wikipedia.org/wiki/ANSI_escape_code#3-bit_and_4-bit>"]
        pub fn $method_name() -> Self {
            Self::Fixed($value)
        }
        )*
    };
}

impl Color {
    named_color_methods! {
        pub fn black("Black") => 0;
        pub fn red("Red") => 1;
        pub fn green("Green") => 2;
        pub fn yellow("Yellow") => 3;
        pub fn blue("Blue") => 4;
        pub fn magenta("Magenta") => 5;
        pub fn cyan("Cyan") => 6;

        /// Typically "white" is not the color with all RBG channels at their maximum value
        /// (i.e. `#FFFFFF`). That value is more often ["bright white"](Self::bright_white).
        pub fn white("White") => 7;

        /// The name "Bright Black" comes from the rest of the "Bright" colors, but this color
        /// really is just gray. The [`gray`](Self::gray) method is provided as well, and produces
        /// the same value as this one.
        pub fn bright_black("Bright Black") => 8;
        /// This method is an alias for [`bright_black`](Self::bright_black) because its name can
        /// be a little confusing.
        pub fn gray("Gray") => 8;

        pub fn bright_red("Bright Red") => 9;
        pub fn bright_green("Bright Green") => 10;
        pub fn bright_yellow("Bright Yellow") => 11;
        pub fn bright_blue("Bright Blue") => 12;
        pub fn bright_magenta("Bright Magenta") => 13;
        pub fn bright_cyan("Bright Cyan") => 14;
        pub fn bright_white("Bright White") => 15;
    }
}

/// Error resulting from failing to parse a [`Color`]
///
/// For information on accepted formats, refer to the documentation on [`Color`] itself.
#[derive(Debug, Error)]
pub enum ColorParseError {
    /// The provided string was not ASCII
    #[error("Colors cannot have non-ASCII characters")]
    MustBeAscii,
    /// A hex literal had non-hex characters
    #[error("Hex color literal must only have hexadecimal characters")]
    HexLiteralNotHex,
    /// A hex literal was given, but it had the wrong length
    ///
    /// This can also occur for strings like `#F3A`, which is valid in many other places. For
    /// simplicity, we don't allow it here.
    #[error("Hex color literal must have 6 characters")]
    HexLiteralBadLength,
    /// An 8-bit color number was expected, but something wasn't right (e.g., invalid character,
    /// too big, etc.)
    #[error("Invald 8-bit color number")]
    Invalid8BitNum,
    /// A color namespace (e.g., `css`) that we don't recognize was used
    ///
    /// Currently, the following namespaces are supported:
    ///  * CSS, with `css:<NAME>`
    ///  * Vim, with `vim:<NAME>`
    #[error("Unrecognized color namespace {0:?}")]
    UnrecognizedNamespace(String),
    /// The color name wasn't found in the selected namespace
    ///
    #[error("Color name not found in namespace: no {name:?} in namespace `{namespace}`")]
    NotFoundInNamespace {
        /// The namespace -- currently either `css` or `vim`
        namespace: &'static str,
        /// The name of the color that we couldn't find
        name: String,
    },
    /// Some other error went wrong, and the string wasn't close enough to something we expected
    /// for us to figure out what it was
    #[error("Could not parse color")]
    GeneralFailure,
}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, ColorParseError> {
        if !s.is_ascii() {
            return Err(ColorParseError::MustBeAscii);
        }

        let s = s.to_ascii_lowercase();

        if let Some(s) = s.strip_prefix('#') {
            // parse a hex color literal
            if !s.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(ColorParseError::HexLiteralNotHex);
            } else if s.len() != 6 {
                return Err(ColorParseError::HexLiteralBadLength);
            }

            let hexdigit = |idx: usize| -> u8 {
                match s.as_bytes()[idx] {
                    b @ b'0'..=b'9' => b - b'0',
                    b @ b'a'..=b'f' => b - b'a',
                    _ => unreachable!(),
                }
            };

            let r = (hexdigit(0) << 4) + hexdigit(1);
            let g = (hexdigit(2) << 4) + hexdigit(3);
            let b = (hexdigit(4) << 4) + hexdigit(5);
            Ok(Self::Rgb(r, g, b))
        } else if let Some(s) = s.strip_prefix('@') {
            // parse an 8-bit color value
            match s.parse::<u8>() {
                Ok(n) => Ok(Self::Fixed(n)),
                Err(_) => Err(ColorParseError::Invalid8BitNum),
            }
        } else if let Some(s) = s.strip_prefix("css:") {
            // parse a CSS color name
            match css_names::NAMES.binary_search_by_key(&s, |n| n.name) {
                Ok(i) => {
                    let (r, g, b) = css_names::NAMES[i].rgb;
                    Ok(Self::Rgb(r, g, b))
                }
                Err(_) => Err(ColorParseError::NotFoundInNamespace {
                    namespace: "css",
                    name: s.to_owned(),
                }),
            }
        } else if let Some(s) = s.strip_prefix("vim:") {
            // parse a vim color name
            match vim_names::NAMES.binary_search_by_key(&s, |n| n.name) {
                Ok(i) => {
                    let (r, g, b) = vim_names::NAMES[i].rgb();
                    Ok(Self::Rgb(r, g, b))
                }
                Err(_) => Err(ColorParseError::NotFoundInNamespace {
                    namespace: "vim",
                    name: s.to_owned(),
                }),
            }
        } else {
            // Try to parse a "standard" color name
            match s.as_str() {
                "black" => Ok(Self::Fixed(0)),
                "red" => Ok(Self::Fixed(1)),
                "green" => Ok(Self::Fixed(2)),
                "yellow" => Ok(Self::Fixed(3)),
                "blue" => Ok(Self::Fixed(4)),
                "magenta" => Ok(Self::Fixed(5)),
                "cyan" => Ok(Self::Fixed(6)),
                "white" => Ok(Self::Fixed(7)),
                "bright black" => Ok(Self::Fixed(8)),
                "bright red" => Ok(Self::Fixed(9)),
                "bright green" => Ok(Self::Fixed(10)),
                "bright yellow" => Ok(Self::Fixed(11)),
                "bright blue" => Ok(Self::Fixed(12)),
                "bright magenta" => Ok(Self::Fixed(13)),
                "bright cyan" => Ok(Self::Fixed(14)),
                "bright white" => Ok(Self::Fixed(15)),
                _ => {
                    if let Some((prefix, _)) = s.split_once(':') {
                        Err(ColorParseError::UnrecognizedNamespace(prefix.to_owned()))
                    } else {
                        Err(ColorParseError::GeneralFailure)
                    }
                }
            }
        }
    }
}
