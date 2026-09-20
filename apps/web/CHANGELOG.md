# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support displaying genuine first-frame video cover thumbnails on storage cards, with a semi-transparent dark play badge and hover scale effect
- Support 720p web-optimized transcoding and seamless quality switching in the video player for smooth playback in low-bandwidth conditions
- Automatically focus and select all text in input boxes when opening "New Folder" and "Rename" dialogs for instant editing
- Support high-performance image preview with 1600px derivatives, LQIP blur placeholders, idle neighbor preloading, and flash-free switching
- Independent upload queue composable function and comprehensive unit tests
- Runtime-configurable Nginx backend proxy template
- Mobile floating action button (FAB) and card typography optimizations

### Changed

- Upgrade frontend and toolchain dependencies including Vue, Pinia, Vite, Biome, and vue-tsc
- Upgrade to TypeScript 6.0.2 via npm alias
- Refactor file fetching, upload status, authenticated downloads, batch operations, and related tests
- Simplify CI/CD, Docker deployment, and project check commands

### Fixed

- Fix mobile browser preview header and bottom toolbar obstruction by the notch/status bar and Safari bar / iOS Home bar (adapted with `100dvh` and `env(safe-area-inset-*)`)
- Fix horizontal overflow and center misalignment of all modal dialogs (trash, profile, change password, avatar crop, public link) on small screens (auto-adapted width `calc(100vw - 24px)`)
- Fix oversized blank margins on image preview stage across different aspect ratios (shrinkwrap container)
- Change `pnpm check` to a read-only check and provide explicit `pnpm check:fix`
- Prevent pnpm from automatically reinstalling dependencies before running project scripts

### Removed

- Remove coverage artifacts checked into version control
- Remove unused dependencies, utility functions, state modules, and temporary fix scripts

## [0.3.0] - 2026-03-13

### Fixed

- Fix issue where "Uploading" status remained in the bottom right corner after files were successfully uploaded
- Fix Upload component prop name mismatch causing `onChange` callback not being properly passed (ActionButton, Empty, EmptyItem)
- Explicitly call `onUploadChange` via `onSuccess`/`onError` in the Upload wrapper to ensure accurate upload completion notifications
- Fix issue where GlobalDropzone using `v-if` caused premature component unmount and lost drag-and-drop success callbacks (changed to `v-show`)
- Fix `CanceledError` caused by duplicate `fetchFiles` calls, adding `uploadCleanedUp` guard and silent handling for `axios.isCancel`

### Changed

- Catch and silently handle Axios `CanceledError` in `useFetchFiles`

[0.3.0]: https://github.com/liwei9995/fragrans-drive/compare/v0.2.0...v0.3.0
