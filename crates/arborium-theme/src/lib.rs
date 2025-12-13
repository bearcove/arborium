//! Theme support for arborium syntax highlighting.
//!
//! This crate provides:
//! - Highlight category definitions (the canonical list of syntax categories)
//! - Capture name to theme slot mapping
//! - Theme parsing from Helix-style TOML files
//! - CSS and ANSI output generation
//! - Built-in themes (catppuccin, dracula, tokyo-night, etc.)
//!
//! # Capture Name Mapping
//!
//! The crate provides a unified system for mapping the many capture names from
//! various sources (nvim-treesitter, helix, etc.) to a small set of theme slots.
//! See [`highlights::capture_to_slot`] and [`highlights::tag_for_capture`] for details.

pub mod highlights;
pub mod theme;

pub use highlights::{
    CAPTURE_NAMES, COUNT, HIGHLIGHTS, HighlightDef, ThemeSlot, capture_to_slot,
    slot_to_highlight_index, tag_for_capture, tag_to_name,
};

pub use theme::{Color, Modifiers, Style, Theme, ThemeError, builtin};
