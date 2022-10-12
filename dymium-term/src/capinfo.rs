//! Helper types for parsing terminal capabilities

use serde::de::{Deserializer, Error};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};
use thiserror::Error;

/// Capabilities for a set of terminal emulators or similar programs
///
/// The `TermCapSet` is typically parsed from a single YAML file describing all of the terminals.
pub struct TermCapSet {
    terminals: BTreeMap<String, LabelledTermCap>,
}

/// Collected [`TermCap`]s grouped by value of `$TERM` that they set
///
/// The `GroupedTermCaps` is created by the [`group_by_env_var`] method on [`TermCapSet`].
///
/// [`group_by_env_var`]: TermCapSet::group_by_env_var
pub struct GroupedTermCaps {
    by_name: BTreeMap<String, Arc<LabelledTermCap>>,
    by_term_var: BTreeMap<String, TermCapGroup>,
}

/// Grouped minimum [`TermCap`] for a set of terminals that all use the same `$TERM` value
pub struct TermCapGroup {
    min_caps: TermCap,
    members: BTreeMap<String, Arc<LabelledTermCap>>,
}

/// A [`TermCap`] with an associated [`TerminalName`]
#[derive(Debug, Clone, Deserialize)]
pub struct LabelledTermCap {
    /// Name of the terminal
    pub name: TerminalName,
    /// Capabilities associated with the terminal
    #[serde(flatten)]
    pub caps: TermCap,
}

/// Name of the terminal emulator or similar program
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalName {
    /// A name for use within code
    ///
    /// The name must consist only of alphanumerics, hyphens, or underscores.
    #[serde(deserialize_with = "deserialize_compact_name")]
    pub compact: String,
    /// The name of the emulator or otherwise, as a human-readable name
    pub pretty: String,
    /// The value of the `$TERM` environment variable used by this terminal
    ///
    /// If multiple described terminals use the same environment variable (e.g., with `libvte`
    /// using `xterm-256color`), then we limit the capabilities to the minimum set implemented by
    /// all of the terminals we know about.
    pub term: String,
}

/// Capabilities of a terminal emulator or similar program
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TermCap {
    /// Capabilities for styling text
    pub style: StyleCap,
    /// Capabilities for interacting with the cursor
    pub cursor: CursorCap,
    /// Capabilities for scrolling content on the screen
    pub scroll: ScrollCap,
}

// helper function to deserialize "compact" terminal names -- disallowing certain characters
fn deserialize_compact_name<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let s = String::deserialize(deserializer)?;
    let is_ok = !s.is_empty()
        && s.bytes().all(|b| match b {
            _ if !b.is_ascii_graphic() => false,
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => true,
            b'-' | b'_' => true,
            _ => false,
        });

    if !is_ok {
        let msg = "Compact name must consist of alphanumerics, hyhpens, or underscores";
        return Err(D::Error::custom(msg));
    }

    Ok(s)
}

/// Capabilities for styling text
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleCap {
    /// Reset all styling: `true` if enabled, `false` if disabled
    ///
    /// Pretty much every terminal has this capability. However, if a terminal is *not* marked as
    /// having this, certain operations may assume that because there is no way to reset the
    /// styling, none should ever be used.
    ///
    /// *Standard*: VT100 <br>
    /// *Escape Sequence*: `ESC[0m`
    #[serde(alias = "resetAll")]
    #[serde(alias = "reset-all")]
    pub reset_all: bool,

    /// Text coloring capabilities
    #[serde(alias = "setColor")]
    #[serde(alias = "set-color")]
    pub set_color: ColorCap,
    /// Capabilities for resetting foreground or background colors: `true` if enabled, `false` if
    /// disabled
    ///
    /// *Standard*: ECMA-48 3rd <br>
    /// *Escape Sequence*: `ESC[39m` (foreground), `ESC[49m` (background)
    #[serde(alias = "unsetColor")]
    #[serde(alias = "unset-color")]
    pub unset_color: bool,

    /// Inverse capabilities: `true` if enabled, `false` if disabled
    ///
    /// *Standard*: VT100 <br>
    /// *Escape Sequence*: `ESC[7m`
    #[serde(alias = "setInverse")]
    #[serde(alias = "set-inverse")]
    pub set_inverse: bool,
    /// Resetting inversion capabilities: `true` if enabled, `false` if disabled
    ///
    /// *Standard*: ECMA-48 3rd <br>
    /// *Escape Sequence*: `ESC[27m`
    #[serde(alias = "unsetInverse")]
    #[serde(alias = "unset-inverse")]
    pub unset_inverse: bool,

    /// Italics capabilities: `true` if enabled, `false` if disabled
    ///
    /// *Standard*: ECMA-48 2nd
    /// *Escape Sequence*: `ESC[3m`
    #[serde(alias = "setItalics")]
    #[serde(alias = "set-italics")]
    pub set_italics: bool,
    /// Resetting *just* italics: `true` if enabled, `false` if disabled
    ///
    /// *Standard*: ECMA-48 3rd
    /// *Escape Sequence*: `ESC[23m`
    #[serde(alias = "unsetItalics")]
    #[serde(alias = "unset-italics")]
    pub unset_italics: bool,

    /// Bold text capabilities: `true` if enabled, `false` if disabled
    ///
    /// *Standard*: VT100
    /// *Escape Sequence*: `ESC[1m`
    #[serde(alias = "setBold")]
    #[serde(alias = "set-bold")]
    pub set_bold: bool,
    /// Faint text capabilities: `true` if enabled, `false` if disabled
    ///
    /// *Standard*: ECMA-48 2nd
    /// *Escape Sequence*: `ESC[2m`
    #[serde(alias = "setFaint")]
    #[serde(alias = "set-faint")]
    pub set_faint: bool,
    /// Resetting bold and faint: `true` if enabled, `false`, if disabled
    ///
    /// *Standard*: ECMA-48 3rd <br>
    /// *Escape Sequence*: `ESC[22m`
    #[serde(alias = "unsetBoldFaint")]
    #[serde(alias = "unset-bold-faint")]
    pub unset_bold_faint: bool,

    /// Underlining capabilities
    #[serde(alias = "setUnderline")]
    #[serde(alias = "set-underline")]
    pub set_underline: UnderlineCap,
    /// Resetting underline (i.e. back to nothing): `true` if enabled, `false` if disabled
    ///
    /// *Standard*: ECMA-48 3rd <br>
    /// *Escape Sequence*: `ESC[24m`
    #[serde(alias = "unsetUnderline")]
    #[serde(alias = "unset-underline")]
    pub unset_underline: bool,
}

/// Capabilities for displaying colors
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ColorCap {
    /// The terminal cannot display colors
    #[serde(alias = "none")]
    None,
    /// The terminal only has support for 4-bit colors, like `ESC[31m` (red foreground) or
    /// `ESC[103m` (bright yellow background)
    ///
    /// *Standard*: Unknown (VT100? This is hard to find!) <br>
    /// *Escape Sequence*: `ESC[<N>m` with N in `30..=37` or `90..=97` (foreground) and `40..=47`
    /// or `100..=107` (background)
    #[serde(alias = "fixed4bit")]
    #[serde(alias = "fixed-4bit")]
    Fixed4Bit,
    /// The terminal only has support for 8-bit colors (aka "256 color")
    ///
    /// This means the terminal works with any [`Color::Fixed`](crate::Color::Fixed).
    ///
    /// Some terminals historically used 88-color schemes, but to our knowledge those are no longer
    /// in use; if you have a need to record their capabilities here, they can just use
    /// `Fixed4Bit`.
    ///
    /// *Standard*: aixterm <br>
    /// *Escape Sequence*: `ESC[<N>m` with N in `90..=97` (foreground) and `100..=107` (background)
    #[serde(alias = "fixed8bit")]
    #[serde(alias = "fixed-8bit")]
    Fixed8Bit,
    /// The terminal supports 8-bit colors and 24-bit full RGB selection
    ///
    /// This means the terminal works with any [`Color::Rgb`] *or* [`Color::Fixed`]
    ///
    /// [`Color::Fixed`]: crate::Color::Fixed
    /// [`Color::Rgb`]: crate::Color::Rgb
    #[serde(alias = "rgb")]
    #[serde(alias = "RGB")]
    Rgb(RgbCapSet),
}

/// The set of RGB color capabilities for a terminal
///
/// It is possible for none of the fields to equal `true`; in this case, the capabilities from the
/// containing [`ColorCap`] should be assumed to be limited to `Fixed8Bit`.
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RgbCapSet {
    /// Xterm-style RGB colors
    ///
    /// Xterm actually supports both this style and Konsole's, but it is distinct in supporting
    /// this style of RGB colors.
    ///
    /// *Standard*: Xterm
    /// *Escape Sequence*: `ESC[38:2:<I>:<R>:<G>:<B>m` (foreground) and `ESC[38:2:<I>:<R>:<G>:<B>m`
    /// (background).
    ///
    /// **Note**: the color space identifier `I` is ignored. If `konsole` is available, it should
    /// be used instead of this format.
    #[serde(alias = "Xterm")]
    pub xterm: bool,
    /// Konsole-style RGB colors
    ///
    /// This is the style used essentially by all modern terminal emulators (including Xterm), but
    /// Konsole introduced it and so that is the name.
    ///
    /// *Standard*: Konsole (ish)
    /// *Escape Sequence*: `ESC[38;2;<R>;<G>;<B>m` (foreground) and `ESC[48;2;<R>;<G>;<B>m`
    /// (background)
    #[serde(alias = "Konsole")]
    pub konsole: bool,
}

/// Capabilities for underlining text
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum UnderlineCap {
    /// The terminal cannot underline text
    #[serde(alias = "none")]
    None,
    /// The terminal supports basic, un-styled underlining of text
    ///
    /// *Standard*: VT100
    /// *Escape Sequence*: `ESC[4m`
    #[serde(alias = "basic")]
    Basic,
    /// The terminal supports some level of underline styling beyond basic underlining
    #[serde(alias = "fancy")]
    Fancy(FancyUnderlineCap),
}

/// Capabilities for styling underlines
///
/// All fields mark the capability as enabled if `true` and disabled if `false`.
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FancyUnderlineCap {
    /// Double-underline style capabilities
    ///
    /// *Standard*: ECMA-48 3rd
    /// *Escape Sequence*: `ESC[21m`
    pub double: bool,
    /// Kitty-style underline capabilities
    ///
    /// This includes a number of additional escape sequences:
    ///
    /// | Function | Escape sequence |
    /// |----------|-----------------|
    /// | Unset underline | `ESC[4:0m` |
    /// | Straight underline | `ESC[4:1m` |
    /// | Double underline | `ESC[4:2m` |
    /// | Curly underline | `ESC[4:3m` |
    /// | Dotted underline | `ESC[4:4m` |
    /// | Dashed underline | `ESC[4:5m` |
    /// | Underline color | `ESC[58;5;<N>m` or `ESC[58;2;<R>;<G>;<B>m` |
    /// | Reset underline color | `ESC[59m` |
    #[serde(alias = "Kitty")]
    pub kitty: bool,
}

/// Capabilities for interacting with the cursor
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CursorCap {
    /// Basic directional cursor movement capabilities
    ///
    /// This covers a number of escape sequences:
    ///
    /// * `ESC[<N?>A` -- Cursor up `N` times (default 1)
    /// * `ESC[<N?>B` -- Cursor down `N` times (default 1)
    /// * `ESC[<N?>C` -- Cursor forward `N` times (default 1)
    /// * `ESC[<N?>D` -- Cursor down `N` times (default 1)
    /// * `ESC[<N?>E` -- Cursor down `N` times and to first column (default 1)
    /// * `ESC[<N?>F` -- Cursor up `N` times and to first column (default 1)
    /// * `ESC[<N?>G` -- Cursor to column `N` (default 1)
    /// * `ESC[<R?>;<C?>H` -- Cursor to position (`row;column`, default `1;1`)
    ///
    /// *Standard*: ECMA-48
    #[serde(alias = "basicMovement")]
    #[serde(alias = "basic-movement")]
    basic_movement: bool,

    /// Capabilities for setting the cursor's style
    #[serde(alias = "setStyle")]
    #[serde(alias = "set-style")]
    set_style: CursorStyleCap,

    /// Capabilities for saving and restoring the cursor position
    ///
    /// *Standard*: ECMA-48
    /// *Escape Sequence*: `ESC[s` (save) and `ESC[u` (restore)
    #[serde(alias = "saveAndRestore")]
    #[serde(alias = "save-and-restore")]
    save_and_restore: bool,
}

/// Capabilities for setting the cursor's style
///
/// Some terminals have this conditionally enabled, depending on user configuration. This is just
/// cosmetic, so we're ok with partial support here.
///
/// **Note**: Throughout the escape sequences for this type, we reference `<SP>`, which is just an
/// unambiguous way of referring to the space character (hex value 0x20).
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CursorStyleCap {
    /// Capabilities for VT520-style cursor style setting
    ///
    /// *Standard*: VT520
    /// *Escape Sequence*: `ESC[<N?><SP>q` where `N` is one of: `0`/`1` (blink block), `2` (steady
    /// block), `3` (blink underline), or `4` (steady underline). The default is `1` (blink block).
    basic: bool,
    /// Xterm-extended cursor style settings
    ///
    /// *Standard*: Xterm
    /// *Escape Sequence*: `ESC[<N><SP>q` where `N` is either `5` (blink bar) or `6` (steady bar)
    #[serde(alias = "xterm-extended")]
    #[serde(alias = "xtermExtended")]
    xterm_extended: bool,
}

/// Capabilities for scrolling the terminal
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScrollCap {
    /// Basic scrolling capabilities (can it scroll the screen at all?)
    ///
    /// *Standard*: ECMA
    /// *Escape Sequence*: `ESC[<N?>S` (scroll up, default: 1) and `ESC[<N?>^` (scroll down,
    /// default: 1)
    basic: bool,

    /// Capabilities for setting a scroll region
    ///
    /// *Standard*: VT100
    /// *Escape Sequence*: `ESC[<Top?>;<Bot?>r` (default: full size of window)
    #[serde(alias = "set-region")]
    #[serde(alias = "setRegion")]
    set_region: bool,
}

/// Error occuring from loading a [`TermCapSet`]
#[derive(Debug, Error)]
pub enum LoadTermCapsError {
    /// An error from failing to read the file
    #[error(transparent)]
    Io(#[from] io::Error),
    /// An error from failing to parse the content of the file
    #[error("Failed to parse YAML")]
    Yaml(
        #[source]
        #[from]
        serde_yaml::Error,
    ),
    /// An error from duplicated terminal names in the [`TermCapSet`]
    ///
    /// The inner `String` contains the formatted error message.
    #[error("{0}")]
    DuplicateNames(String),
}

impl TermCapSet {
    /// Loads the `TermCapSet` from the file
    pub fn load_all_from_file(path: &Path) -> Result<Self, LoadTermCapsError> {
        use std::collections::btree_map::Entry;

        let content = fs::read(path)?;
        let vec: Vec<LabelledTermCap> = serde_yaml::from_slice(&content)?;

        let mut terminals = BTreeMap::new();
        let mut duplicates = Vec::new();
        for labelled_cap in vec {
            let name = labelled_cap.name.compact.clone();
            match terminals.entry(name) {
                Entry::Occupied(_) => duplicates.push(labelled_cap.name.compact.clone()),
                Entry::Vacant(e) => drop(e.insert(labelled_cap)),
            }
        }

        match duplicates.len() {
            0 => Ok(TermCapSet { terminals }),
            1 => Err(LoadTermCapsError::DuplicateNames(format!(
                "Duplicated terminal name: {:?}",
                &duplicates[0]
            ))),
            len => {
                let mut msg = "Duplicated terminal names: ".to_owned();
                for (i, s) in duplicates.iter().enumerate() {
                    if i == len - 1 {
                        msg.push_str(", and ");
                    } else if i != 0 {
                        msg.push_str(", ");
                    }

                    msg.push_str(s);
                }

                Err(LoadTermCapsError::DuplicateNames(msg))
            }
        }
    }

    /// Groups the `TermCap`s by the value of `$TERM` that they use, producing a mapping with the
    /// minimum capabilities indicated by values of the `$TERM` variable
    pub fn group_by_env_var(self) -> GroupedTermCaps {
        use std::collections::btree_map::Entry;

        let by_name = self
            .terminals
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<BTreeMap<_, _>>();
        let mut by_term_var = BTreeMap::new();

        for (k, cap) in by_name.iter() {
            match by_term_var.entry(cap.name.term.clone()) {
                Entry::Vacant(e) => {
                    let min_caps = cap.caps.clone();
                    let mut components = BTreeMap::new();
                    components.insert(k.clone(), Arc::clone(cap));
                    let _ = e.insert(TermCapGroup { min_caps, members: components });
                }
                Entry::Occupied(mut e) => {
                    let group = e.get_mut();
                    group.min_caps = group.min_caps.min(cap.caps);
                    group.members.insert(k.clone(), Arc::clone(cap));
                }
            }
        }

        GroupedTermCaps { by_name, by_term_var }
    }
}

impl GroupedTermCaps {
    /// Returns information about the set of terminals that use the provided environment variable,
    /// if there are any
    ///
    /// This method will typically be used when getting information about the current terminal,
    /// given by the environment.
    pub fn get(&self, term_env_var: &str) -> Option<&TermCapGroup> {
        self.by_term_var.get(term_env_var)
    }

    /// Returns the information about the terminal with the given "compact" name
    ///
    /// This method will typically be used when overriding the terminal in use.
    pub fn get_by_name(&self, compact_name: &str) -> Option<&Arc<LabelledTermCap>> {
        self.by_name.get(compact_name)
    }

    /// Produces an iterator over all recognized values of the `$TERM` environment variable
    pub fn env_vars(&self) -> impl Iterator<Item = &str> {
        self.by_term_var.keys().map(|string| string.as_str())
    }

    /// Produces an iterator over all recognized terminals
    pub fn terminals(&self) -> impl Iterator<Item = &Arc<LabelledTermCap>> {
        self.by_name.values()
    }
}

impl TermCapGroup {
    /// Minimum capability set among terminals with this `$TERM` value
    pub fn min_caps(&self) -> &TermCap {
        &self.min_caps
    }

    /// Produces an iterator over the terminals with this `$TERM` value
    pub fn members(&self) -> impl Iterator<Item = &TerminalName> {
        self.members.values().map(|labelled| &labelled.name)
    }
}

impl TermCap {
    /// Produces the `TermCap` corresponding to the minimum shared set of capabilities
    fn min(self, other: Self) -> Self {
        TermCap {
            style: self.style.min(other.style),
            cursor: self.cursor.min(other.cursor),
            scroll: self.scroll.min(other.scroll),
        }
    }
}

impl StyleCap {
    fn min(self, other: Self) -> Self {
        StyleCap {
            reset_all: self.reset_all && other.reset_all,
            set_color: self.set_color.min(other.set_color),
            unset_color: self.unset_color && other.unset_color,
            set_inverse: self.set_inverse && other.set_inverse,
            unset_inverse: self.unset_inverse && other.unset_inverse,
            set_italics: self.set_italics && other.set_italics,
            unset_italics: self.unset_italics && other.unset_italics,
            set_bold: self.set_bold && other.set_bold,
            set_faint: self.set_faint && other.set_faint,
            unset_bold_faint: self.unset_bold_faint && other.unset_bold_faint,
            set_underline: self.set_underline.min(other.set_underline),
            unset_underline: self.unset_underline && other.unset_underline,
        }
    }
}

impl ColorCap {
    fn min(self, other: Self) -> Self {
        match (self, other) {
            (ColorCap::None, _) | (_, ColorCap::None) => ColorCap::None,
            (ColorCap::Fixed4Bit, _) | (_, ColorCap::Fixed4Bit) => ColorCap::Fixed4Bit,
            (ColorCap::Fixed8Bit, _) | (_, ColorCap::Fixed8Bit) => ColorCap::Fixed8Bit,
            (ColorCap::Rgb(this), ColorCap::Rgb(that)) => ColorCap::Rgb(this.min(that)),
        }
    }
}

impl RgbCapSet {
    fn min(self, other: Self) -> Self {
        RgbCapSet {
            xterm: self.xterm && other.xterm,
            konsole: self.konsole && other.konsole,
        }
    }
}

impl UnderlineCap {
    fn min(self, other: Self) -> Self {
        match (self, other) {
            (UnderlineCap::None, _) | (_, UnderlineCap::None) => UnderlineCap::None,
            (UnderlineCap::Basic, _) | (_, UnderlineCap::Basic) => UnderlineCap::Basic,
            (UnderlineCap::Fancy(this), UnderlineCap::Fancy(that)) => {
                UnderlineCap::Fancy(this.min(that))
            }
        }
    }
}

impl FancyUnderlineCap {
    fn min(self, other: Self) -> Self {
        FancyUnderlineCap {
            double: self.double && other.double,
            kitty: self.kitty && other.kitty,
        }
    }
}

impl CursorCap {
    fn min(self, other: Self) -> Self {
        CursorCap {
            basic_movement: self.basic_movement && other.basic_movement,
            set_style: self.set_style.min(other.set_style),
            save_and_restore: self.save_and_restore && other.save_and_restore,
        }
    }
}

impl CursorStyleCap {
    fn min(self, other: Self) -> Self {
        CursorStyleCap {
            basic: self.basic && other.basic,
            xterm_extended: self.xterm_extended && other.xterm_extended,
        }
    }
}

impl ScrollCap {
    fn min(self, other: Self) -> Self {
        ScrollCap {
            basic: self.basic && other.basic,
            set_region: self.set_region && other.set_region,
        }
    }
}
