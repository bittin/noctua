// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/view/canvas.rs
//
// Render the center canvas area with the current document.

use cosmic::iced::{ContentFit, Length};
use cosmic::iced_widget::stack;
use cosmic::widget::{container, text};
use cosmic::Element;

use super::crop::crop_overlay;
use super::image_viewer::Viewer;
use crate::app::model::{ToolMode, ViewMode};
use crate::app::{AppMessage, AppModel};
use crate::config::AppConfig;
use crate::fl;

/// Render the center canvas area with the current document.
pub fn view<'a>(model: &'a AppModel, config: &'a AppConfig) -> Element<'a, AppMessage> {
    if let Some(doc) = &model.document {
        let handle = doc.handle();
        let (width, height) = doc.dimensions();

        let (scale, content_fit) = match model.view_mode {
            ViewMode::Fit => (1.0, ContentFit::Contain),
            ViewMode::ActualSize => (1.0, ContentFit::None),
            ViewMode::Custom(z) => (z, ContentFit::None),
        };

        let img_viewer = Viewer::new(handle)
            .with_state(scale, model.pan_x, model.pan_y)
            .on_state_change(|scale, offset_x, offset_y| AppMessage::ViewerStateChanged {
                scale,
                offset_x,
                offset_y,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(content_fit)
            .min_scale(config.min_scale)
            .max_scale(config.max_scale)
            .scale_step(config.scale_step - 1.0);

        if model.tool_mode == ToolMode::Crop {
            let overlay = crop_overlay(
                width,
                height,
                &model.crop_selection,
                config.crop_show_grid,
                scale,
                model.pan_x,
                model.pan_y,
            );

            stack![overlay, img_viewer].into()
        } else {
            container(img_viewer)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    } else {
        container(text(fl!("no-document")))
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    }
}
