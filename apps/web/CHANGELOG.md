# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- 视频卡片支持展示真实首帧封面，并在右下角增加半透明暗底播放微角标与悬停放大动效
- 视频播放器支持 720p Web 优化转码与原画无缝画质切换，保障弱网环境流畅播放
- 打开“新建文件夹”与“重命名”对话框时，文本输入框自动聚焦并全选所有文字，方便直接打字替换
- 大图预览支持 1600px 衍生图、LQIP 模糊占位图、相邻图片视口空闲预加载以及防白屏闪烁切换
- 新增独立的上传队列组合式函数及单元测试
- 新增运行时可配置的 Nginx 后端代理模板
- 新增移动端悬浮操作按钮（FAB）与卡片排版优化

### Changed

- 升级 Vue、Pinia、Vite、Biome、vue-tsc 等前端与工具链依赖
- 通过 npm 别名升级至 TypeScript 6.0.2
- 重构文件加载、上传状态、鉴权下载、批量操作和相关测试
- 简化 CI/CD、Docker 部署和项目检查命令

### Fixed

- 修复移动端浏览器预览大图/视频时顶部被刘海/状态栏遮挡、底部被 Safari 工具栏与 Home 手势条遮挡的问题（适配 `100dvh` 与 `env(safe-area-inset-*)` 安全区域）
- 修复移动端全量弹窗（回收站、个人中心、修改密码、头像裁剪、公开分享等）在小屏设备下的横向溢出与居中错位（宽度自适应 `calc(100vw - 24px)`）
- 修复预览大图舞台在不同纵横比下的留白过大问题（紧凑贴合）
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
