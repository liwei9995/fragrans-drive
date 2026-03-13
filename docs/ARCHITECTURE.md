# Fragrans 架构说明

## 概述

Fragrans 是一个基于 NestJS 的自托管文件存储服务，提供用户认证、文件上传/下载、缩略图生成、MD5 去重和加密存储能力。

## 模块划分

```
src/
├── auth/          # 认证模块：JWT、Passport Local
├── common/        # 公共：装饰器、守卫、拦截器、过滤器
├── config/        # 配置：封装 node-config
├── storage/       # 存储核心：文件 CRUD、本地存储实现
├── users/         # 用户模块：用户 CRUD
├── utils/         # 工具：加密、MD5、流判断
├── app.module.ts  # 根模块
└── main.ts        # 入口
```

### 模块职责

| 模块 | 职责 |
|------|------|
| **AuthModule** | 登录、JWT 签发、Local/JWT 策略 |
| **UsersModule** | 用户增删改查、密码哈希 |
| **StorageModule** | 文件上传/下载、文件夹、列表、移动、软删除 |
| **ConfigModule** | 统一配置读取（config 包 + 环境变量） |

## 数据流

### 认证流程

1. 用户调用 `POST /v1/auth/login`，传入 username/password
2. LocalStrategy 校验用户，AuthService 签发 JWT
3. 后续请求在 Header 中携带 `Authorization: Bearer <token>`
4. JwtAuthGuard 校验 token，RolesGuard 校验角色

### 文件上传流程

1. 客户端 `POST /v1/storage/upload`，携带 `multipart/form-data`
2. StorageService.store() 计算 MD5，检查是否已存在（去重）
3. 若为新文件，写入 LocalStorage（目录分片 + 可选加密）
4. 若为图片，异步生成缩略图并存储
5. 返回文件 ID 列表

### 文件下载流程

1. 客户端 `GET /v1/storage/:id?token=xxx`（公开路由，需 token 鉴权）
2. JwtService 解码 token 获取 userId
3. StorageService.getFile() 校验归属，从 LocalStorage 读取流
4. 返回 StreamableFile

## 存储设计

### 本地存储（LocalStorage）

- **路径分片**：按 MD5 前 6 位分目录，避免单目录文件过多
- **加密**：可选 AES 加密，iv 存于 MongoDB
- **缩略图**：图片自动生成缩略图，单独存储并关联

### 数据库（MongoDB）

- **users**：用户表
- **storages**：文件元数据（name、MD5Hash、parentId、userId、trashed 等）

## 安全设计

- **JWT**：secret 与过期时间通过环境变量配置
- **CORS**：生产环境建议通过 `CORS_ORIGINS` 限制来源
- **Helmet**：安全头
- **密码**：bcrypt 哈希

## 扩展点

- **存储后端**：可替换 `storage.local.ts` 为 S3、OSS 等实现
- **认证**：可扩展 OAuth、SSO 等策略
