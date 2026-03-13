# Fragrans 部署指南

## Docker 部署（推荐）

### 一键部署

```bash
docker-compose build
docker-compose up -d
```

服务端口映射：

- **应用**：8085 → 3847
- **MongoDB**：25018 → 27017
- **Mongo-Express**：8084 → 8081（管理界面）

### 环境变量

可通过 `docker-compose.yaml` 的 `environment` 或 `.env` 文件配置：

| 变量 | 说明 | 示例 |
|------|------|------|
| `JWT_SECRET` | JWT 密钥（必填） | 随机 32 位字符串 |
| `JWT_EXPIRES_IN` | Token 过期时间 | 6000s |
| `MONGO_URI` | MongoDB 连接串 | mongodb://user:pass@mongo:27017/db?authSource=admin |
| `CORS_ORIGINS` | 允许的 CORS 来源 | https://your-domain.com |
| `STORAGE_DESTINATION` | 存储路径 | /app/bucket/storage |
| `HTTP_PORT` | 服务端口 | 3847 |

### 数据持久化

- `storage-files`：文件存储卷
- `storage-db`：MongoDB 数据卷

### 生产建议

1. **JWT_SECRET**：使用强随机字符串，切勿使用默认值
2. **CORS_ORIGINS**：限制为实际前端域名
3. **MongoDB**：使用独立实例，配置认证与备份
4. **去掉 privileged**：生产环境移除 `privileged: true`
5. **健康检查**：可添加 `healthcheck` 指令供负载均衡使用

## 传统部署

### 构建

```bash
pnpm install --frozen-lockfile
pnpm build
```

### 运行

```bash
NODE_ENV=production \
  JWT_SECRET=your-secret \
  MONGO_URI=mongodb://... \
  node dist/main.js
```

或使用 PM2：

```bash
pm2 start dist/main.js --name fragrans
```

### 反向代理（Nginx）

```nginx
location / {
  proxy_pass http://127.0.0.1:3847;
  proxy_http_version 1.1;
  proxy_set_header Upgrade $http_upgrade;
  proxy_set_header Connection 'upgrade';
  proxy_set_header Host $host;
  proxy_cache_bypass $http_upgrade;
}
```

## 健康检查

可访问根路径 `/` 或后续添加的 `/health` 接口，用于负载均衡探针。

## 升级

1. 拉取新代码
2. `pnpm install --frozen-lockfile`
3. `pnpm build`
4. 重启服务（如 `docker-compose up -d --build` 或 `pm2 restart fragrans`）
