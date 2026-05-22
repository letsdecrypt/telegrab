# telegrab

抓取 [telegra.ph](https://telegra.ph) 页面中的图片，下载并归档为 CBZ（Comic Book Zip）格式。

参考项目：[Telegraph-Image-Downloader](https://github.com/Artezon/Telegraph-Image-Downloader)

## 功能

- HTML 页面解析，提取标题、日期、图片链接
- 批量下载图片
- 归档为 CBZ 格式，内含 `ComicInfo.xml`，兼容 Komga
- 标签系统：自动从标题提取作者、社团、原作等标签
- 文件系统监控：自动发现新增/删除的 CBZ 文件
- 异步任务队列，支持多 Worker 并行处理
- GraphQL API（async-graphql）+ REST API（axum）

## 项目结构

```
crates/
  telegrab-model/     数据模型（entity + dto）
  telegrab-db/        数据库访问层（repository + migrations）
  telegrab-core/      核心业务逻辑（service + graceful shutdown）
  telegrab/           入口 + HTTP/GraphQL 层
```

## API 端点

| 路径 | 说明 |
|------|------|
| `GET  /api/health` | 健康检查 |
| `/api/doc` | 文档 CRUD |
| `/api/pic` | 图片管理 |
| `/api/cbz` | CBZ 管理 |
| `/api/task` | 任务队列 |
| `/resource/*` | 静态资源（CBZ 下载） |
| `/graphql` | GraphQL 端点 |

## 配置

配置文件位于 `configuration/` 目录。通过 `APP_ENVIRONMENT` 环境变量切换：

```bash
APP_ENVIRONMENT=local     # 加载 configuration/local.yaml
APP_ENVIRONMENT=production # 加载 configuration/production.yaml
```

关键配置项：

- `database.auto_migrate` — 启动时自动运行数据库迁移
- `worker.count` — 后台 Worker 数量（0 = CPU 核心数）
- `pic_dir` / `cbz_dir` — 图片 / CBZ 存储目录

## 运行

```bash
# 开发模式（自动重载）
systemfd --no-pid -s http::9000 -- cargo watch -x "run --bin telegrab"

# 生产构建
cargo build --release
```

## 前置依赖

- Rust 1.94+
- PostgreSQL 数据库
