# Fragrans

<p align="center">
  A high-performance file storage service rewritten in <b>Rust</b>.
</p>

## 📖 Description

**Fragrans** (Osmanthus fragrans) 旨在提供一个高效、安全且可扩展的个人文件存储解决方案。本项目已从原有的 Node.js/NestJS 架构完整重写为 Rust，以获得更低的内存占用、更高的并发处理能力以及更小的部署体积。

> "暗淡轻黄体性柔，情疏迹远只香留。" —— 正如桂花般，Fragrans 追求在后台默默且高效地处理您的每一份文件。

## ✨ 特性 (Features)

- **高性能底层**：基于 Rust + Axum + Tokio，充分利用异步 I/O。
- **存储优化**：
  - **MD5 去重**：相同内容的文件在物理存储上仅保留一份。
  - **文件分片**：采用 `aa/bb/cc/hash` 的分层存储结构，避免单目录文件过多。
  - **加密存储**：所有物理文件均采用 AES-256-CTR 加密，确保数据落地安全。
- **图像处理**：自动为上传的图片生成 WebP/JPEG 缩略图。
- **安全认证**：基于 JWT 的身份验证机制。
- **极简部署**：提供多阶段构建的 Docker 镜像，体积仅约 30MB。

## 🛠️ 技术栈 (Tech Stack)

- **语言**: Rust
- **Web 框架**: [Axum](https://github.com/tokio-rs/axum)
- **异步运行时**: [Tokio](https://tokio.rs/)
- **数据库**: MongoDB (Official Rust Driver)
- **加密与哈希**: `bcrypt`, `aes`, `ctr`, `md5`
- **图像处理**: `image` crate

## 🚀 快速开始 (Quick Start)

### 环境要求

- Rust 工具链 (1.75+)
- MongoDB (推荐 5.0+)

### 本地开发

1. **配置环境变量**
   复制 `.env.example` 为 `.env` 并根据需要修改：

   ```bash
   cp .env.example .env
   ```

2. **启动数据库**
   使用现有的 Docker Compose 启动 MongoDB：

   ```bash
   docker-compose up -d mongo
   ```

3. **运行项目**
   ```bash
   cargo run
   ```
   服务默认监听端口：`3821`

### 运行测试

```bash
# 运行所有单元测试 (待完善)
cargo test
# 检查代码格式
cargo fmt --all -- --check
# 运行 Lint 检查
cargo clippy
```

## 🌐 API 接口文档 (API Documentation)

目前 API 遵循 RESTful 规范，基础版本为 `/v1`。

### 访问方式

本项目通过 **utoipa** 自动生成 OpenAPI 接口文档，您可以通过交互式 UI 直接测试接口：

- **Swagger UI**: [http://localhost:3821/swagger-ui](http://localhost:3821/swagger-ui)
- **OpenAPI JSON**: `/api-docs/openapi.json`

主要接口概览：

| 模块        | 路径                 | 方法     | 描述                       |
| ----------- | -------------------- | -------- | -------------------------- |
| **Auth**    | `/v1/auth/login`     | POST     | 用户登录 (获取 Token)      |
| **Users**   | `/v1/users`          | GET/POST | 用户管理 (需 Token)        |
| **Storage** | `/v1/storage/upload` | POST     | 文件上传 (需 Token)        |
| **Storage** | `/v1/storage/list`   | POST     | 获取文件列表 (需 Token)    |
| **Storage** | `/v1/storage/{id}`   | GET      | 文件下载 (支持 token 验证) |

*提示：在 Swagger UI 中点击 "Authorize" 并输入 Bearer Token 即可测试加密接口。*

## 📦 部署 (Deployment)

### Docker 部署

本项目提供优化后的多阶段构建 Dockerfile：

```bash
# 构建镜像
docker build -t fragrans-rust .

# 使用 Docker Compose 一键启动
docker-compose up -d
```

对外映射端口默认为 `8085`。

## 📂 目录结构

- `src/api/`: API 路由处理器与中间件。
- `src/domain/`: 领域模型 (User, Storage)。
- `src/infrastructure/`: 外部服务实现 (DB, Storage I/O, Image processing)。
- `src/config/`: 配置加载逻辑。
- `src/utils/`: 通用工具类 (加密、哈希)。

## 🤝 贡献与支持

- **作者**: [Aaron Li](https://www.oyiyio.com)
- **许可证**: MIT
