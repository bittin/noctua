// SPDX-License-Identifier: GPL-3.0-or-later
// src/ui/widgets/mod.rs
//
// Custom widgets module.

pub mod crop_types;
pub mod image_viewer;

pub use crop_types::{CropRegion, CropSelection, DragHandle};
pub use image_viewer::Viewer;
