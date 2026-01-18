// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/document/portable.rs
//
// Portable documents (PDF) with poppler backend.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use cairo::{Context, Format, ImageSurface};
use image::{imageops, DynamicImage, ImageReader};
use poppler::PopplerDocument;

use super::{cache, ImageHandle};
use crate::constant::{FULL_ROTATION, PDF_RENDER_SCALE, PDF_THUMBNAIL_SCALE, ROTATION_STEP};

/// Represents a portable document (PDF).
pub struct PortableDocument {
    /// The parsed PDF document.
    document: PopplerDocument,
    /// Path to the source file (for caching).
    source_path: PathBuf,
    /// Total number of pages.
    page_count: u32,
    /// Current page index (0-based).
    current_page: u32,
    /// Rotation in degrees (0, 90, 180, 270).
    pub rotation: i16,
    /// Current rendered page as image.
    pub rendered: DynamicImage,
    /// Image handle for display.
    pub handle: ImageHandle,
    /// Cached thumbnail handles for each page (None = not yet generated).
    thumbnail_cache: Option<Vec<ImageHandle>>,
}

impl PortableDocument {
    /// Open a PDF document and render the first page.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let document = PopplerDocument::new_from_file(path, None)
            .map_err(|e| anyhow::anyhow!("Failed to parse PDF: {}", e))?;

        let page_count = document.get_n_pages() as u32;
        if page_count == 0 {
            return Err(anyhow::anyhow!("PDF has no pages"));
        }

        let rendered = Self::render_page(&document, 0, 0)?;
        let handle = super::create_image_handle(&rendered);

        Ok(Self {
            document,
            source_path: path.to_path_buf(),
            page_count,
            current_page: 0,
            rotation: 0,
            rendered,
            handle,
            thumbnail_cache: None,
        })
    }

    /// Check if all thumbnails are ready.
    pub fn thumbnails_ready(&self) -> bool {
        self.thumbnail_cache
            .as_ref()
            .map(|c| c.len() as u32 >= self.page_count)
            .unwrap_or(false)
    }

    /// Get the number of thumbnails currently loaded.
    pub fn thumbnails_loaded(&self) -> u32 {
        self.thumbnail_cache
            .as_ref()
            .map(|c| c.len() as u32)
            .unwrap_or(0)
    }

    /// Initialize thumbnail cache (empty, ready for incremental loading).
    pub fn init_thumbnail_cache(&mut self) {
        if self.thumbnail_cache.is_none() {
            self.thumbnail_cache = Some(Vec::with_capacity(self.page_count as usize));
        }
    }

    /// Generate a single thumbnail page. Returns the next page to generate, or None if done.
    pub fn generate_thumbnail_page(&mut self, page: u32) -> Option<u32> {
        // Initialize cache if needed.
        self.init_thumbnail_cache();

        // Check if we should generate this page.
        let should_generate = {
            let cache = self.thumbnail_cache.as_ref()?;
            page as usize >= cache.len() && page < self.page_count
        };

        if should_generate {
            let handle = self.load_or_generate_thumbnail(page);
            if let Some(cache) = self.thumbnail_cache.as_mut() {
                cache.push(handle);
            }
        }

        // Return next page if not done.
        let next = page + 1;
        if next < self.page_count {
            Some(next)
        } else {
            None
        }
    }

    /// Generate all thumbnails at once (legacy, blocking).
    pub fn generate_thumbnails(&mut self) {
        if self.thumbnails_ready() {
            return;
        }
        self.init_thumbnail_cache();
        for page in 0..self.page_count {
            self.generate_thumbnail_page(page);
        }
    }

    /// Load thumbnail from cache or generate and cache it.
    fn load_or_generate_thumbnail(&self, page: u32) -> ImageHandle {
        if let Some(handle) = cache::load_thumbnail(&self.source_path, page) {
            return handle;
        }

        match Self::render_page_at_scale(&self.document, page, 0, PDF_THUMBNAIL_SCALE) {
            Ok(img) => {
                let _ = cache::save_thumbnail(&self.source_path, page, &img);
                super::create_image_handle(&img)
            }
            Err(e) => {
                log::warn!("Failed to generate thumbnail for page {}: {}", page, e);
                ImageHandle::from_rgba(1, 1, vec![0, 0, 0, 0])
            }
        }
    }

    /// Render a specific page from the document to an image.
    fn render_page(
        document: &PopplerDocument,
        page_index: u32,
        rotation: i16,
    ) -> anyhow::Result<DynamicImage> {
        Self::render_page_at_scale(document, page_index, rotation, PDF_RENDER_SCALE)
    }

    /// Render a specific page at a given scale.
    fn render_page_at_scale(
        document: &PopplerDocument,
        page_index: u32,
        rotation: i16,
        scale: f64,
    ) -> anyhow::Result<DynamicImage> {
        let page = document
            .get_page(page_index as usize)
            .ok_or_else(|| anyhow::anyhow!("Failed to get page {}", page_index))?;

        let (page_width, page_height) = page.get_size();

        let (width, height) = if rotation == 90 || rotation == 270 {
            (page_height, page_width)
        } else {
            (page_width, page_height)
        };

        let scaled_width = (width * scale) as i32;
        let scaled_height = (height * scale) as i32;

        let surface = ImageSurface::create(Format::ARgb32, scaled_width, scaled_height)
            .map_err(|e| anyhow::anyhow!("Failed to create Cairo surface: {}", e))?;

        let context = Context::new(&surface)
            .map_err(|e| anyhow::anyhow!("Failed to create Cairo context: {}", e))?;

        // Fill with white background.
        context.set_source_rgb(1.0, 1.0, 1.0);
        let _ = context.paint();

        context.scale(scale, scale);

        if rotation != 0 {
            let center_x = width / 2.0;
            let center_y = height / 2.0;
            context.translate(center_x, center_y);
            context.rotate(f64::from(rotation) * std::f64::consts::PI / 180.0);
            context.translate(-page_width / 2.0, -page_height / 2.0);
        }

        page.render(&context);

        drop(context);
        surface.flush();

        let mut png_data: Vec<u8> = Vec::new();
        surface
            .write_to_png(&mut png_data)
            .map_err(|e| anyhow::anyhow!("Failed to write PNG: {}", e))?;

        let image = ImageReader::new(Cursor::new(png_data))
            .with_guessed_format()
            .map_err(|e| anyhow::anyhow!("Failed to read PNG format: {}", e))?
            .decode()
            .map_err(|e| anyhow::anyhow!("Failed to decode PNG: {}", e))?;

        Ok(image)
    }

    /// Re-render the current page.
    fn rerender(&mut self) {
        match Self::render_page(&self.document, self.current_page, self.rotation) {
            Ok(rendered) => {
                self.rendered = rendered;
                self.refresh_handle();
            }
            Err(e) => {
                log::error!("Failed to render PDF page: {}", e);
            }
        }
    }

    /// Rebuild the handle after mutating `rendered`.
    pub fn refresh_handle(&mut self) {
        self.handle = super::create_image_handle(&self.rendered);
    }

    /// Returns the dimensions of the currently rendered page.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.rendered.width(), self.rendered.height())
    }

    /// Navigate to a specific page.
    pub fn goto_page(&mut self, page: u32) -> anyhow::Result<()> {
        if page >= self.page_count {
            return Err(anyhow::anyhow!(
                "Page {} out of range (0-{})",
                page,
                self.page_count - 1
            ));
        }
        self.current_page = page;
        self.rerender();
        Ok(())
    }

    /// Navigate to the next page.
    pub fn next_page(&mut self) -> bool {
        if self.current_page + 1 < self.page_count {
            self.current_page += 1;
            self.rerender();
            true
        } else {
            false
        }
    }

    /// Navigate to the previous page.
    pub fn prev_page(&mut self) -> bool {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.rerender();
            true
        } else {
            false
        }
    }

    /// Rotate 90 degrees clockwise.
    pub fn rotate_cw(&mut self) {
        self.rotation = (self.rotation + ROTATION_STEP).rem_euclid(FULL_ROTATION);
        self.rerender();
    }

    /// Rotate 90 degrees counter-clockwise.
    pub fn rotate_ccw(&mut self) {
        self.rotation = (self.rotation - ROTATION_STEP).rem_euclid(FULL_ROTATION);
        self.rerender();
    }

    /// Flip horizontally.
    pub fn flip_horizontal(&mut self) {
        self.rendered = DynamicImage::ImageRgba8(imageops::flip_horizontal(&self.rendered));
        self.refresh_handle();
    }

    /// Flip vertically.
    pub fn flip_vertical(&mut self) {
        self.rendered = DynamicImage::ImageRgba8(imageops::flip_vertical(&self.rendered));
        self.refresh_handle();
    }

    /// Extract metadata for this portable document.
    pub fn extract_meta(&self, path: &Path) -> super::meta::DocumentMeta {
        let (width, height) = self.dimensions();
        super::meta::build_portable_meta(path, width, height, self.page_count)
    }

    /// Get total page count.
    pub fn page_count(&self) -> u32 {
        self.page_count
    }

    /// Get current page index (0-based).
    pub fn current_page(&self) -> u32 {
        self.current_page
    }

    /// Get cached thumbnail handle for a specific page.
    /// Returns None if thumbnails not yet generated.
    pub fn get_thumbnail(&self, page: u32) -> Option<ImageHandle> {
        self.thumbnail_cache
            .as_ref()
            .and_then(|cache| cache.get(page as usize).cloned())
    }
}
