// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/document/vector.rs
//
// Vector documents (SVG, etc.).

use std::path::Path;

use image::{imageops, DynamicImage, RgbaImage};
use resvg::tiny_skia::{self, Pixmap};
use resvg::usvg::{Options, Tree};

use super::ImageHandle;
use crate::constant::{FULL_ROTATION, MIN_PIXMAP_SIZE, ROTATION_STEP};

/// Accumulated transformations for a vector document.
#[derive(Debug, Clone, Copy, Default)]
pub struct VectorTransform {
    /// Rotation in degrees (0, 90, 180, 270).
    pub rotation: i16,
    /// Horizontal flip.
    pub flip_h: bool,
    /// Vertical flip.
    pub flip_v: bool,
}

/// Represents a vector document such as SVG.
pub struct VectorDocument {
    /// Parsed SVG document for re-rendering at different scales.
    document: Tree,
    /// Native width of the SVG (from viewBox or width attribute).
    native_width: u32,
    /// Native height of the SVG (from viewBox or height attribute).
    native_height: u32,
    /// Current render scale (1.0 = native size).
    current_scale: f32,
    /// Accumulated transformations.
    transform: VectorTransform,
    /// Rasterized image at the current scale.
    pub rendered: DynamicImage,
    /// Image handle for display.
    pub handle: ImageHandle,
    /// Current rendered width.
    pub width: u32,
    /// Current rendered height.
    pub height: u32,
}

impl VectorDocument {
    /// Load a vector document from disk.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let raw_data = std::fs::read_to_string(path)?;

        // Parse SVG with default options.
        let options = Options::default();
        let document = Tree::from_str(&raw_data, &options)?;

        // Get native size from the parsed document.
        let size = document.size();
        let native_width = size.width().ceil() as u32;
        let native_height = size.height().ceil() as u32;

        let transform = VectorTransform::default();

        // Render at native scale (1.0).
        let (rendered, width, height) =
            render_document(&document, native_width, native_height, 1.0, &transform)?;
        let handle = super::create_image_handle(&rendered);

        Ok(Self {
            document,
            native_width,
            native_height,
            current_scale: 1.0,
            transform,
            rendered,
            handle,
            width,
            height,
        })
    }

    /// Returns the dimensions of the rasterized representation.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Re-render the SVG at a new scale, preserving transformations.
    /// Returns true if re-rendering occurred.
    pub fn render_at_scale(&mut self, scale: f32) -> bool {
        // Skip if scale hasn't changed
        if (self.current_scale - scale).abs() < f32::EPSILON {
            return false;
        }

        match render_document(
            &self.document,
            self.native_width,
            self.native_height,
            scale,
            &self.transform,
        ) {
            Ok((rendered, width, height)) => {
                self.current_scale = scale;
                self.rendered = rendered;
                self.width = width;
                self.height = height;
                self.handle = super::create_image_handle(&self.rendered);
                true
            }
            Err(e) => {
                log::error!("Failed to re-render SVG at scale {}: {}", scale, e);
                false
            }
        }
    }

    /// Rotate 90 degrees clockwise.
    pub fn rotate_cw(&mut self) {
        self.transform.rotation =
            (self.transform.rotation + ROTATION_STEP).rem_euclid(FULL_ROTATION);
        self.rerender();
    }

    /// Rotate 90 degrees counter-clockwise.
    pub fn rotate_ccw(&mut self) {
        self.transform.rotation =
            (self.transform.rotation - ROTATION_STEP).rem_euclid(FULL_ROTATION);
        self.rerender();
    }

    /// Flip horizontally.
    pub fn flip_horizontal(&mut self) {
        self.transform.flip_h = !self.transform.flip_h;
        self.rerender();
    }

    /// Flip vertically.
    pub fn flip_vertical(&mut self) {
        self.transform.flip_v = !self.transform.flip_v;
        self.rerender();
    }

    /// Re-render with current scale and transform.
    fn rerender(&mut self) {
        if let Ok((rendered, width, height)) = render_document(
            &self.document,
            self.native_width,
            self.native_height,
            self.current_scale,
            &self.transform,
        ) {
            self.rendered = rendered;
            self.width = width;
            self.height = height;
            self.handle = super::create_image_handle(&self.rendered);
        }
    }

    /// Extract metadata for this vector document.
    pub fn extract_meta(&self, path: &Path) -> super::meta::DocumentMeta {
        // Report native dimensions in metadata.
        super::meta::build_vector_meta(path, self.native_width, self.native_height)
    }
}

/// Render the SVG document at a given scale with transformations.
fn render_document(
    document: &Tree,
    native_width: u32,
    native_height: u32,
    scale: f32,
    transform: &VectorTransform,
) -> anyhow::Result<(DynamicImage, u32, u32)> {
    let width = (((native_width as f32) * scale).ceil() as u32).max(MIN_PIXMAP_SIZE);
    let height = (((native_height as f32) * scale).ceil() as u32).max(MIN_PIXMAP_SIZE);

    let mut pixmap =
        Pixmap::new(width, height).ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

    let ts = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(document, ts, &mut pixmap.as_mut());

    let mut image = pixmap_to_dynamic_image(&pixmap);

    // Apply flip transformations
    if transform.flip_h {
        image = DynamicImage::ImageRgba8(imageops::flip_horizontal(&image));
    }
    if transform.flip_v {
        image = DynamicImage::ImageRgba8(imageops::flip_vertical(&image));
    }

    // Apply rotation
    image = match transform.rotation {
        90 => DynamicImage::ImageRgba8(imageops::rotate90(&image)),
        180 => DynamicImage::ImageRgba8(imageops::rotate180(&image)),
        270 => DynamicImage::ImageRgba8(imageops::rotate270(&image)),
        _ => image,
    };

    let final_width = image.width();
    let final_height = image.height();

    Ok((image, final_width, final_height))
}

/// Convert a tiny_skia Pixmap to a DynamicImage.
fn pixmap_to_dynamic_image(pixmap: &Pixmap) -> DynamicImage {
    let width = pixmap.width();
    let height = pixmap.height();

    // tiny_skia uses premultiplied alpha, we need to unpremultiply for image crate
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for pixel in pixmap.pixels() {
        let a = pixel.alpha();
        if a == 0 {
            pixels.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            // Unpremultiply: color = premultiplied_color * 255 / alpha
            let r = (pixel.red() as u16 * 255 / a as u16) as u8;
            let g = (pixel.green() as u16 * 255 / a as u16) as u8;
            let b = (pixel.blue() as u16 * 255 / a as u16) as u8;
            pixels.extend_from_slice(&[r, g, b, a]);
        }
    }

    let rgba_image = RgbaImage::from_raw(width, height, pixels)
        .expect("Failed to create RgbaImage from pixmap data");

    DynamicImage::ImageRgba8(rgba_image)
}
