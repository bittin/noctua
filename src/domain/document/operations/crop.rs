// SPDX-License-Identifier: GPL-3.0-or-later
// src/domain/document/operations/crop.rs
//
// Crop operation domain model.

/// Crop region in pixel coordinates.
/// 
/// Pure domain model - represents a rectangular region to crop.
/// No UI concerns, just data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl CropRegion {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn as_tuple(&self) -> (u32, u32, u32, u32) {
        (self.x, self.y, self.width, self.height)
    }

    /// Check if region has valid dimensions.
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}
