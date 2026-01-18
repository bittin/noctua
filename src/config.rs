// SPDX-License-Identifier: GPL-3.0-or-later
// src/config.rs
//
// Global configuration for the application with cosmic-config support.

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use std::path::PathBuf;

/// Global configuration for the application.
#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 1]
pub struct AppConfig {
    /// Optional default directory to open images from.
    pub default_image_dir: Option<PathBuf>,
    /// Whether the nav bar (left panel) is visible.
    pub nav_bar_visible: bool,
    /// Whether the context drawer (right panel) is visible.
    pub context_drawer_visible: bool,
    /// Scale step factor for keyboard zoom (e.g., 1.1 = 10% per step).
    pub scale_step: f32,
    /// Pan step size in pixels per key press.
    pub pan_step: f32,
    /// Minimum zoom scale (e.g., 0.1 = 10%).
    pub min_scale: f32,
    /// Maximum zoom scale (e.g., 20.0 = 2000%).
    pub max_scale: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_image_dir: dirs::picture_dir().or_else(dirs::home_dir),
            nav_bar_visible: false,
            context_drawer_visible: false,
            scale_step: 1.1,
            pan_step: 50.0,
            min_scale: 0.1,
            max_scale: 8.0,
        }
    }
}
