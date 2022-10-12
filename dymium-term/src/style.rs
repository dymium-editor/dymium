//! Styling available through ANSI escape codes

use crate::Color;

/// Collection of styling information for terminal-based output
#[derive(Debug, Copy, Clone, Default)]
pub struct Style {
    /// Color of the text, if provided
    pub foreground: Option<Color>,
    /// Background color behind the text, if provided
    pub background: Option<Color>,
    /// Flag indicating whether the text foreground and background colors should be inverted
    pub inverse: bool,
    /// Flag indicating whether the text is bold
    pub bold: bool,
    /// Flag indicating whether the text is faint (i.e., decreased intensity)
    ///
    /// **Note:** This flag *does not* override boldness.
    pub faint: bool,
    /// Flag indicating whether the text is italicized
    pub italic: bool,
    /// Underline style, if the text is underlined
    pub underline: Option<UnderlineStyle>,
    /// Flag indicating whether the text has a strikethrough
    pub strikethrough: bool,
}

/// Styling for an underline
///
/// Most terminal emulators do not support extra styling for underlines, but some do. This is a
/// dedicated separate type in order to allow users with fancier terminals to have some more fun ✨
///
/// By default, underlines have an unspecified color and an [`UnderlineShape::Straight`].
#[derive(Debug, Copy, Clone, Default)]
pub struct UnderlineStyle {
    /// Color of the underline, if specified
    pub color: Option<Color>,
    /// Shape of the underline
    pub style: UnderlineShape,
}

/// The shape of the underline underneath some text
///
/// Most terminal emulators do not support changing the shape of an underline, but some do. By
/// default, styling will fall back to `Straight` if the shape is unsupported.
#[derive(Debug, Copy, Clone, Default)]
pub enum UnderlineShape {
    /// Normal, straight underlines
    ///
    /// This is the default shape, and is what the others fall back to if the terminal does not
    /// support it.
    #[default]
    Straight,
    /// Two straight lines instead of one
    Double,
    /// Wiggly underlines, like those you might see from a spell-checker when it rejects your
    /// writing
    Curly,
    /// Dotted underlining
    Dotted,
    /// Dashed underlining
    Dashed,
}

/// `Style` creation & modification
impl Style {
    /// Returns a `Style` equivalent to the default
    pub const fn new() -> Self {
        Style {
            foreground: None,
            background: None,
            inverse: false,
            bold: false,
            faint: false,
            italic: false,
            underline: None,
            strikethrough: false,
        }
    }

    /// Sets the foreground color of the text, or removes it if the color is `None`
    pub const fn foreground(self, color: Option<Color>) -> Self {
        Style { foreground: color, ..self }
    }

    /// Sets the background color of the text, or removes it if the color is `None`
    pub const fn background(self, color: Option<Color>) -> Self {
        Style { background: color, ..self }
    }

    /// Sets whether the text is inverted -- i.e., the foreground and background colors are
    /// switched
    pub const fn inverse(self, enabled: bool) -> Self {
        Style { inverse: enabled, ..self }
    }

    /// Sets whether the text is bold
    pub const fn bold(self, enabled: bool) -> Self {
        Style { bold: enabled, ..self }
    }

    /// Sets whether the text is faint (i.e., decreased intensity)
    ///
    /// **Note:** This flag *does not* override boldness.
    pub const fn faint(self, enabled: bool) -> Self {
        Style { faint: enabled, ..self }
    }

    /// Sets whether the text is italicized
    pub const fn italic(self, enabled: bool) -> Self {
        Style { italic: enabled, ..self }
    }

    /// Sets the underlining style of the text, or removes it if the style is `None`
    pub const fn underline(self, style: Option<UnderlineStyle>) -> Self {
        Style { underline: style, ..self }
    }

    /// Sets whether the text has a strikethrough
    pub const fn strikethrough(self, enabled: bool) -> Self {
        Style { strikethrough: enabled, ..self }
    }
}
