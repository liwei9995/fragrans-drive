# Fragrans 重构优化指南

本文档基于当前主流技术栈，对 Fragrans 项目提供详细的重构与优化建议。

## 一、项目现状概览

### 1.1 技术栈（已升级）

| 技术 | 升级前 | 当前版本 | 说明 |
|------|--------|----------|------|
| NestJS | 8.x | 10.x | 核心框架 |
| TypeScript | 4.3.5 | 5.x | 类型系统与工具链 |
| Mongoose | 6.3.0 | 7.x | ODM |
| Node.js | 18 (Docker) | 20 (Docker) | 引擎版本 |
| Jest | 27 | 29 | 测试框架 |
| Husky | 7.x | 9.x | Git 钩子 |

### 1.2 架构特点

- **模块化**：Auth、Users、Storage、Config 模块划分清晰
- **认证**：JWT + Passport Local，支持角色守卫
- **存储**：本地文件系统，支持 MD5 去重、缩略图、加密存储、软删除

### 1.3 已知问题

1. **安全**：JWT secret 硬编码，CORS 使用 `*`，数据库凭证在配置文件中
2. **配置**：无 `.env` 支持，存储路径未使用 `config.storage.destination`
3. **类型**：`strictNullChecks`、`noImplicitAny` 关闭，类型安全较弱
4. **文档**：缺少 API 文档、架构说明、贡献指南
5. **依赖**：多处版本过旧，存在安全与兼容性风险

---

## 二、重构优化建议（按优先级）

### 2.1 高优先级：安全与配置

#### 2.1.1 JWT Secret 环境变量化

**现状**：`src/auth/constants.ts` 与 `src/storage/constants.ts` 中硬编码 secret。

**建议**：

```typescript
// 使用 process.env.JWT_SECRET，fallback 仅用于开发
export const jwtConstants = {
  secret: process.env.JWT_SECRET || 'dev-only-fallback',
  expiresIn: process.env.JWT_EXPIRES_IN || '6000s',
}
```

生产环境必须通过环境变量注入，禁止使用 fallback。

#### 2.1.2 CORS 收紧

**现状**：`app.enableCors({ origin: '*' })`。

**建议**：生产环境使用白名单：

```typescript
app.enableCors({
  origin: process.env.CORS_ORIGINS?.split(',') || ['http://localhost:3847'],
  credentials: true,
})
```

#### 2.1.3 数据库凭证环境变量化

**现状**：`config/default.json` 与 `config/production.json` 中明文存储。

**建议**：支持环境变量覆盖，例如：

- `MONGO_URI` 或 `MONGO_USERNAME`、`MONGO_PASSWORD`、`MONGO_HOST`、`MONGO_PORT`、`MONGO_DATABASE`

#### 2.1.4 引入 @nestjs/config 与 .env

**建议**：

- 使用 `@nestjs/config` 加载 `.env`
- 配置优先级：环境变量 > .env > config/*.json

### 2.2 高优先级：存储路径与配置

#### 2.2.1 LocalStorage 使用配置路径

**现状**：`storage.local.ts` 使用 `join(__dirname, '../../bucket/storage')`，未使用 `config.storage.destination`。

**建议**：在 StorageModule 中注入 ConfigService，将 `destination` 传给 LocalStorage 构造函数。

### 2.3 中优先级：依赖升级

#### 2.3.1 升级路径建议

| 阶段 | 目标 | 说明 |
|------|------|------|
| 第一阶段 | NestJS 10 | 避开 Express v5 的破坏性变更 |
| 第一阶段 | TypeScript 5.x | 提升类型与工具链 |
| 第一阶段 | Mongoose 8.x | 与 @nestjs/mongoose 9 兼容 |
| 第二阶段 | NestJS 11 | 需适配 Express v5 路由匹配 |
| 第二阶段 | Mongoose 9.x | 可选，功能更丰富 |

#### 2.3.2 升级注意事项

- **mongoose-paginate**：v5 与 Mongoose 8 兼容性需验证，必要时考虑 `mongoose-aggregate-paginate-v2` 或自实现分页
- **NestJS 11**：需 Node 20+，Express v5 路由需使用 `/*splat` 等命名通配符
- **TypeScript 5**：建议逐步开启 `strictNullChecks`、`noImplicitAny`

### 2.4 中优先级：代码质量

#### 2.4.1 TypeScript 严格模式

**建议**：分步开启：

```json
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "strictNullChecks": true,
    "noImplicitAny": true,
    "forceConsistentCasingInFileNames": true
  }
}
```

可先开启 `strictNullChecks`，逐个模块修复后再开启其他选项。

#### 2.4.2 JWT 配置统一

**现状**：AuthModule 与 StorageModule 各自注册 JwtModule，配置重复。

**建议**：在 AppModule 或共享模块中统一注册 JwtModule，通过 `JwtModule.registerAsync` 从 ConfigService 读取。

#### 2.4.3 Storage store 类型与调用

**现状**：`store(files, userId, parentId)` 中 `files` 类型与 controller 传入的 `Array<Express.Multer.File>` 需对齐。

**建议**：明确 DTO 与类型定义，避免 `any`。

### 2.5 中优先级：API 文档

#### 2.5.1 Swagger/OpenAPI

**建议**：添加 `@nestjs/swagger`，为所有 Controller 补充装饰器：

```typescript
// main.ts
const config = new DocumentBuilder()
  .setTitle('Fragrans API')
  .setVersion('1.0')
  .addBearerAuth()
  .build()
const document = SwaggerModule.createDocument(app, config)
SwaggerModule.setup('api', app, document)
```

### 2.6 低优先级：可观测性与运维

#### 2.6.1 健康检查

**建议**：添加 `@nestjs/terminus`，提供 `/health` 接口，用于负载均衡与容器探针。

#### 2.6.2 结构化日志

**建议**：使用 `pino` 或 `winston` 替代 `Logger`，输出 JSON 便于集中收集。

#### 2.6.3 Docker 优化

- 使用 `healthcheck` 指令
- 生产环境去掉 `privileged: true`
- 使用 `node:25-alpine` 作为基础镜像

### 2.7 低优先级：测试

- 增加单元测试覆盖率
- E2E 覆盖核心 API（上传、下载、列表、权限）

---

## 三、开发与发布体验优化

### 3.1 脚本与工具

- 增加 `pnpm dev`、`pnpm build`、`pnpm start:prod` 等别名
- 增加 `pnpm db:up`、`pnpm db:down` 管理 MongoDB 容器
- 增加 `prepare` 脚本以初始化 Husky

### 3.2 Husky 9

Husky 9 使用 `.husky/` 目录，需执行 `pnpm exec husky init` 并迁移钩子。

### 3.3 环境变量模板

提供 `.env.example`，列出所有可配置项及说明。

---

## 四、文档完善建议

| 文档 | 内容 |
|------|------|
| README.md | 项目简介、快速开始、环境要求、pnpm 用法、Logo 路径修正 |
| docs/ARCHITECTURE.md | 模块划分、数据流、存储设计 |
| docs/DEVELOPMENT.md | 本地开发、调试、测试、代码规范 |
| docs/DEPLOYMENT.md | Docker 部署、环境变量、生产配置 |
| docs/API.md | API 概览（或链接到 Swagger） |
| CONTRIBUTING.md | 贡献流程、Pull Request 规范 |

---

## 五、实施路线图

建议按以下顺序执行：

1. **阶段 1（安全与配置）**：JWT 环境变量、CORS、.env、存储路径配置
2. **阶段 2（依赖升级）**：NestJS 10、TypeScript 5、Mongoose 8
3. **阶段 3（文档与体验）**：Swagger、文档完善、脚本优化
4. **阶段 4（可选）**：类型严格化、健康检查、日志优化

---

## 六、参考资源

- [NestJS Migration Guide](https://docs.nestjs.com/migration-guide)
- [Mongoose Version Support](https://mongoosejs.com/docs/version-support.html)
- [Express v5 Migration](https://expressjs.com/en/guide/migrating-5.html)
