// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/view/crop/overlay.rs
//
// Crop overlay widget with selection UI (overlay, border, handles, grid).
// Inspired by cosmic-viewer (https://codeberg.org/bhh by Bryan Hyland

use crate::app::view::crop::selection::{CropSelection, DragHandle};
use cosmic::{
    Element, Renderer,
    iced::{
        Color, Length, Point, Rectangle, Size,
        advanced::{
            Clipboard, Layout, Shell, Widget,
            layout::{Limits, Node},
            renderer::{Quad, Renderer as QuadRenderer},
            widget::Tree,
        },
        event::{Event, Status},
        mouse::{self, Button, Cursor},
    },
};

const HANDLE_SIZE: f32 = 14.0;
const HANDLE_HIT_SIZE: f32 = 28.0;
const OVERLAY_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
const HANDLE_COLOR: Color = Color::WHITE;
const BORDER_COLOR: Color = Color::WHITE;
const BORDER_WIDTH: f32 = 2.0;
const GRID_COLOR: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.8);
const GRID_WIDTH: f32 = 1.0;

pub struct CropOverlay {
    img_width: u32,
    img_height: u32,
    selection: CropSelection,
    show_grid: bool,
    scale: f32,
    pan_x: f32,
    pan_y: f32,
}

impl CropOverlay {
    pub fn new(
        img_width: u32,
        img_height: u32,
        selection: &CropSelection,
        show_grid: bool,
        scale: f32,
        pan_x: f32,
        pan_y: f32,
    ) -> Self {
        Self {
            img_width,
            img_height,
            selection: selection.clone(),
            show_grid,
            scale,
            pan_x,
            pan_y,
        }
    }

    fn get_base_scale(&self, bounds: &Rectangle) -> f32 {
        let scale_x = bounds.width / self.img_width as f32;
        let scale_y = bounds.height / self.img_height as f32;
        scale_x.min(scale_y) // Fit to bounds (wie bei ViewMode::Fit)
    }

    fn get_effective_scale(&self, bounds: &Rectangle) -> f32 {
        if self.scale > 0.0 {
            self.scale
        } else {
            self.get_base_scale(bounds)
        }
    }

    fn screen_to_image(&self, bounds: &Rectangle, point: Point) -> (f32, f32) {
        let effective_scale = self.get_effective_scale(bounds);

        // Berechne zentrierte Position des Images mit aktuellem Zoom
        let img_screen_width = self.img_width as f32 * effective_scale;
        let img_screen_height = self.img_height as f32 * effective_scale;
        let offset_x = (bounds.width - img_screen_width) / 2.0 - self.pan_x;
        let offset_y = (bounds.height - img_screen_height) / 2.0 - self.pan_y;

        let x = ((point.x - bounds.x - offset_x) / effective_scale)
            .max(0.0)
            .min(self.img_width as f32);
        let y = ((point.y - bounds.y - offset_y) / effective_scale)
            .max(0.0)
            .min(self.img_height as f32);
        (x, y)
    }

    fn image_to_screen(&self, bounds: &Rectangle, img_x: f32, img_y: f32) -> Point {
        let effective_scale = self.get_effective_scale(bounds);

        // Berechne zentrierte Position des Images mit aktuellem Zoom
        let img_screen_width = self.img_width as f32 * effective_scale;
        let img_screen_height = self.img_height as f32 * effective_scale;
        let offset_x = (bounds.width - img_screen_width) / 2.0 - self.pan_x;
        let offset_y = (bounds.height - img_screen_height) / 2.0 - self.pan_y;

        Point::new(
            bounds.x + offset_x + img_x * effective_scale,
            bounds.y + offset_y + img_y * effective_scale,
        )
    }

    fn hit_test_handle(&self, bounds: &Rectangle, point: Point) -> DragHandle {
        let Some((rx, ry, rw, rh)) = self.selection.region else {
            return DragHandle::None;
        };

        let top_left = self.image_to_screen(bounds, rx, ry);
        let top_right = self.image_to_screen(bounds, rx + rw, ry);
        let bottom_left = self.image_to_screen(bounds, rx, ry + rh);
        let bottom_right = self.image_to_screen(bounds, rx + rw, ry + rh);

        if self.point_in_handle(point, top_left) {
            return DragHandle::TopLeft;
        }
        if self.point_in_handle(point, top_right) {
            return DragHandle::TopRight;
        }
        if self.point_in_handle(point, bottom_left) {
            return DragHandle::BottomLeft;
        }
        if self.point_in_handle(point, bottom_right) {
            return DragHandle::BottomRight;
        }

        let mid_top = self.image_to_screen(bounds, rx + rw / 2.0, ry);
        let mid_bottom = self.image_to_screen(bounds, rx + rw / 2.0, ry + rh);
        let mid_left = self.image_to_screen(bounds, rx, ry + rh / 2.0);
        let mid_right = self.image_to_screen(bounds, rx + rw, ry + rh / 2.0);

        if self.point_in_handle(point, mid_top) {
            return DragHandle::Top;
        }
        if self.point_in_handle(point, mid_bottom) {
            return DragHandle::Bottom;
        }
        if self.point_in_handle(point, mid_left) {
            return DragHandle::Left;
        }
        if self.point_in_handle(point, mid_right) {
            return DragHandle::Right;
        }

        let selection_rect = Rectangle::new(
            top_left,
            Size::new(bottom_right.x - top_left.x, bottom_right.y - top_left.y),
        );

        if selection_rect.contains(point) {
            return DragHandle::Move;
        }

        DragHandle::None
    }

    fn point_in_handle(&self, point: Point, handle_center: Point) -> bool {
        let half = HANDLE_HIT_SIZE / 2.0;
        point.x >= handle_center.x - half
            && point.x <= handle_center.x + half
            && point.y >= handle_center.y - half
            && point.y <= handle_center.y + half
    }

    fn cursor_for_handle(&self, handle: DragHandle) -> mouse::Interaction {
        match handle {
            DragHandle::None => mouse::Interaction::Crosshair,
            DragHandle::TopLeft | DragHandle::BottomRight => {
                mouse::Interaction::ResizingDiagonallyDown
            }
            DragHandle::TopRight | DragHandle::BottomLeft => {
                mouse::Interaction::ResizingDiagonallyUp
            }
            DragHandle::Top | DragHandle::Bottom => mouse::Interaction::ResizingVertically,
            DragHandle::Left | DragHandle::Right => mouse::Interaction::ResizingHorizontally,
            DragHandle::Move => mouse::Interaction::Grabbing,
        }
    }
}

impl Widget<super::super::super::AppMessage, cosmic::Theme, Renderer> for CropOverlay {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &cosmic::Theme,
        _style: &cosmic::iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let effective_scale = self.get_effective_scale(&bounds);

        if let Some((rx, ry, rw, rh)) = self.selection.region {
            if rw > 0.0 && rh > 0.0 {
                // Berechne zentrierte Position des Images mit aktuellem Zoom/Pan
                let img_screen_width = self.img_width as f32 * effective_scale;
                let img_screen_height = self.img_height as f32 * effective_scale;
                let offset_x = (bounds.width - img_screen_width) / 2.0 - self.pan_x;
                let offset_y = (bounds.height - img_screen_height) / 2.0 - self.pan_y;

                let sel_x = bounds.x + offset_x + rx * effective_scale;
                let sel_y = bounds.y + offset_y + ry * effective_scale;
                let sel_w = rw * effective_scale;
                let sel_h = rh * effective_scale;

                if sel_y > bounds.y {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                bounds.position(),
                                Size::new(bounds.width, sel_y - bounds.y),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }

                let sel_bottom = sel_y + sel_h;
                let img_bottom = bounds.y + bounds.height;
                if sel_bottom < img_bottom {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(bounds.x, sel_bottom),
                                Size::new(bounds.width, img_bottom - sel_bottom),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }

                if sel_x > bounds.x {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(bounds.x, sel_y),
                                Size::new(sel_x - bounds.x, sel_h),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }

                let sel_right = sel_x + sel_w;
                let img_right = bounds.x + bounds.width;
                if sel_right < img_right {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(sel_right, sel_y),
                                Size::new(img_right - sel_right, sel_h),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }

                let border_width = BORDER_WIDTH;
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(sel_x, sel_y),
                            Size::new(sel_w, border_width),
                        ),
                        ..Quad::default()
                    },
                    BORDER_COLOR,
                );
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(sel_x, sel_y + sel_h - border_width),
                            Size::new(sel_w, border_width),
                        ),
                        ..Quad::default()
                    },
                    BORDER_COLOR,
                );
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(sel_x, sel_y),
                            Size::new(border_width, sel_h),
                        ),
                        ..Quad::default()
                    },
                    BORDER_COLOR,
                );
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(sel_x + sel_w - border_width, sel_y),
                            Size::new(border_width, sel_h),
                        ),
                        ..Quad::default()
                    },
                    BORDER_COLOR,
                );

                let handle_half = HANDLE_SIZE / 2.0;
                let handles = [
                    (sel_x, sel_y),
                    (sel_x + sel_w, sel_y),
                    (sel_x, sel_y + sel_h),
                    (sel_x + sel_w, sel_y + sel_h),
                    (sel_x + sel_w / 2.0, sel_y),
                    (sel_x + sel_w / 2.0, sel_y + sel_h),
                    (sel_x, sel_y + sel_h / 2.0),
                    (sel_x + sel_w, sel_y + sel_h / 2.0),
                ];

                for (hx, hy) in handles {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(hx - handle_half, hy - handle_half),
                                Size::new(HANDLE_SIZE, HANDLE_SIZE),
                            ),
                            ..Quad::default()
                        },
                        HANDLE_COLOR,
                    );
                }

                if self.show_grid && rw > 10.0 && rh > 10.0 {
                    let grid_sp_x = sel_w / 3.0;
                    let grid_sp_y = sel_h / 3.0;

                    for i in 1..3 {
                        let offset_x = sel_x + grid_sp_x * i as f32;
                        let offset_y = sel_y + grid_sp_y * i as f32;

                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle::new(
                                    Point::new(offset_x, sel_y),
                                    Size::new(GRID_WIDTH, sel_h),
                                ),
                                ..Quad::default()
                            },
                            GRID_COLOR,
                        );
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle::new(
                                    Point::new(sel_x, offset_y),
                                    Size::new(sel_w, GRID_WIDTH),
                                ),
                                ..Quad::default()
                            },
                            GRID_COLOR,
                        );
                    }
                }
            } else {
                renderer.fill_quad(
                    Quad {
                        bounds,
                        ..Quad::default()
                    },
                    OVERLAY_COLOR,
                );
            }
        } else {
            renderer.fill_quad(
                Quad {
                    bounds,
                    ..Quad::default()
                },
                OVERLAY_COLOR,
            );
        }
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, super::super::super::AppMessage>,
        _viewport: &Rectangle,
    ) -> Status {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let handle = self.hit_test_handle(&bounds, pos);
                    let (img_x, img_y) = self.screen_to_image(&bounds, pos);

                    shell.publish(super::super::super::AppMessage::CropDragStart {
                        x: img_x,
                        y: img_y,
                        handle,
                    });
                    // Always capture in crop mode to prevent image viewer from panning
                    return Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.selection.is_dragging {
                    if let Some(pos) = cursor.position_in(bounds) {
                        let (img_x, img_y) = self.screen_to_image(&bounds, pos);
                        shell.publish(super::super::super::AppMessage::CropDragMove {
                            x: img_x,
                            y: img_y,
                        });
                        return Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                if self.selection.is_dragging {
                    shell.publish(super::super::super::AppMessage::CropDragEnd);
                    return Status::Captured;
                }
            }
            _ => {}
        }

        Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        if self.selection.is_dragging {
            return self.cursor_for_handle(self.selection.drag_handle);
        }

        if let Some(pos) = cursor.position_in(bounds) {
            let handle = self.hit_test_handle(&bounds, pos);
            if handle != DragHandle::None {
                return self.cursor_for_handle(handle);
            }
            if bounds.contains(pos) {
                return mouse::Interaction::Crosshair;
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a> From<CropOverlay> for Element<'a, super::super::super::AppMessage> {
    fn from(overlay: CropOverlay) -> Self {
        Self::new(overlay)
    }
}

pub fn crop_overlay(
    img_width: u32,
    img_height: u32,
    selection: &CropSelection,
    show_grid: bool,
    scale: f32,
    pan_x: f32,
    pan_y: f32,
) -> CropOverlay {
    CropOverlay::new(
        img_width, img_height, selection, show_grid, scale, pan_x, pan_y,
    )
}
