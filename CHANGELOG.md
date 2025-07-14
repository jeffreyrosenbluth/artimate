# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-14

### Added
- Initial release of Artimate
- Simple pixel-based graphics framework for creative coding
- Two application modes: `SketchMode` for simple graphics and `AppMode` for stateful applications
- Built-in window management with `winit` integration
- GPU-accelerated rendering using the `pixels` crate (v0.15.0)
- Mouse and keyboard input handling
- Automatic frame saving to PNG files
- Configuration system for window size, title, and rendering options
- Comprehensive documentation with examples
- Support for both simple sketches and complex stateful applications

### Features
- `App::sketch()` - Create simple applications with just a draw function
- `App::app()` - Create stateful applications with model, update, and draw functions
- `Config` struct for customizing window and rendering behavior
- Mouse position tracking (`mouse_x()`, `mouse_y()`)
- Keyboard event handlers (`on_key_press()`, `on_key_release()`, `on_key_held()`)
- Mouse button event handlers (`on_mouse_press()`)
- Frame limiting and single-frame rendering options
- Automatic performance statistics reporting
- Integration-friendly pixel buffer API

### Dependencies
- `delegate` 0.13.3 - Method delegation
- `dirs` 6.0 - Directory path utilities
- `pixels` 0.15.0 - GPU-accelerated pixel buffer
- `png` 0.17.16 - PNG image encoding
- `winit` 0.30.11 - Window creation and event handling

[Unreleased]: https://github.com/jeffreyrosenbluth/artimate/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jeffreyrosenbluth/artimate/releases/tag/v0.1.0