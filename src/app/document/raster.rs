// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/document/raster.rs

use std::path::Path;

use image::{imageops, DynamicImage, GenericImageView, ImageReader};

use super::ImageHandle;

/// Represents a raster image document (PNG, JPEG, WebP, ...).
pub struct RasterDocument {
    /// The decoded image document.
    document: DynamicImage,
    /// Cached handle for rendering.
    pub handle: ImageHandle,
}

impl RasterDocument {
    /// Load a raster document from disk.
    pub fn open(path: &Path) -> image::ImageResult<Self> {
        let document = ImageReader::open(path)?.decode()?;
        let handle = super::create_image_handle(&document);

        Ok(Self { document, handle })
    }

    /// Rebuild the handle after mutating `document`.
    pub fn refresh_handle(&mut self) {
        self.handle = super::create_image_handle(&self.document);
    }

    /// Returns the native pixel dimensions (width, height).
    pub fn dimensions(&self) -> (u32, u32) {
        self.document.dimensions()
    }

    /// Save the current document to disk.
    pub fn save(&self, path: &Path) -> image::ImageResult<()> {
        self.document.save(path)
    }

    /// Extract metadata for this raster document.
    pub fn extract_meta(&self, path: &Path) -> super::meta::DocumentMeta {
        let (width, height) = self.dimensions();
        super::meta::build_raster_meta(path, &self.document, width, height)
    }

    /// Rotate 90 degrees clockwise.
    pub fn rotate_cw(&mut self) {
        self.document = DynamicImage::ImageRgba8(imageops::rotate90(&self.document));
        self.refresh_handle();
    }

    /// Rotate 90 degrees counter-clockwise.
    pub fn rotate_ccw(&mut self) {
        self.document = DynamicImage::ImageRgba8(imageops::rotate270(&self.document));
        self.refresh_handle();
    }

    /// Flip horizontally.
    pub fn flip_horizontal(&mut self) {
        self.document = DynamicImage::ImageRgba8(imageops::flip_horizontal(&self.document));
        self.refresh_handle();
    }

    /// Flip vertically.
    pub fn flip_vertical(&mut self) {
        self.document = DynamicImage::ImageRgba8(imageops::flip_vertical(&self.document));
        self.refresh_handle();
    }
}
