//! Terminal interaction library for `dymium`
//!
//! This crate includes a number of things relevant to interacting with terminals, primarily based
//! around ANSI escape codes. However, other capabilities are also implemented here (e.g, enabling
//! "raw" mode, detecting the [Kitty keyboard protocol]).
//!
//! [Kitty keyboard protocol]: https://sw.kovidgoyal.net/kitty/keyboard-protocol/

#![deny(missing_docs, rustdoc::broken_intra_doc_links)]

pub mod capinfo;
mod cmd;
mod color;
mod style;

pub use cmd::{Command, CursorCommand, ScrollCommand};
pub use color::{Color, ColorParseError};
pub use style::{Style, UnderlineShape, UnderlineStyle};
