# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-06-11

### Fixed

- Clone derive to FIGfont

## [0.3.0] - 2026-06-10

### Added

- Crate-level documentation

### Changed

- Exposed `from_reader` method on font loader
- API cleanup: made `flc` module private

## [0.2.0] - 2026-06-09

### Added

- `figlist` and `showfigfonts` CLI subcommands for font discovery and inspection
- ISO-2022 encoding support via decoder infrastructure
- Decoder infrastructure for font encoding handling
- Terminal width auto-detection with `-t` flag
- ZIP-compressed font (`.flc`) file support
- Control file (`.flc`) support for font overrides
- `--layout-mode` / `-m` CLI option for layout mode selection
- `-v` / `--version` flag
- `-n` / `--normal` flag for explicit normal newline mode
- `-s` and `-S` smushing mode flags
- `-l` / `--left`, `-x`, `-L`, `-X` text direction and justification options
- Infocode option
- Right-to-left printing support
- Octal code tag support in figfont module
- Overlap spacing option

### Changed

- Reworked stdin handling to decode input before rendering
- Redesigned API to remove spec implementation leaks
- Moved paragraph logic to library crate
- Unified input handling across CLI and library paths
- Moved CLI error type from library to binary crate
- Introduced `cli` feature flag for optional CLI dependencies
- Renamed control pipeline struct for clarity
- Improved TLV font support
- Documented public API types, fields, and constants
- CI expanded to test on macOS and Windows
- Adjusted help text alignment and long option formatting

### Fixed

- Preserved decoder state across lines for multi-line decoding
- Corrected paragraph mode trailing space for all input paths
- Fixed stdin processing to use line-by-line `read_until` with proper EOL handling
- Preserved hardblank-derived trailing spaces in wrapper output
- Adjusted center alignment to match reference figlet behavior
- Corrected character-level wrapping to match reference figlet
- Adjusted newline flush to match reference figlet behavior
- Rejected invalid CLI flags with proper error messages
- Implemented last-flag-wins behavior for paragraph and alignment options
- Properly smushed hardblank characters
- Handled explicit newlines in input for wrapper
- Handled spaces at wrap points in wrapper
- Fixed write_line API and removed unnecessary return value
- Fixed negative character code handling in figfont
- Gracefully handled missing optional headers in figfont
- Fixed character code 214 loading
- Fixed UTF-8 character smushing
- Fixed output alignment for center and right modes
