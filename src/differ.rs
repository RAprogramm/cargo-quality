// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Diff generation and display with responsive layout.
//!
//! This module provides professional diff visualization that adapts to terminal
//! width, offering vertical layout for narrow terminals and side-by-side for
//! wider displays.

mod display;
mod generator;
mod types;

pub use display::{show_full, show_interactive, show_summary};
pub use generator::generate_diff;
pub use types::DiffResult;
