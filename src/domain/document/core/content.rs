// SPDX-License-Identifier: GPL-3.0-or-later
// src/domain/document/core/content.rs
//
// Type-erased document content enum.

use std::fmt;
use std::path::Path;

use cosmic::iced_renderer::graphics::image::image_rs::ImageFormat as CosmicImageFormat;
use cosmic::widget::image::Handle as ImageHandle;

use super::document::{
    DocResult, DocumentInfo, FlipDirection, InterpolationQuality, MultiPage, MultiPageThumbnails,
    RenderOutput, Renderable, Rotation, RotationMode, Transformable, TransformState,
};

use crate::domain::document::types::raster::RasterDocument;
#[cfg(feature = "vector")]
use crate::domain::document::types::vector::VectorDocument;
#[cfg(feature = "portable")]
use crate::domain::document::types::portable::PortableDocument;

// ============================================================================
// Document Kind
// ============================================================================

/// Supported document kinds (for format detection).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Raster,
    Vector,
    Portable,
}

impl DocumentKind {
    /// Detect document kind from file path.
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();

        // SVG
        #[cfg(feature = "vector")]
        if ext == "svg" || ext == "svgz" {
            return Some(Self::Vector);
        }

        // PDF
        #[cfg(feature = "portable")]
        if ext == "pdf" {
            return Some(Self::Portable);
        }

        // Raster: Check via cosmic/image-rs
        if CosmicImageFormat::from_path(path).is_ok() {
            return Some(Self::Raster);
        }

        None
    }
}

impl fmt::Display for DocumentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raster => write!(f, "Raster"),
            Self::Vector => write!(f, "Vector"),
            Self::Portable => write!(f, "Portable"),
        }
    }
}

// ============================================================================
// Document Content Enum
// ============================================================================

/// Type-erased document content.
///
/// The application only holds one document at a time, so the size difference
/// between variants is acceptable. Boxing would add unnecessary indirection.
#[allow(clippy::large_enum_variant)]
pub enum DocumentContent {
    Raster(RasterDocument),
    #[cfg(feature = "vector")]
    Vector(VectorDocument),
    #[cfg(feature = "portable")]
    Portable(PortableDocument),
}

impl fmt::Debug for DocumentContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raster(_) => write!(f, "DocumentContent::Raster(...)"),
            #[cfg(feature = "vector")]
            Self::Vector(_) => write!(f, "DocumentContent::Vector(...)"),
            #[cfg(feature = "portable")]
            Self::Portable(_) => write!(f, "DocumentContent::Portable(...)"),
        }
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

impl Renderable for DocumentContent {
    fn render(&mut self, scale: f64) -> DocResult<RenderOutput> {
        match self {
            Self::Raster(doc) => doc.render(scale),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.render(scale),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.render(scale),
        }
    }

    fn info(&self) -> DocumentInfo {
        match self {
            Self::Raster(doc) => doc.info(),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.info(),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.info(),
        }
    }
}

impl Transformable for DocumentContent {
    fn rotate(&mut self, rotation: Rotation) {
        match self {
            Self::Raster(doc) => doc.rotate(rotation),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.rotate(rotation),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.rotate(rotation),
        }
    }

    fn flip(&mut self, direction: FlipDirection) {
        match self {
            Self::Raster(doc) => doc.flip(direction),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.flip(direction),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.flip(direction),
        }
    }

    fn transform_state(&self) -> TransformState {
        match self {
            Self::Raster(doc) => doc.transform_state(),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.transform_state(),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.transform_state(),
        }
    }

    fn rotate_fine(&mut self, angle_degrees: f32) {
        match self {
            Self::Raster(doc) => doc.rotate_fine(angle_degrees),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.rotate_fine(angle_degrees),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.rotate_fine(angle_degrees),
        }
    }

    fn reset_fine_rotation(&mut self) {
        match self {
            Self::Raster(doc) => doc.reset_fine_rotation(),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.reset_fine_rotation(),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.reset_fine_rotation(),
        }
    }

    fn set_interpolation_quality(&mut self, quality: InterpolationQuality) {
        match self {
            Self::Raster(doc) => doc.set_interpolation_quality(quality),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.set_interpolation_quality(quality),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.set_interpolation_quality(quality),
        }
    }
}

// ============================================================================
// Convenience Methods
// ============================================================================

impl DocumentContent {
    /// Rotate document 90 degrees clockwise.
    pub fn rotate_cw(&mut self) {
        let new_rotation_mode = self.transform_state().rotation.rotate_cw();
        match new_rotation_mode {
            RotationMode::Standard(rot) => self.rotate(rot),
            RotationMode::Fine(deg) => {
                let normalized = ((deg / 90.0).round() as i16 * 90) % 360;
                let rot = match normalized {
                    0 => Rotation::None,
                    90 => Rotation::Cw90,
                    180 => Rotation::Cw180,
                    270 => Rotation::Cw270,
                    _ => Rotation::None,
                };
                self.rotate(rot);
            }
        }
    }

    /// Rotate document 90 degrees counter-clockwise.
    pub fn rotate_ccw(&mut self) {
        let new_rotation_mode = self.transform_state().rotation.rotate_ccw();
        match new_rotation_mode {
            RotationMode::Standard(rot) => self.rotate(rot),
            RotationMode::Fine(deg) => {
                let normalized = ((deg / 90.0).round() as i16 * 90 + 360) % 360;
                let rot = match normalized {
                    0 => Rotation::None,
                    90 => Rotation::Cw90,
                    180 => Rotation::Cw180,
                    270 => Rotation::Cw270,
                    _ => Rotation::None,
                };
                self.rotate(rot);
            }
        }
    }

    /// Flip document horizontally.
    pub fn flip_horizontal(&mut self) {
        self.flip(FlipDirection::Horizontal);
    }

    /// Flip document vertically.
    pub fn flip_vertical(&mut self) {
        self.flip(FlipDirection::Vertical);
    }

    /// Get the document kind.
    #[must_use]
    pub fn kind(&self) -> DocumentKind {
        match self {
            Self::Raster(_) => DocumentKind::Raster,
            #[cfg(feature = "vector")]
            Self::Vector(_) => DocumentKind::Vector,
            #[cfg(feature = "portable")]
            Self::Portable(_) => DocumentKind::Portable,
        }
    }

    /// Check if document supports multiple pages.
    #[must_use]
    pub fn is_multi_page(&self) -> bool {
        matches!(self, Self::Portable(_))
    }

    /// Get total page count (returns 1 for single-page documents).
    #[must_use]
    pub fn page_count(&self) -> usize {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.page_count(),
            _ => 1,
        }
    }

    /// Get current page index (0 for single-page documents).
    #[must_use]
    pub fn current_page(&self) -> usize {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.current_page(),
            _ => 0,
        }
    }

    /// Navigate to a specific page (no-op for single-page documents).
    pub fn go_to_page(&mut self, page: usize) -> DocResult<()> {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.go_to_page(page),
            _ => Ok(()),
        }
    }

    /// Get thumbnail for a specific page (mutable access for trait compatibility).
    pub fn get_thumbnail(&mut self, page: usize) -> DocResult<Option<ImageHandle>> {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.get_thumbnail(page),
            _ => Ok(None),
        }
    }

    /// Get thumbnail handle for a specific page (read-only access).
    /// Returns None if the thumbnail hasn't been generated yet.
    #[must_use]
    pub fn get_thumbnail_handle(&self, page: usize) -> Option<ImageHandle> {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.get_thumbnail_handle(page),
            _ => None,
        }
    }

    /// Check if thumbnails are ready to be generated.
    #[must_use]
    pub fn thumbnails_ready(&self) -> bool {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.thumbnails_ready(),
            _ => false,
        }
    }

    /// Get count of thumbnails currently loaded.
    #[must_use]
    pub fn thumbnails_loaded(&self) -> usize {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => PortableDocument::thumbnails_loaded(doc),
            _ => 0,
        }
    }

    /// Check if all thumbnails have been loaded (trait-compliant).
    #[must_use]
    pub fn all_thumbnails_loaded(&self) -> bool {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => MultiPageThumbnails::thumbnails_loaded(doc),
            _ => false,
        }
    }

    /// Generate thumbnail for a specific page.
    pub fn generate_thumbnail_page(&mut self, page: usize) -> DocResult<()> {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => MultiPageThumbnails::generate_thumbnail_page(doc, page),
            _ => Ok(()),
        }
    }

    /// Generate all thumbnails.
    pub fn generate_thumbnails(&mut self) -> DocResult<()> {
        match self {
            #[cfg(feature = "portable")]
            Self::Portable(doc) => MultiPageThumbnails::generate_all_thumbnails(doc),
            _ => Ok(()),
        }
    }

    /// Get the current rendered image handle.
    #[must_use]
    pub fn handle(&self) -> Option<ImageHandle> {
        match self {
            Self::Raster(doc) => Some(doc.handle()),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => Some(doc.handle()),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => Some(doc.handle()),
        }
    }

    /// Get current dimensions after transformations.
    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::Raster(doc) => doc.dimensions(),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.dimensions(),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.dimensions(),
        }
    }

    /// Crop the document (supported for all types - works on rendered output).
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) -> DocResult<()> {
        match self {
            Self::Raster(doc) => doc.crop(x, y, width, height).map_err(|e| anyhow::anyhow!(e)),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.crop(x, y, width, height).map_err(|e| anyhow::anyhow!(e)),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.crop(x, y, width, height).map_err(|e| anyhow::anyhow!(e)),
        }
    }

    /// Extract document metadata (basic info and EXIF if available).
    #[must_use]
    pub fn extract_meta(&self, path: &Path) -> crate::domain::document::core::metadata::DocumentMeta {
        match self {
            Self::Raster(doc) => doc.extract_meta(path),
            #[cfg(feature = "vector")]
            Self::Vector(doc) => doc.extract_meta(path),
            #[cfg(feature = "portable")]
            Self::Portable(doc) => doc.extract_meta(path),
        }
    }
}
