// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/document/mod.rs
//
// Document module root: common enums and type erasure for document kinds.

pub mod cache;
pub mod file;
pub mod meta;
pub mod portable;
pub mod raster;
pub mod utils;
pub mod vector;

use cosmic::iced_renderer::graphics::image::image_rs::ImageFormat as CosmicImageFormat;
use image::GenericImageView;
use std::fmt;
use std::path::Path;

use self::portable::PortableDocument;
use self::raster::RasterDocument;
use self::vector::VectorDocument;

/// Trait for documents that support multiple pages (PDF, multi-page TIFF, etc.).
pub trait MultiPage {
    /// Total number of pages in the document.
    fn page_count(&self) -> u32;

    /// Current page index (0-based).
    fn current_page(&self) -> u32;

    /// Navigate to a specific page.
    fn goto_page(&mut self, page: u32) -> anyhow::Result<()>;

    /// Check if thumbnails are ready for display.
    fn thumbnails_ready(&self) -> bool;

    /// Generate thumbnails (uses disk cache when available).
    fn generate_thumbnails(&mut self);

    /// Get cached thumbnail handle for a specific page.
    fn get_thumbnail(&self, page: u32) -> Option<ImageHandle>;
}

/// Re-export the image handle type for use by submodules.
pub type ImageHandle = cosmic::iced::widget::image::Handle;

/// Create an iced image handle from a DynamicImage.
///
/// This is the central function for converting rendered images to display handles.
/// Used by raster, vector, and portable document types.
pub fn create_image_handle(img: &image::DynamicImage) -> ImageHandle {
    let (w, h) = img.dimensions();
    let rgba = img.to_rgba8();
    let pixels = rgba.into_raw();
    ImageHandle::from_rgba(w, h, pixels)
}

/// High-level classification of documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Raster,
    Vector,
    Portable,
}

/// Unified document type used by the application.
pub enum DocumentContent {
    Raster(RasterDocument),
    Vector(VectorDocument),
    Portable(PortableDocument),
}

impl fmt::Debug for DocumentContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentContent::Raster(_) => f.write_str("DocumentContent::Raster(..)"),
            DocumentContent::Vector(_) => f.write_str("DocumentContent::Vector(..)"),
            DocumentContent::Portable(_) => f.write_str("DocumentContent::Portable(..)"),
        }
    }
}

impl DocumentKind {
    /// Derive document kind from file extension.
    ///
    /// - `pdf`  => Portable
    /// - `svg`  => Vector
    /// - supported image extensions (via libcosmic/image_rs ImageFormat)
    ///   => Raster
    ///
    /// Returns `None` if the extension is not recognized as any supported kind.
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext_os = path.extension()?;
        let ext_str = ext_os.to_str()?;
        let ext_lower = ext_str.to_ascii_lowercase();

        match ext_lower.as_str() {
            "pdf" => return Some(DocumentKind::Portable),
            "svg" => return Some(DocumentKind::Vector),
            _ => {}
        }

        // Ask libcosmic/image_rs if this extension corresponds to a known image
        // format. If yes, we treat it as a raster document.
        if CosmicImageFormat::from_extension(ext_os).is_some() {
            return Some(DocumentKind::Raster);
        }

        None
    }
}

impl DocumentContent {
    /// Returns a cloneable image handle for rendering.
    ///
    /// This is intentionally linear: every concrete document type
    /// owns some kind of `ImageHandle`, and the canvas can
    /// just call `doc.handle()` without additional branching.
    pub fn handle(&self) -> ImageHandle {
        match self {
            DocumentContent::Raster(doc) => doc.handle.clone(),
            DocumentContent::Vector(doc) => doc.handle.clone(),
            DocumentContent::Portable(doc) => doc.handle.clone(),
        }
    }

    /// Returns the native dimensions (width, height) of the document in pixels.
    ///
    /// For raster images this is the actual pixel size.
    /// For vector/portable documents this is the rasterized size at default DPI.
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            DocumentContent::Raster(doc) => doc.dimensions(),
            DocumentContent::Vector(doc) => doc.dimensions(),
            DocumentContent::Portable(doc) => doc.dimensions(),
        }
    }
    /// Extract metadata from the document.
    /// Requires the file path for file size and EXIF extraction.
    pub fn extract_meta(&self, path: &Path) -> meta::DocumentMeta {
        match self {
            DocumentContent::Raster(doc) => doc.extract_meta(path),
            DocumentContent::Vector(doc) => doc.extract_meta(path),
            DocumentContent::Portable(doc) => doc.extract_meta(path),
        }
    }

    /// Rotate document 90 degrees clockwise.
    pub fn rotate_cw(&mut self) {
        match self {
            DocumentContent::Raster(doc) => doc.rotate_cw(),
            DocumentContent::Vector(doc) => doc.rotate_cw(),
            DocumentContent::Portable(doc) => doc.rotate_cw(),
        }
    }

    /// Rotate document 90 degrees counter-clockwise.
    pub fn rotate_ccw(&mut self) {
        match self {
            DocumentContent::Raster(doc) => doc.rotate_ccw(),
            DocumentContent::Vector(doc) => doc.rotate_ccw(),
            DocumentContent::Portable(doc) => doc.rotate_ccw(),
        }
    }

    /// Flip document horizontally.
    pub fn flip_horizontal(&mut self) {
        match self {
            DocumentContent::Raster(doc) => doc.flip_horizontal(),
            DocumentContent::Vector(doc) => doc.flip_horizontal(),
            DocumentContent::Portable(doc) => doc.flip_horizontal(),
        }
    }

    /// Flip document vertically.
    pub fn flip_vertical(&mut self) {
        match self {
            DocumentContent::Raster(doc) => doc.flip_vertical(),
            DocumentContent::Vector(doc) => doc.flip_vertical(),
            DocumentContent::Portable(doc) => doc.flip_vertical(),
        }
    }

    /// Check if this document supports multiple pages.
    pub fn is_multi_page(&self) -> bool {
        match self {
            DocumentContent::Portable(doc) => doc.page_count() > 1,
            // TODO: RasterDocument for multi-page TIFF
            _ => false,
        }
    }

    /// Get page count if this is a multi-page document.
    pub fn page_count(&self) -> Option<u32> {
        match self {
            DocumentContent::Portable(doc) => Some(doc.page_count()),
            // TODO: RasterDocument for multi-page TIFF
            _ => None,
        }
    }

    /// Get current page index if this is a multi-page document.
    pub fn current_page(&self) -> Option<u32> {
        match self {
            DocumentContent::Portable(doc) => Some(doc.current_page()),
            // TODO: RasterDocument for multi-page TIFF
            _ => None,
        }
    }

    /// Navigate to a specific page if this is a multi-page document.
    pub fn goto_page(&mut self, page: u32) -> anyhow::Result<()> {
        match self {
            DocumentContent::Portable(doc) => doc.goto_page(page),
            // TODO: RasterDocument for multi-page TIFF
            _ => Err(anyhow::anyhow!("Document does not support multiple pages")),
        }
    }

    /// Get cached thumbnail handle for a specific page.
    pub fn get_thumbnail(&self, page: u32) -> Option<ImageHandle> {
        match self {
            DocumentContent::Portable(doc) => doc.get_thumbnail(page),
            // TODO: RasterDocument for multi-page TIFF
            _ => None,
        }
    }

    /// Check if thumbnails are ready for display.
    pub fn thumbnails_ready(&self) -> bool {
        match self {
            DocumentContent::Portable(doc) => doc.thumbnails_ready(),
            // TODO: RasterDocument for multi-page TIFF
            _ => false,
        }
    }

    /// Get number of thumbnails currently loaded.
    pub fn thumbnails_loaded(&self) -> u32 {
        match self {
            DocumentContent::Portable(doc) => doc.thumbnails_loaded(),
            // TODO: RasterDocument for multi-page TIFF
            _ => 0,
        }
    }

    /// Generate a single thumbnail page. Returns next page to generate, or None if done.
    pub fn generate_thumbnail_page(&mut self, page: u32) -> Option<u32> {
        match self {
            DocumentContent::Portable(doc) => doc.generate_thumbnail_page(page),
            // TODO: RasterDocument for multi-page TIFF
            _ => None,
        }
    }

    /// Generate all thumbnails at once (blocking).
    pub fn generate_thumbnails(&mut self) {
        match self {
            DocumentContent::Portable(doc) => doc.generate_thumbnails(),
            // TODO: RasterDocument for multi-page TIFF
            _ => {}
        }
    }
}

/// Set an image file as desktop wallpaper.
///
/// Delegates to `utils::set_as_wallpaper` which tries multiple methods.
pub fn set_as_wallpaper(path: &Path) {
    utils::set_as_wallpaper(path);
}
