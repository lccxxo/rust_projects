# web_skeleton

企业级 Rust Web 后端骨架，基于 **axum + tokio**，遵循 Rust web 开发的分层与模块化惯例。开箱即跑，可直接在此基础上开发业务。

## 技术栈

| 用途 | 库 |
|------|----|
| Web 框架 | [axum](https://github.com/tokio-rs/axum) 0.8 |
| 异步运行时 | [tokio](https://tokio.rs) 1 |
| HTTP 中间件 | [tower-http](https://github.com/tower-rs/tower-http)（日志/CORS/超时/请求 ID） |
| 结构化日志 | [tracing](https://github.com/tokio-rs/tracing) + tracing-subscriber |
| 序列化 | [serde](https://serde.rs) + serde_json |
| 错误处理 | [thiserror](https://github.com/dtolnay/thiserror) + anyhow |
| 参数校验 | [validator](https://github.com/Keats/validator) |
| 配置 | dotenvy（环境变量 + .env） |
| 工具 | uuid、chrono、async-trait |

## 目录结构

```
src/
├── main.rs            # 入口：加载配置/日志、启动服务
├── app.rs             # 应用组装：路由 + 全局中间件栈
├── config.rs          # 配置：从环境变量加载
├── state.rs           # 共享状态（config + 各仓储），经 State 注入
├── error.rs           # 统一错误类型 AppError（实现 IntoResponse）
├── response.rs        # 统一响应包装 ApiResponse / Created
├── extract.rs         # 自定义提取器 ValidatedJson（自动校验）
├── middleware/        # 自定义中间件
│   └── auth.rs        #   Bearer Token 鉴权示例
├── routes/            # 路由层（薄 handler）
│   ├── health.rs      #   健康检查
│   └── user.rs        #   用户 CRUD
├── services/          # 业务逻辑层
│   └── user.rs
├── models/            # 领域实体 + 请求/响应 DTO
│   └── user.rs
└── repository/        # 仓储层：trait 抽象 + 内存实现
    └── user.rs
```

分层调用链：`routes` → `services` → `repository` → 存储。
横向支撑：`error` / `response` / `extract` / `middleware` / `config` / `state`。

## 运行

```bash
cp .env.example .env    # 可选，按需改配置
cargo run
```

默认监听 `http://127.0.0.1:3000`。

## 接口示例

```bash
# 健康检查
curl http://127.0.0.1:3000/health

# 创建用户
curl -X POST http://127.0.0.1:3000/api/users \
  -H 'Content-Type: application/json' \
  -d '{"name":"张三","email":"zhangsan@example.com"}'

# 列表 / 详情 / 更新 / 删除
curl http://127.0.0.1:3000/api/users
curl http://127.0.0.1:3000/api/users/{id}
curl -X PUT http://127.0.0.1:3000/api/users/{id} \
  -H 'Content-Type: application/json' -d '{"name":"新名字"}'
curl -X DELETE http://127.0.0.1:3000/api/users/{id}
```

统一响应结构：

```json
{ "code": 0, "message": "success", "data": { ... } }
```

错误响应（如参数校验失败）：

```json
{ "code": 42200, "message": "参数校验失败", "data": { "email": [ ... ] } }
```

## 如何扩展

**加一个新资源（如 post）**，照着 user 复制四处：
1. `models/post.rs` — 定义实体和 DTO
2. `repository/post.rs` — 定义 `PostRepository` trait + 实现
3. `services/post.rs` — 业务逻辑
4. `routes/post.rs` — 路由和 handler，然后在 `routes/mod.rs` 里 `.nest("/posts", ...)`
5. 在 `state.rs` 的 `AppState` 里加上 `posts` 字段

**换成真实数据库（如 PostgreSQL）**：
- 加依赖 `sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "chrono"] }`
- 新建 `SqlxUserRepository`，实现同一个 `UserRepository` trait
- 在 `AppState::new` 里把 `InMemoryUserRepository` 换成它
- 业务层、路由层**无需改动**——这就是 trait 抽象的价值

**给路由加鉴权**：
```rust
use axum::middleware;
// 在需要保护的路由上：
.layer(middleware::from_fn(crate::middleware::auth::require_auth))
```

## 配置项

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `APP_HOST` | 127.0.0.1 | 监听地址 |
| `APP_PORT` | 3000 | 监听端口 |
| `APP_LOG_LEVEL` | info | 日志等级 |
| `APP_REQUEST_TIMEOUT_SECS` | 30 | 请求超时（秒） |
| `APP_BODY_LIMIT_BYTES` | 2097152 | 请求体上限（2MB） |

也可用 `RUST_LOG` 做细粒度日志控制，如 `RUST_LOG=web_skeleton=debug,tower_http=debug`。
