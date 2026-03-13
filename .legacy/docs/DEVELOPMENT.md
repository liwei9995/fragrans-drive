# Fragrans 开发指南

## 环境要求

- **Node.js** >= 18
- **pnpm** >= 8（推荐）
- **MongoDB** 5.x / 6.x / 7.x
- **Docker**（可选，用于本地 MongoDB）

## 快速开始

```bash
# 1. 克隆并安装
git clone <repo>
cd fragrans
pnpm install

# 2. 启动 MongoDB（Docker）
pnpm db:up

# 3. 复制环境变量（可选）
cp .env.example .env

# 4. 开发模式
pnpm dev
# 或
pnpm start:dev
```

服务默认运行在 http://localhost:3847，Swagger 文档在 http://localhost:3847/api。

## 常用脚本

| 脚本 | 说明 |
|------|------|
| `pnpm dev` | 开发模式（watch） |
| `pnpm build` | 构建 |
| `pnpm start:prod` | 生产模式运行 |
| `pnpm lint` | ESLint 检查并修复 |
| `pnpm format` | Prettier 格式化 |
| `pnpm test` | 单元测试 |
| `pnpm test:e2e` | E2E 测试 |
| `pnpm db:up` | 启动开发用 MongoDB |
| `pnpm db:down` | 停止 MongoDB |

## 配置

- **config/default.json**：默认配置（本地开发）
- **config/production.json**：生产配置（Docker）
- **config/test.json**：测试配置
- **.env**：环境变量覆盖（见 .env.example）

环境变量优先级高于 config 文件。

## 代码规范

- **ESLint** + **Prettier**：提交前自动格式化（lint-staged）
- **Commitlint**：提交信息需符合 Angular 规范（husky commit-msg）

示例提交：

```
feat(storage): add thumbnail support
fix(auth): correct JWT expiry check
```

## 调试

```bash
pnpm start:debug
```

使用 VS Code 时，可配置 launch.json 附加到进程。

## 测试

- **单元测试**：`src/**/*.spec.ts`
- **E2E 测试**：`test/*.e2e-spec.ts`，需 MongoDB 运行

```bash
pnpm db:up
pnpm test:e2e
```

## 目录约定

- `src/`：业务代码
- `config/`：环境配置
- `test/`：E2E 测试
- `docs/`：文档

## 常见问题

### MongoDB 连接失败

确认 MongoDB 已启动，且 `config/default.json` 或 `MONGO_URI` 配置正确。

### JWT 校验失败

检查 `JWT_SECRET` 是否与签发时一致，`JWT_EXPIRES_IN` 是否过期。

### 存储路径权限

确保应用对 `STORAGE_DESTINATION` 或默认 `bucket/storage` 有读写权限。
