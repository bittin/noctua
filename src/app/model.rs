// SPDX-License-Identifier: GPL-3.0-or-later
// src/app/model.rs
//
// Application state.

use std::path::PathBuf;

use crate::app::document::meta::DocumentMeta;
use crate::app::document::DocumentContent;
use crate::app::view::crop::CropSelection;
use crate::config::AppConfig;

// =============================================================================
// Enums
// =============================================================================

#[derive(Debug, Clone, Copy)]
pub enum ViewMode {
    Fit,
    ActualSize,
    Custom(f32),
}

impl ViewMode {
    pub fn zoom_factor(&self) -> Option<f32> {
        match self {
            ViewMode::Fit => None,
            ViewMode::ActualSize => Some(1.0),
            ViewMode::Custom(z) => Some(*z),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    None,
    Crop,
    Scale,
}

// =============================================================================
// Model
// =============================================================================

pub struct AppModel {
    // Document.
    pub document: Option<DocumentContent>,
    pub metadata: Option<DocumentMeta>,
    pub current_path: Option<PathBuf>,

    // Navigation.
    pub folder_entries: Vec<PathBuf>,
    pub current_index: Option<usize>,

    // View.
    pub view_mode: ViewMode,
    pub pan_x: f32,
    pub pan_y: f32,

    // Tools.
    pub tool_mode: ToolMode,
    pub crop_selection: CropSelection,

    // UI state.
    pub error: Option<String>,
    pub tick: u64,
}

impl AppModel {
    pub fn new(_config: AppConfig) -> Self {
        Self {
            document: None,
            metadata: None,
            current_path: None,
            folder_entries: Vec::new(),
            current_index: None,
            view_mode: ViewMode::Fit,
            pan_x: 0.0,
            pan_y: 0.0,
            tool_mode: ToolMode::None,
            crop_selection: CropSelection::default(),
            error: None,
            tick: 0,
        }
    }

    pub fn set_error<S: Into<String>>(&mut self, msg: S) {
        self.error = Some(msg.into());
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn reset_pan(&mut self) {
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    pub fn zoom_factor(&self) -> Option<f32> {
        self.view_mode.zoom_factor()
    }
}
