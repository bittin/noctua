// SPDX-License-Identifier: GPL-3.0-or-later
// src/ui/widgets/crop_overlay.rs
//
// Simple crop overlay.

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

use crate::ui::widgets::crop_model::{CropSelection, DragHandle};
use crate::ui::AppMessage;

const HANDLE_SIZE: f32 = 12.0;
const HANDLE_HIT_SIZE: f32 = 24.0;
const OVERLAY_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
const HANDLE_COLOR: Color = Color::WHITE;
const BORDER_COLOR: Color = Color::WHITE;
const BORDER_WIDTH: f32 = 2.0;

pub struct CropOverlay {
    selection: CropSelection,
    show_grid: bool,
    last_click: Option<std::time::Instant>,
}

impl CropOverlay {
    pub fn new(selection: &CropSelection, show_grid: bool) -> Self {
        Self {
            selection: selection.clone(),
            show_grid,
            last_click: None,
        }
    }

    fn hit_test_handle(&self, point: Point) -> DragHandle {
        let Some((x, y, w, h)) = self.selection.region else {
            return DragHandle::None;
        };

        let handles = [
            (Point::new(x, y), DragHandle::TopLeft),
            (Point::new(x + w, y), DragHandle::TopRight),
            (Point::new(x, y + h), DragHandle::BottomLeft),
            (Point::new(x + w, y + h), DragHandle::BottomRight),
            (Point::new(x + w / 2.0, y), DragHandle::Top),
            (Point::new(x + w / 2.0, y + h), DragHandle::Bottom),
            (Point::new(x, y + h / 2.0), DragHandle::Left),
            (Point::new(x + w, y + h / 2.0), DragHandle::Right),
        ];

        for (pos, handle) in handles {
            if point_in_handle(point, pos) {
                return handle;
            }
        }

        if point.x >= x && point.x <= x + w && point.y >= y && point.y <= y + h {
            return DragHandle::Move;
        }

        DragHandle::None
    }

    fn draw_overlay(&self, renderer: &mut Renderer, bounds: Rectangle) {
        let Some((x, y, w, h)) = self.selection.region else {
            draw_quad(renderer, bounds, OVERLAY_COLOR);
            return;
        };

        // Convert relative coords to absolute screen coords
        let abs_x = bounds.x + x;
        let abs_y = bounds.y + y;
        let abs_right = abs_x + w;
        let abs_bottom = abs_y + h;

        // Top
        if abs_y > bounds.y {
            draw_quad(
                renderer,
                Rectangle::new(
                    Point::new(bounds.x, bounds.y),
                    Size::new(bounds.width, abs_y - bounds.y),
                ),
                OVERLAY_COLOR,
            );
        }

        // Bottom
        if abs_bottom < bounds.y + bounds.height {
            draw_quad(
                renderer,
                Rectangle::new(
                    Point::new(bounds.x, abs_bottom),
                    Size::new(bounds.width, bounds.y + bounds.height - abs_bottom),
                ),
                OVERLAY_COLOR,
            );
        }

        // Left
        if abs_x > bounds.x {
            draw_quad(
                renderer,
                Rectangle::new(
                    Point::new(bounds.x, abs_y),
                    Size::new(abs_x - bounds.x, h),
                ),
                OVERLAY_COLOR,
            );
        }

        // Right
        if abs_right < bounds.x + bounds.width {
            draw_quad(
                renderer,
                Rectangle::new(
                    Point::new(abs_right, abs_y),
                    Size::new(bounds.x + bounds.width - abs_right, h),
                ),
                OVERLAY_COLOR,
            );
        }
    }

    fn draw_border(&self, renderer: &mut Renderer, bounds: Rectangle) {
        let Some((x, y, w, h)) = self.selection.region else {
            return;
        };

        // Add bounds offset
        let abs_x = bounds.x + x;
        let abs_y = bounds.y + y;

        // Top
        draw_quad(
            renderer,
            Rectangle::new(Point::new(abs_x, abs_y), Size::new(w, BORDER_WIDTH)),
            BORDER_COLOR,
        );

        // Bottom
        draw_quad(
            renderer,
            Rectangle::new(
                Point::new(abs_x, abs_y + h - BORDER_WIDTH),
                Size::new(w, BORDER_WIDTH),
            ),
            BORDER_COLOR,
        );

        // Left
        draw_quad(
            renderer,
            Rectangle::new(Point::new(abs_x, abs_y), Size::new(BORDER_WIDTH, h)),
            BORDER_COLOR,
        );

        // Right
        draw_quad(
            renderer,
            Rectangle::new(
                Point::new(abs_x + w - BORDER_WIDTH, abs_y),
                Size::new(BORDER_WIDTH, h),
            ),
            BORDER_COLOR,
        );
    }

    fn draw_handles(&self, renderer: &mut Renderer, bounds: Rectangle) {
        let Some((x, y, w, h)) = self.selection.region else {
            return;
        };

        let half = HANDLE_SIZE / 2.0;

        // Add bounds offset
        let abs_x = bounds.x + x;
        let abs_y = bounds.y + y;

        let handles = [
            Point::new(abs_x, abs_y),
            Point::new(abs_x + w, abs_y),
            Point::new(abs_x, abs_y + h),
            Point::new(abs_x + w, abs_y + h),
            Point::new(abs_x + w / 2.0, abs_y),
            Point::new(abs_x + w / 2.0, abs_y + h),
            Point::new(abs_x, abs_y + h / 2.0),
            Point::new(abs_x + w, abs_y + h / 2.0),
        ];

        for pos in handles {
            draw_quad(
                renderer,
                Rectangle::new(
                    Point::new(pos.x - half, pos.y - half),
                    Size::new(HANDLE_SIZE, HANDLE_SIZE),
                ),
                HANDLE_COLOR,
            );
        }
    }

    fn draw_grid(&self, renderer: &mut Renderer, bounds: Rectangle) {
        if !self.show_grid {
            return;
        }

        let Some((x, y, w, h)) = self.selection.region else {
            return;
        };

        if w <= 10.0 || h <= 10.0 {
            return;
        }

        // Add bounds offset
        let abs_x = bounds.x + x;
        let abs_y = bounds.y + y;

        let grid_color = Color::from_rgba(1.0, 1.0, 1.0, 0.3);
        let third_w = w / 3.0;
        let third_h = h / 3.0;

        // 2 vertical
        for i in 1..3 {
            let line_x = abs_x + third_w * i as f32;
            draw_quad(
                renderer,
                Rectangle::new(Point::new(line_x, abs_y), Size::new(1.0, h)),
                grid_color,
            );
        }

        // 2 horizontal
        for i in 1..3 {
            let line_y = abs_y + third_h * i as f32;
            draw_quad(
                renderer,
                Rectangle::new(Point::new(abs_x, line_y), Size::new(w, 1.0)),
                grid_color,
            );
        }
    }
}

impl Widget<AppMessage, cosmic::Theme, Renderer> for CropOverlay {
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

        self.draw_overlay(renderer, bounds);
        self.draw_border(renderer, bounds);
        self.draw_handles(renderer, bounds);
        self.draw_grid(renderer, bounds);
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, AppMessage>,
        _viewport: &Rectangle,
    ) -> Status {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let handle = self.hit_test_handle(pos);

                    if handle == DragHandle::Move {
                        use std::time::{Duration, Instant};
                        let now = Instant::now();
                        if let Some(last) = self.last_click {
                            if now.duration_since(last) < Duration::from_millis(400) {
                                shell.publish(AppMessage::ApplyCrop);
                                self.last_click = None;
                                return Status::Captured;
                            }
                        }
                        self.last_click = Some(now);
                    }

                    shell.publish(AppMessage::CropDragStart {
                        x: pos.x,
                        y: pos.y,
                        handle,
                    });
                    return Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.selection.is_dragging {
                    if let Some(pos) = cursor.position_in(bounds) {
                        shell.publish(AppMessage::CropDragMove {
                            x: pos.x,
                            y: pos.y,
                            max_x: bounds.width,
                            max_y: bounds.height,
                        });
                        return Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                if self.selection.is_dragging {
                    shell.publish(AppMessage::CropDragEnd);
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

        if let Some(pos) = cursor.position_in(bounds) {
            let handle = self.hit_test_handle(pos);
            return match handle {
                DragHandle::TopLeft | DragHandle::BottomRight => {
                    mouse::Interaction::ResizingDiagonallyDown
                }
                DragHandle::TopRight | DragHandle::BottomLeft => {
                    mouse::Interaction::ResizingDiagonallyUp
                }
                DragHandle::Top | DragHandle::Bottom => mouse::Interaction::ResizingVertically,
                DragHandle::Left | DragHandle::Right => mouse::Interaction::ResizingHorizontally,
                DragHandle::Move => mouse::Interaction::Grabbing,
                DragHandle::None => mouse::Interaction::Crosshair,
            };
        }

        mouse::Interaction::None
    }
}

impl<'a> From<CropOverlay> for Element<'a, AppMessage> {
    fn from(widget: CropOverlay) -> Self {
        Element::new(widget)
    }
}

fn point_in_handle(point: Point, handle_center: Point) -> bool {
    let half = HANDLE_HIT_SIZE / 2.0;
    point.x >= handle_center.x - half
        && point.x <= handle_center.x + half
        && point.y >= handle_center.y - half
        && point.y <= handle_center.y + half
}

fn draw_quad(renderer: &mut Renderer, bounds: Rectangle, color: Color) {
    renderer.fill_quad(
        Quad {
            bounds,
            ..Quad::default()
        },
        color,
    );
}

pub fn crop_overlay<'a>(selection: &CropSelection, show_grid: bool) -> Element<'a, AppMessage> {
    CropOverlay::new(selection, show_grid).into()
}
