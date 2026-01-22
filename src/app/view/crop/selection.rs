// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/view/crop/selection.rs
//
// Crop selection state and drag handle types.
// Inspired by cosmic-viewer (https://codeberg.org/bhh by Bryan Hyland

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DragHandle {
    #[default]
    None,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Move,
}

#[derive(Debug, Clone, Default)]
pub struct CropSelection {
    pub region: Option<(f32, f32, f32, f32)>,
    pub is_dragging: bool,
    pub drag_handle: DragHandle,
    pub drag_start: Option<(f32, f32)>,
    pub drag_start_region: Option<(f32, f32, f32, f32)>,
}

impl CropSelection {
    pub fn start_new_selection(&mut self, x: f32, y: f32) {
        self.region = Some((x, y, 0.0, 0.0));
        self.is_dragging = true;
        self.drag_handle = DragHandle::None;
        self.drag_start = Some((x, y));
        self.drag_start_region = None;
    }

    pub fn start_handle_drag(&mut self, handle: DragHandle, x: f32, y: f32) {
        self.is_dragging = true;
        self.drag_handle = handle;
        self.drag_start = Some((x, y));
        self.drag_start_region = self.region;
    }

    pub fn update_drag(&mut self, x: f32, y: f32, img_width: f32, img_height: f32) {
        if !self.is_dragging {
            return;
        }

        match self.drag_handle {
            DragHandle::None => {
                if let Some((start_x, start_y)) = self.drag_start {
                    let min_x = start_x.min(x).max(0.0);
                    let min_y = start_y.min(y).max(0.0);
                    let max_x = start_x.max(x).min(img_width);
                    let max_y = start_y.max(y).min(img_height);

                    self.region = Some((min_x, min_y, max_x - min_x, max_y - min_y));
                }
            }
            DragHandle::Move => {
                if let (Some((start_x, start_y)), Some((rx, ry, rw, rh))) =
                    (self.drag_start, self.drag_start_region)
                {
                    let dx = x - start_x;
                    let dy = y - start_y;
                    let new_x = (rx + dx).max(0.0).min(img_width - rw);
                    let new_y = (ry + dy).max(0.0).min(img_height - rh);
                    self.region = Some((new_x, new_y, rw, rh));
                }
            }
            _ => {
                if let (Some((start_x, start_y)), Some((rx, ry, rw, rh))) =
                    (self.drag_start, self.drag_start_region)
                {
                    let dx = x - start_x;
                    let dy = y - start_y;

                    let (new_x, new_y, new_w, new_h) =
                        self.resize_region(rx, ry, rw, rh, dx, dy, img_width, img_height);
                    self.region = Some((new_x, new_y, new_w, new_h));
                }
            }
        }
    }

    fn resize_region(
        &self,
        rx: f32,
        ry: f32,
        rw: f32,
        rh: f32,
        dx: f32,
        dy: f32,
        img_width: f32,
        img_height: f32,
    ) -> (f32, f32, f32, f32) {
        const MIN_SIZE: f32 = 1.0;
        let right = rx + rw;
        let bottom = ry + rh;

        match self.drag_handle {
            DragHandle::TopLeft => {
                let new_rx = (rx + dx).max(0.0).min(right - MIN_SIZE);
                let new_ry = (ry + dy).max(0.0).min(bottom - MIN_SIZE);
                let new_rw = (right - new_rx).max(MIN_SIZE).min(img_width - new_rx);
                let new_rh = (bottom - new_ry).max(MIN_SIZE).min(img_height - new_ry);
                (new_rx, new_ry, new_rw, new_rh)
            }
            DragHandle::TopRight => {
                let new_right = (right + dx).max(rx + MIN_SIZE).min(img_width);
                let new_ry = (ry + dy).max(0.0).min(bottom - MIN_SIZE);
                let new_rw = (new_right - rx).max(MIN_SIZE);
                let new_rh = (bottom - new_ry).max(MIN_SIZE).min(img_height - new_ry);
                (rx, new_ry, new_rw, new_rh)
            }
            DragHandle::BottomLeft => {
                let new_rx = (rx + dx).max(0.0).min(right - MIN_SIZE);
                let new_bottom = (bottom + dy).max(ry + MIN_SIZE).min(img_height);
                let new_rw = (right - new_rx).max(MIN_SIZE);
                let new_rh = (new_bottom - ry).max(MIN_SIZE);
                (new_rx, ry, new_rw, new_rh)
            }
            DragHandle::BottomRight => {
                let new_right = (right + dx).max(rx + MIN_SIZE).min(img_width);
                let new_bottom = (bottom + dy).max(ry + MIN_SIZE).min(img_height);
                let new_rw = (new_right - rx).max(MIN_SIZE);
                let new_rh = (new_bottom - ry).max(MIN_SIZE);
                (rx, ry, new_rw, new_rh)
            }
            DragHandle::Top => {
                let new_ry = (ry + dy).max(0.0).min(bottom - MIN_SIZE);
                let new_rh = (bottom - new_ry).max(MIN_SIZE);
                (rx, new_ry, rw, new_rh)
            }
            DragHandle::Bottom => {
                let new_bottom = (bottom + dy).max(ry + MIN_SIZE).min(img_height);
                let new_rh = (new_bottom - ry).max(MIN_SIZE);
                (rx, ry, rw, new_rh)
            }
            DragHandle::Left => {
                let new_rx = (rx + dx).max(0.0).min(right - MIN_SIZE);
                let new_rw = (right - new_rx).max(MIN_SIZE);
                (new_rx, ry, new_rw, rh)
            }
            DragHandle::Right => {
                let new_right = (right + dx).max(rx + MIN_SIZE).min(img_width);
                let new_rw = (new_right - rx).max(MIN_SIZE);
                (rx, ry, new_rw, rh)
            }
            _ => (rx, ry, rw, rh),
        }
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.drag_start = None;
        self.drag_start_region = None;
    }

    pub fn reset(&mut self) {
        self.region = None;
        self.is_dragging = false;
        self.drag_handle = DragHandle::None;
        self.drag_start = None;
        self.drag_start_region = None;
    }

    pub fn has_selection(&self) -> bool {
        self.region.is_some_and(|(_, _, w, h)| w > 1.0 && h > 1.0)
    }

    pub fn as_pixel_rect(&self) -> Option<(u32, u32, u32, u32)> {
        self.region.and_then(|(x, y, w, h)| {
            if w > 1.0 && h > 1.0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Some((x as u32, y as u32, w as u32, h as u32))
            } else {
                None
            }
        })
    }
}
