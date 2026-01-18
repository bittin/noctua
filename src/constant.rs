// SPDX-License-Identifier: GPL-3.0-or-later
// src/constant.rs
//
// Application constants that should not be changed by the user.

/// Rotation step in degrees (90 = quarter turn).
pub const ROTATION_STEP: i16 = 90;

/// Full rotation in degrees (for modulo calculation in angle normalization).
pub const FULL_ROTATION: i16 = 360;

/// Minutes per degree (GPS coordinate conversion: DMS to decimal degrees).
pub const MINUTES_PER_DEGREE: f64 = 60.0;

/// Seconds per degree (GPS coordinate conversion: DMS to decimal degrees).
pub const SECONDS_PER_DEGREE: f64 = 3600.0;

/// Minimum pixmap size for SVG rendering (prevents 0x0 images).
pub const MIN_PIXMAP_SIZE: u32 = 1;

/// Tolerance for scale comparisons (float precision in zoom synchronization).
pub const SCALE_EPSILON: f32 = 0.0001;

/// Tolerance for offset comparisons (float precision in pan synchronization).
pub const OFFSET_EPSILON: f32 = 0.01;

/// Maximum thumbnail width in pixels (nav bar page thumbnails).
pub const THUMBNAIL_MAX_WIDTH: f32 = 100.0;

/// Thumbnail cache directory name.
pub const CACHE_DIR: &str = "noctua";

/// Thumbnail file extension.
pub const THUMBNAIL_EXT: &str = "png";

/// Default render scale for PDF pages.
pub const PDF_RENDER_SCALE: f64 = 2.0;

/// Thumbnail render scale (smaller for quick rendering).
pub const PDF_THUMBNAIL_SCALE: f64 = 0.25;
