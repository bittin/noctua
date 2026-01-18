// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/view/canvas.rs
//
// Render the center canvas area with the current document.

use cosmic::iced::{ContentFit, Length};
use cosmic::widget::{container, text};
use cosmic::Element;

use super::image_viewer::Viewer;
use crate::app::model::ViewMode;
use crate::app::{AppMessage, AppModel};
use crate::config::AppConfig;
use crate::fl;

/// Render the center canvas area with the current document.
pub fn view<'a>(model: &'a AppModel, config: &'a AppConfig) -> Element<'a, AppMessage> {
    if let Some(doc) = &model.document {
        let handle = doc.handle();

        // Determine zoom scale and content fit based on view mode
        let (scale, content_fit) = match model.view_mode {
            ViewMode::Fit => (1.0, ContentFit::Contain),
            ViewMode::ActualSize => (1.0, ContentFit::None),
            ViewMode::Custom(z) => (z, ContentFit::None),
        };

        // Use our forked viewer with external state control
        // scale_step is (scale_step - 1.0) because viewer uses additive step
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

        container(img_viewer)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        // Placeholder when no document is loaded
        container(text(fl!("no-document")))
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    }
}
