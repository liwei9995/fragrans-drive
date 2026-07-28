<p align="center">
  <a href="https://www.oyiyio.com/" rel="noopener" target="_blank">
    <img width="150" src="./public/logo.svg" alt="Fragrans Drive" />
  </a>
</p>

<h1 align="center">Fragrans Drive</h1>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.3.0-blue" alt="version 0.3.0" />
</p>

Fragrans Drive 是一款基于 Vue 3、Element Plus、Pinia 和 TypeScript 的个人云盘前端，支持私有部署。

> 项目仍在开发中。本仓库只包含前端，使用前需要先部署
> [Fragrans 后端](https://github.com/liwei9995/fragrans)。

## 功能

- 邮箱登录和本地持久化登录状态
- 文件夹浏览、分页加载和面包屑导航
- 新建文件夹、重命名、移动、删除和批量操作
- 文件选择及拖拽上传，支持进度和结果提示
- 图片、视频预览和带鉴权的文件下载
- 桌面端和移动端响应式界面

可观看 [云盘介绍视频](https://www.youtube.com/embed/Uzeur9v44LE) 了解基本功能。

## 技术栈

| 分类 | 技术 |
| --- | --- |
| 应用 | Vue 3.5、Vue Router 5、Pinia 4、Element Plus 2.14 |
| 构建 | Vite 8、TypeScript 6 |
| 代码质量 | Biome 2、vue-tsc 3 |
| 测试 | Vitest 4、Vue Test Utils、Playwright |
| 部署 | Docker、Nginx、GitLab CI |

TypeScript 通过 npm 别名
`typescript -> @typescript/typescript6@6.0.2` 安装，因此 `vue-tsc`、Vite
和编辑器仍可按标准的 `typescript` 包名解析它。`vue-tsc` 用于检查 `.vue`
单文件组件模板和脚本中的 TypeScript 类型。

## 环境要求

- Node.js 22 或更高版本；CI 和 Docker 当前使用 Node.js 24
- pnpm 11.15.1，推荐通过 Corepack 使用项目声明的版本
- Fragrans 后端，开发环境默认监听 `127.0.0.1:3821`
- Docker（仅容器部署需要）

## 本地开发

```bash
corepack enable
pnpm install --frozen-lockfile
pnpm dev
```

开发服务器默认运行在 <http://localhost:5173>。

前端请求使用 `VITE_API_URL=/api`。开发环境由
[Vite 配置](./vite.config.ts) 将 `/api` 转发到
`http://127.0.0.1:3821`，并在转发时移除 `/api` 前缀。后端地址不同时，
请修改 `vite.config.ts` 中的 `server.proxy['/api'].target`。

## 常用命令

| 命令 | 用途 |
| --- | --- |
| `pnpm dev` | 启动开发服务器 |
| `pnpm build` | 类型检查并构建生产资源 |
| `pnpm preview` | 本地预览构建结果 |
| `pnpm check` | 只读运行 Biome 检查 |
| `pnpm check:fix` | 自动修复 Biome 可修复问题 |
| `pnpm check:types` | 使用 `vue-tsc` 检查 Vue 和 TypeScript 类型 |
| `pnpm check:all` | 顺序运行 Biome、类型检查和单元测试 |
| `pnpm test` | 运行单元测试 |
| `pnpm coverage` | 运行单元测试并生成覆盖率报告 |
| `pnpm test:e2e` | 在 Chromium、Firefox 和 WebKit 中运行端到端测试 |
| `pnpm verify` | 运行 CI 使用的完整验证流程 |

首次运行端到端测试前需要安装浏览器：

```bash
pnpm exec playwright install
pnpm test:e2e
```

单元测试覆盖率门槛为：语句和行 90%、分支 88%、函数 85%。生成的
`coverage/`、`playwright-report/` 和 `test-results/` 不纳入版本控制。

## Docker 部署

部署脚本支持 macOS 和 Linux，会构建镜像、替换同名容器并将服务暴露在
<http://localhost:8061>：

```bash
bash deploy/helper.sh
```

容器内 Nginx 默认将 `/api` 转发到宿主机的 `3821` 端口。可通过
`API_UPSTREAM` 指定其他后端地址：

```bash
API_UPSTREAM=http://192.168.1.10:3821 bash deploy/helper.sh
```

`API_UPSTREAM` 在容器启动时注入，不需要为不同环境重新构建前端镜像。

## CI

GitLab CI 使用 Node.js 24，通过冻结的 `pnpm-lock.yaml` 安装依赖后运行
`pnpm verify`。该流程包含 Biome lint、Vue/TypeScript 类型检查、带覆盖率
门槛的单元测试和生产构建。

## 项目结构

```text
src/
├── api/          # Axios 客户端、接口类型和 API 模块
├── components/   # 通用文件卡片、列表项和媒体预览组件
├── hooks/        # 文件加载、文件夹创建和上传队列逻辑
├── routers/      # 页面路由和错误页
├── store/        # Pinia 全局状态
├── utils/        # 存储 URL 和缩略图工具
└── views/        # 登录页和云盘首页
e2e/              # Playwright 端到端测试
deploy/           # Docker 启动脚本和 Nginx 配置
```
