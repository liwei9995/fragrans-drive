# Fragrans

<p align="center">
  A <a href="http://nodejs.org" target="_blank">Node.js</a> project based on <a href="https://github.com/nestjs/nest" target="_blank">Nest</a> for building file storage service.
</p>

## Description

Osmanthus fragrans Lour is my favorite flower, so I named this project fragrans. In my opinion, a distributed file storage system is composed of osmanthus-like files scattered on the ground.

Fragrans aims to provide users the ability to deploy their own file storage service with efficiency and scalability.

## Features

- 用户认证（JWT + Passport）
- 文件上传 / 下载 / 列表 / 移动 / 软删除
- MD5 去重、缩略图生成、加密存储
- Swagger API 文档
- Docker 部署支持

## Requirements

- Node.js >= 18
- MongoDB 5.x / 6.x / 7.x
- pnpm（推荐）或 npm

## Quick Start

```bash
# 1. 启动 MongoDB
pnpm db:up
# 或: docker-compose -f docker-compose.develop.yaml up -d

# 2. 安装依赖
pnpm install

# 3. 开发模式
pnpm dev
```

服务运行在 http://localhost:3847，API 文档在 http://localhost:3847/api。

## Scripts

| 命令 | 说明 |
|------|------|
| `pnpm dev` | 开发模式（watch） |
| `pnpm build` | 构建 |
| `pnpm start:prod` | 生产模式 |
| `pnpm lint` | 代码检查 |
| `pnpm test` | 单元测试 |
| `pnpm test:e2e` | E2E 测试 |
| `pnpm db:up` | 启动 MongoDB |
| `pnpm db:down` | 停止 MongoDB |

## Configuration

复制 `.env.example` 为 `.env` 并配置环境变量。生产环境必须设置 `JWT_SECRET`。

详见 [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)。

## Deployment

```bash
docker-compose build
docker-compose up -d
```

详见 [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)。

## Documentation

- [架构说明](docs/ARCHITECTURE.md)
- [开发指南](docs/DEVELOPMENT.md)
- [部署指南](docs/DEPLOYMENT.md)
- [重构优化建议](docs/REFACTORING_GUIDE.md)

## Stay in touch

- Author - [Aaron Li](https://www.oyiyio.com)

## License

Fragrans is [MIT licensed](LICENSE).
