# Noctua Features

This document describes the implemented and planned features of Noctua, a modern image viewer for the COSMIC desktop environment.

## Current Features

### Document Support

#### Raster Images (Implemented)
- **Formats**: PNG, JPEG, GIF, BMP, TIFF, WebP, and all formats supported by `image-rs`
- **Capabilities**:
  - Full pixel-perfect rendering at 100% zoom
  - Lossless transformations (rotate, flip)
  - Real-time transformation preview
  - EXIF metadata extraction

#### Vector Graphics (Implemented)
- **Formats**: SVG
- **Rendering**: High-quality rendering via `resvg` library
- **Capabilities**:
  - Scalable display at any zoom level without quality loss
  - Transformations (rotate, flip)

#### Portable Documents (Implemented)
- **Formats**: PDF
- **Rendering**: Full PDF rendering via poppler library
- **Multi-page navigation**: Browse through all pages of a document
- **Page thumbnails**: Left sidebar shows page previews (generated on demand)
- **Transformations**: Rotate and flip on rendered pages

### Navigation

#### Folder Navigation (Implemented)
- **Automatic folder scanning**: When opening an image, all supported images in the same folder are indexed
- **Quick navigation**:
  - Arrow keys (Left/Right) to navigate between images
  - Footer displays current position (e.g., "3 / 42")
  - Seamless transitions between images

#### File Opening (Implemented)
- **Command-line arguments**: Open images directly from terminal
- **Default directory**: Configurable starting location (defaults to XDG Pictures)
- File dialog not yet implemented

### View Controls

#### Zoom (Implemented)
- **Mouse wheel**: Zoom in/out centered on cursor position
- **Keyboard shortcuts**:
  - `+` or `=` - Zoom in
  - `-` - Zoom out
  - `1` - Reset to 100% (Actual Size)
  - `f` - Fit to window
- **View modes**:
  - **Fit**: Automatically scales image to fit window while preserving aspect ratio
  - **Actual Size**: Displays image at 100% (1:1 pixel mapping)
  - **Custom**: Any zoom level from 10% to 2000%
- **Footer display**: Real-time zoom percentage or "Fit" indicator

#### Pan (Implemented)
- **Mouse drag**: Click and drag to pan around zoomed images
- **Keyboard shortcuts**: `Ctrl + Arrow Keys` for precise panning
- **Smart boundaries**: Pan is automatically limited to image boundaries
- **Auto-center**: Images smaller than viewport are automatically centered

#### Bidirectional State Sync (Implemented)
- Mouse interactions update keyboard/button controls
- Keyboard/button controls update mouse interaction state
- No conflicts between input methods

### Transformations

#### Image Manipulation (Implemented)
- **Rotate**:
  - `r` - Rotate 90° clockwise
  - `Shift + r` - Rotate 90° counter-clockwise
  - Toolbar buttons available
- **Flip**:
  - `h` - Flip horizontally (mirror)
  - `v` - Flip vertically
  - Toolbar buttons available
- **Lossless operations**: All transformations preserve original image quality
- **Real-time preview**: Changes are immediately visible

### User Interface

#### COSMIC Integration (Implemented)
- **Native COSMIC design**: Follows COSMIC desktop design language
- **Theme support**: Automatically adapts to system light/dark theme
- **Header toolbar**:
  - Left: Navigation controls (Previous/Next) and panel toggle
  - Center: Transformation buttons (Rotate, Flip) - horizontally centered
  - Right: Information panel toggle
- **Footer bar**:
  - Zoom controls with buttons
  - Current zoom level display
  - Image dimensions
  - Navigation position counter

#### Panels (Implemented)
- **Properties panel**:
  - Image metadata display
  - File information
  - Action buttons:
    - Set as Wallpaper (works with COSMIC, GNOME, KDE, XFCE, and tiling WMs)
    - Open With… (planned)
    - Show in Folder (planned)
  - Toggle with `i` key or toolbar button
- **Navigation panel** (Left sidebar):
  - Toggle with `n` key or toolbar button
  - For multi-page documents (PDF): Shows page thumbnails
  - Click to navigate to specific page

#### Keyboard Shortcuts (Implemented)
Full keyboard-driven workflow:
- Navigation: `←` `→`
- Zoom: `+` `-` `1` `f`
- Pan: `Ctrl + ←` `Ctrl + →` `Ctrl + ↑` `Ctrl + ↓`
- Transform: `r` `Shift+r` `h` `v`
- Panels: `i` `n`
- Actions: `w` (Set as Wallpaper)

### Desktop Integration

#### Wallpaper Support (Implemented)
- **Set as Wallpaper**: One-click wallpaper setting with cross-desktop compatibility
- **Supported desktop environments**:
  - COSMIC Desktop (direct config file integration)
  - GNOME (via gsettings)
  - KDE Plasma (via wallpaper crate)
  - XFCE (via wallpaper crate)
  - Tiling window managers (via feh)
- **Multiple access methods**:
  - Keyboard shortcut: `w`
  - Icon button in Properties panel
  - Tooltip support for discoverability
- **Automatic fallback**: Tries multiple methods until one succeeds

### Configuration

#### Persistent Settings (Implemented)
- **Panel states**: Remembers which panels were open
- **Default directory**: Customizable starting location
- **Settings location**: `~/.config/noctua/config.toml`

### Technical Features

#### Architecture (Implemented)
- **Clean separation**: View layer agnostic to document format
- **Polymorphic documents**: Single `DocumentContent` interface for all formats
- **Efficient rendering**: Leverages COSMIC's iced renderer
- **Type-safe transformations**: Compile-time guarantees for image operations

#### Performance (Implemented)
- **Lazy loading**: Images loaded on-demand
- **Efficient folder scanning**: Fast directory traversal
- **Minimal memory footprint**: Only active document kept in memory
- **Smooth zooming**: Hardware-accelerated rendering

## Planned Features

### High Priority

#### File Operations
- File dialog integration (OpenPath message prepared)
- Save transformed images
- Copy/Move/Delete operations
- Drag-and-drop support

### Medium Priority

#### Multi-format TIFF Support
- Multi-page TIFF navigation (infrastructure ready)
- Page thumbnails for TIFF (same as PDF)

#### Metadata Editing
- EXIF data modification
- Comment annotations
- Tag management

### Low Priority

#### Advanced Editing
- Crop tool (message prepared)
- Scale/Resize tool (message prepared)
- Basic color adjustments (brightness, contrast)

#### Enhanced Navigation
- Grid view for folder contents
- Quick jump to file

#### Slideshow Mode
- Auto-advance timer
- Configurable intervals
- Fullscreen support

## Feature Status Legend

- **Implemented**: Fully functional and tested
- **Planned**: Design complete, implementation pending
- **Partial**: Basic functionality exists, enhancements needed
- **In Progress**: Currently being developed
