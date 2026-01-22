// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/view/crop/mod.rs
//
// Crop selection module: overlay widget and selection state.
// Inspired by cosmic-viewer (https://codeberg.org/bhh by Bryan Hyland

mod selection;
mod overlay;

pub use selection::{CropSelection, DragHandle};
pub use overlay::crop_overlay;
