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
use crate::fl;

/// Render the center canvas area with the current document.
pub fn view(model: &AppModel) -> Element<'_, AppMessage> {
    if let Some(doc) = &model.document {
        let handle = doc.handle();

        // Determine zoom scale and content fit based on view mode
        let (scale, content_fit) = match model.view_mode {
            ViewMode::Fit => (1.0, ContentFit::Contain),
            ViewMode::ActualSize => (1.0, ContentFit::None),
            ViewMode::Custom(z) => (z, ContentFit::None),
        };

        // Use our forked viewer with external state control
        let img_viewer = Viewer::new(handle)
            .with_state(scale, model.pan_x, model.pan_y)
            .on_state_change(|scale, offset_x, offset_y| {
                AppMessage::ViewerStateChanged {
                    scale,
                    offset_x,
                    offset_y,
                }
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(content_fit)
            .min_scale(0.1)
            .max_scale(20.0)
            .scale_step(0.1);

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
