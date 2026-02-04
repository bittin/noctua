// SPDX-License-Identifier: GPL-3.0-or-later
// src/ui/widgets/mod.rs
//
// Custom widgets module.

pub mod crop_model;
pub mod crop_overlay;
pub mod image_viewer;

// Re-exports for convenience
pub use crop_model::{CropSelection, DragHandle};
pub use crop_overlay::crop_overlay;
pub use image_viewer::Viewer;
