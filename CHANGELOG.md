# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- 新增独立的上传队列组合式函数及单元测试
- 新增运行时可配置的 Nginx 后端代理模板

### Changed

- 升级 Vue、Pinia、Vite、Biome、vue-tsc 等前端与工具链依赖
- 通过 npm 别名升级至 TypeScript 6.0.2
- 重构文件加载、上传状态、鉴权下载、批量操作和相关测试
- 简化 GitLab CI、Docker 部署和项目检查命令

### Fixed

- 将 `pnpm check` 改为只读检查，并提供显式的 `pnpm check:fix`
- 禁止 pnpm 在运行项目脚本前自动重装依赖

### Removed

- 移除已纳入版本控制的覆盖率产物
- 移除未使用的依赖、工具函数、状态模块和临时修复脚本

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
