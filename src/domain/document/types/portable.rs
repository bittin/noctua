// SPDX-License-Identifier: GPL-3.0-or-later
// src/domain/document/types/portable.rs
//
// Portable documents (PDF) with poppler backend.

use std::io::Cursor;
use std::path::{Path, PathBuf};

/// PDF page render quality multiplier (2.0 = double resolution for sharp display).
const PDF_RENDER_QUALITY: f64 = 2.0;

/// PDF thumbnail size multiplier (0.25 = 25% for fast preview generation).
const PDF_THUMBNAIL_SIZE: f64 = 0.25;

use cairo::{Context, Format, ImageSurface};
use image::{DynamicImage, GenericImageView, ImageReader};
use poppler::PopplerDocument;

use cosmic::widget::image::Handle as ImageHandle;

use crate::domain::document::core::document::{
    DocResult, DocumentInfo, FlipDirection, MultiPage, MultiPageThumbnails, Renderable,
    RenderOutput, Rotation, RotationMode, TransformState, Transformable,
};

/// Represents a portable document (PDF).
pub struct PortableDocument {
    /// The parsed PDF document.
    document: PopplerDocument,
    /// Path to the source file (for caching).
    source_path: PathBuf,
    /// Total number of pages.
    num_pages: usize,
    /// Current page index (0-based).
    page_index: usize,
    /// Current transformation state.
    transform: TransformState,
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
            .map_err(|e| anyhow::anyhow!("Failed to parse PDF: {e}"))?;

        let num_pages = document.get_n_pages();
        if num_pages == 0 {
            return Err(anyhow::anyhow!("PDF has no pages"));
        }

        let rendered = Self::render_page(&document, 0, RotationMode::Standard(Rotation::None))?;
        let handle = Self::create_image_handle_from_image(&rendered);

        Ok(Self {
            document,
            source_path: path.to_path_buf(),
            num_pages,
            page_index: 0,
            transform: TransformState::default(),
            rendered,
            handle,
            thumbnail_cache: None,
        })
    }

    /// Returns the current pixel dimensions (width, height).
    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        self.rendered.dimensions()
    }

    /// Get the current image handle.
    #[must_use]
    pub fn handle(&self) -> ImageHandle {
        self.handle.clone()
    }

    /// Get native dimensions of current page.
    #[must_use]
    pub fn native_dimensions(&self) -> (u32, u32) {
        self.rendered.dimensions()
    }

    /// Get the number of thumbnails currently loaded.
    pub fn thumbnails_loaded(&self) -> usize {
        self.thumbnail_cache.as_ref().map_or(0, Vec::len)
    }

    /// Get thumbnail handle for a specific page (read-only access).
    /// Returns None if the thumbnail hasn't been generated yet.
    #[must_use]
    pub fn get_thumbnail_handle(&self, page: usize) -> Option<ImageHandle> {
        self.thumbnail_cache
            .as_ref()
            .and_then(|cache| cache.get(page).cloned())
    }

    // Helper functions

    /// Extract metadata for this portable document.
    pub fn extract_meta(&self, path: &Path) -> crate::domain::document::core::metadata::DocumentMeta {
        use crate::domain::document::core::metadata::{BasicMeta, DocumentMeta};

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_path = path.to_string_lossy().to_string();
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        let (width, height) = self.dimensions();
        let format = format!("PDF ({} pages)", self.num_pages);

        let basic = BasicMeta {
            file_name,
            file_path,
            format,
            width,
            height,
            file_size,
            color_type: "Rendered".to_string(),
        };

        DocumentMeta { basic, exif: None }
    }

    /// Crop the current page to the specified rectangle.
    /// Works on rendered output (raster).
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<(), String> {
        let (img_width, img_height) = self.rendered.dimensions();

        // Validate crop region
        if x >= img_width || y >= img_height {
            return Err(format!(
                "Crop region ({}, {}) is outside rendered bounds ({}, {})",
                x, y, img_width, img_height
            ));
        }

        // Clamp dimensions
        let crop_width = width.min(img_width - x);
        let crop_height = height.min(img_height - y);

        if crop_width == 0 || crop_height == 0 {
            return Err("Crop region has zero width or height".to_string());
        }

        // Crop rendered image
        self.rendered = self.rendered.crop_imm(x, y, crop_width, crop_height);

        // Update handle
        self.handle = Self::create_image_handle_from_image(&self.rendered);

        Ok(())
    }
    fn create_image_handle_from_image(img: &DynamicImage) -> ImageHandle {
        let (width, height) = img.dimensions();
        let pixels = img.to_rgba8().into_raw();
        ImageHandle::from_rgba(width, height, pixels)
    }

    /// Initialize thumbnail cache (empty, ready for incremental loading).
    fn init_thumbnail_cache(&mut self) {
        if self.thumbnail_cache.is_none() {
            self.thumbnail_cache = Some(Vec::with_capacity(self.num_pages));
        }
    }

    /// Generate a single thumbnail page. Returns the next page to generate, or None if done.
    pub fn generate_thumbnail_page(&mut self, page: usize) -> Option<usize> {
        // Initialize cache if needed.
        self.init_thumbnail_cache();

        // Check if we should generate this page.
        let should_generate = {
            let cache = self.thumbnail_cache.as_ref()?;
            page >= cache.len() && page < self.num_pages
        };

        if should_generate {
            let handle = self.load_or_generate_thumbnail(page);
            if let Some(cache) = self.thumbnail_cache.as_mut() {
                cache.push(handle);
            }
        }

        // Return next page if not done.
        let next = page + 1;
        if next < self.num_pages {
            Some(next)
        } else {
            None
        }
    }

    /// Load thumbnail from cache or generate and cache it.
    fn load_or_generate_thumbnail(&self, page: usize) -> ImageHandle {
        // TODO: Re-enable cache once infrastructure layer is set up
        // if let Some(handle) = cache::load_thumbnail(&self.source_path, page) {
        //     return handle;
        // }

        match Self::render_page_at_scale(
            &self.document,
            page,
            RotationMode::Standard(Rotation::None),
            PDF_THUMBNAIL_SIZE,
        ) {
            Ok(img) => {
                // TODO: Re-enable cache once infrastructure layer is set up
                // let _ = cache::save_thumbnail(&self.source_path, page, &img);
                Self::create_image_handle_from_image(&img)
            }
            Err(e) => {
                log::warn!("Failed to generate thumbnail for page {page}: {e}");
                ImageHandle::from_rgba(1, 1, vec![0, 0, 0, 0])
            }
        }
    }

    /// Render a specific page from the document to an image.
    fn render_page(
        document: &PopplerDocument,
        page_index: usize,
        rotation: RotationMode,
    ) -> anyhow::Result<DynamicImage> {
        Self::render_page_at_scale(document, page_index, rotation, PDF_RENDER_QUALITY)
    }

    /// Render a specific page at a given scale.
    fn render_page_at_scale(
        document: &PopplerDocument,
        page_index: usize,
        rotation: RotationMode,
        scale: f64,
    ) -> anyhow::Result<DynamicImage> {
        let page = document
            .get_page(page_index)
            .ok_or_else(|| anyhow::anyhow!("Failed to get page {page_index}"))?;

        let (page_width, page_height) = page.get_size();
        let rotation_degrees = rotation.to_degrees() as i16;

        let (width, height) = if rotation_degrees == 90 || rotation_degrees == 270 {
            (page_height, page_width)
        } else {
            (page_width, page_height)
        };

        #[allow(clippy::cast_possible_truncation)]
        let scaled_width = (width * scale) as i32;
        #[allow(clippy::cast_possible_truncation)]
        let scaled_height = (height * scale) as i32;

        let surface = ImageSurface::create(Format::ARgb32, scaled_width, scaled_height)
            .map_err(|e| anyhow::anyhow!("Failed to create Cairo surface: {e}"))?;

        let context = Context::new(&surface)
            .map_err(|e| anyhow::anyhow!("Failed to create Cairo context: {e}"))?;

        // Fill with white background.
        context.set_source_rgb(1.0, 1.0, 1.0);
        let _ = context.paint();

        context.scale(scale, scale);

        if rotation != RotationMode::Standard(Rotation::None) {
            let center_x = width / 2.0;
            let center_y = height / 2.0;
            context.translate(center_x, center_y);
            context.rotate(f64::from(rotation_degrees) * std::f64::consts::PI / 180.0);
            context.translate(-page_width / 2.0, -page_height / 2.0);
        }

        page.render(&context);

        drop(context);
        surface.flush();

        let mut png_data: Vec<u8> = Vec::new();
        surface
            .write_to_png(&mut png_data)
            .map_err(|e| anyhow::anyhow!("Failed to write PNG: {e}"))?;

        let image = ImageReader::new(Cursor::new(png_data))
            .with_guessed_format()
            .map_err(|e| anyhow::anyhow!("Failed to read PNG format: {e}"))?
            .decode()
            .map_err(|e| anyhow::anyhow!("Failed to decode PNG: {e}"))?;

        Ok(image)
    }

    /// Re-render the current page with current transform.
    fn rerender(&mut self) {
        match Self::render_page(&self.document, self.page_index, self.transform.rotation) {
            Ok(mut rendered) => {
                // Apply flip transformations to the rendered result
                if self.transform.flip_h {
                    rendered = Self::apply_flip(rendered, FlipDirection::Horizontal);
                }
                if self.transform.flip_v {
                    rendered = Self::apply_flip(rendered, FlipDirection::Vertical);
                }
                self.rendered = rendered;
                self.handle = Self::create_image_handle_from_image(&self.rendered);
            }
            Err(e) => {
                log::error!("Failed to render PDF page: {e}");
            }
        }
    }

    fn apply_flip(img: DynamicImage, direction: FlipDirection) -> DynamicImage {
        use image::imageops::{flip_horizontal, flip_vertical};
        match direction {
            FlipDirection::Horizontal => DynamicImage::ImageRgba8(flip_horizontal(&img.to_rgba8())),
            FlipDirection::Vertical => DynamicImage::ImageRgba8(flip_vertical(&img.to_rgba8())),
        }
    }

    /// Navigate to the next page.
    #[allow(dead_code)]
    pub fn next_page(&mut self) -> bool {
        if self.page_index + 1 < self.num_pages {
            self.page_index += 1;
            self.rerender();
            true
        } else {
            false
        }
    }

    /// Navigate to the previous page.
    #[allow(dead_code)]
    pub fn prev_page(&mut self) -> bool {
        if self.page_index > 0 {
            self.page_index -= 1;
            self.rerender();
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

impl Renderable for PortableDocument {
    fn render(&mut self, _scale: f64) -> DocResult<RenderOutput> {
        // PDF rendering quality is fixed for now (PDF_RENDER_QUALITY)
        let (width, height) = self.dimensions();
        Ok(RenderOutput {
            handle: self.handle.clone(),
            width,
            height,
        })
    }

    fn info(&self) -> DocumentInfo {
        let (width, height) = self.dimensions();
        DocumentInfo {
            width,
            height,
            format: "PDF".to_string(),
        }
    }
}

impl Transformable for PortableDocument {
    fn rotate(&mut self, rotation: Rotation) {
        self.transform.rotation = RotationMode::Standard(rotation);
        self.rerender();
    }

    fn flip(&mut self, direction: FlipDirection) {
        match direction {
            FlipDirection::Horizontal => self.transform.flip_h = !self.transform.flip_h,
            FlipDirection::Vertical => self.transform.flip_v = !self.transform.flip_v,
        }
        self.rerender();
    }

    fn transform_state(&self) -> TransformState {
        self.transform
    }
}

impl MultiPage for PortableDocument {
    fn page_count(&self) -> usize {
        self.num_pages
    }

    fn current_page(&self) -> usize {
        self.page_index
    }

    fn go_to_page(&mut self, page: usize) -> DocResult<()> {
        if page >= self.num_pages {
            return Err(anyhow::anyhow!(
                "Page {} out of range (0-{})",
                page,
                self.num_pages - 1
            ));
        }
        self.page_index = page;
        self.rerender();
        Ok(())
    }
}

impl MultiPageThumbnails for PortableDocument {
    fn thumbnails_ready(&self) -> bool {
        self.thumbnail_cache
            .as_ref()
            .is_some_and(|c| c.len() >= self.num_pages)
    }

    fn thumbnails_loaded(&self) -> bool {
        let loaded = PortableDocument::thumbnails_loaded(self);
        loaded >= self.num_pages
    }

    fn generate_thumbnail_page(&mut self, page: usize) -> DocResult<()> {
        PortableDocument::generate_thumbnail_page(self, page);
        Ok(())
    }

    fn generate_all_thumbnails(&mut self) -> DocResult<()> {
        if self.thumbnails_ready() {
            return Ok(());
        }
        self.init_thumbnail_cache();
        for page in 0..self.num_pages {
            PortableDocument::generate_thumbnail_page(self, page);
        }
        Ok(())
    }

    fn get_thumbnail(&mut self, page: usize) -> DocResult<Option<ImageHandle>> {
        Ok(self
            .thumbnail_cache
            .as_ref()
            .and_then(|cache| cache.get(page).cloned()))
    }
}
