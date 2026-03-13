# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-03-13

### Fixed

- 修复文件上传成功后右下角仍显示「正在上传」的问题
- 修复 Upload 组件属性名不匹配导致 `onChange` 回调未正确传递（ActionButton、Empty、EmptyItem）
- 在 Upload 包装器中添加 `onSuccess`/`onError` 显式调用 `onUploadChange`，确保上传完成时通知正确更新
- 修复拖拽上传时 GlobalDropzone 使用 `v-if` 导致组件过早销毁、成功回调无法触发的问题（改为 `v-show`）
- 修复 `fetchFiles` 重复调用引发的 `CanceledError`，增加 `uploadCleanedUp` 防重复及 `axios.isCancel` 静默处理

### Changed

- `useFetchFiles` 中捕获并静默处理 Axios 的 `CanceledError`

[0.3.0]: https://github.com/liwei9995/fragrans-drive/compare/v0.2.0...v0.3.0
