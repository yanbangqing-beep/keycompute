# 贡献指南

感谢你对 KeyCompute 项目的关注！我们欢迎各种形式的贡献，包括代码、文档、问题反馈和功能建议。

## 开发环境

### 技术栈

- **后端**: Rust ≥ 1.92 + Axum 0.8 + Tokio
- **前端**: Dioxus 0.7 (WASM)
- **数据库**: PostgreSQL 16 + SQLx
- **缓存/限流**: Redis 7

### 环境准备

```bash
# 克隆项目
git clone https://github.com/aiqubits/keycompute.git
cd keycompute

# 安装 Rust (如未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 dioxus-cli (前端开发需要)
curl -sSL http://dioxus.dev/install.sh | sh

# 启动依赖服务 (PostgreSQL + Redis)
docker compose up -d postgres redis
```

### 本地开发

```bash
# 复制环境变量配置
cp .env.example .env
# 编辑 .env 填入必要的配置

# 启动后端服务
cargo run -p keycompute-server --features redis

# 启动前端开发服务器 (另一个终端)
cd packages/web && dx serve
```

## 代码规范

### Rust 代码

```bash
# 格式化代码
cargo fmt --all

# 运行 Clippy 检查
cargo clippy --workspace --exclude desktop --exclude mobile --all-targets --all-features --future-incompat-report -- -D warnings

# 运行测试
cargo test --lib --workspace --exclude desktop --exclude mobile
```

### 提交规范

- 使用清晰的提交信息，说明**做了什么**和**为什么**
- 一个提交只完成一个逻辑变更
- 提交信息使用中文或英文，保持统一

示例：
```
feat: 添加 DeepSeek Provider 支持

- 实现 DeepSeek API 客户端
- 添加流式响应处理
- 更新文档
```

## 架构原则

### 模块依赖

```
keycompute-server (入口)
    ↓
llm-gateway / keycompute-auth / keycompute-billing / ... (业务层)
    ↓
keycompute-db / keycompute-runtime (基础设施层)
    ↓
keycompute-types (基础类型)
```

**原则**: 上层模块可以依赖下层模块，禁止反向依赖。

### 新增 Provider

如需添加新的 LLM Provider：

1. 在 `crates/llm-provider/` 下创建新 crate
2. 实现 `keycompute-provider-trait` 中的 trait
3. 在 `llm-gateway` 中注册 Provider
4. 添加对应的测试用例

### 数据库变更

1. 在 `crates/keycompute-db/src/migrations/` 添加迁移文件
2. 在 `crates/keycompute-db/src/models/` 更新模型
3. 运行 `cargo run --bin migration` 应用迁移

## 提交 Pull Request

1. **Fork 仓库** 并创建你的分支 (`git checkout -b feature/amazing-feature`)
2. **提交变更** (`git commit -m 'feat: 添加某个功能'`)
3. **推送到分支** (`git push origin feature/amazing-feature`)
4. **创建 Pull Request**

### PR 检查清单

- [ ] 代码通过 `cargo clippy` 检查
- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 新增功能包含测试用例
- [ ] 所有测试通过
- [ ] 文档已更新（如需要）

## 报告问题

### Bug 报告

请包含以下信息：

- 问题描述
- 复现步骤
- 期望行为 vs 实际行为
- 环境信息（OS、Rust 版本等）
- 相关日志或错误信息

### 功能建议

请描述：

- 功能的使用场景
- 期望的行为
- 可能的实现思路（可选）

## 社区交流

- **Issue 讨论**: 使用 GitHub Issues 进行功能讨论
- **问题反馈**: 发现 Bug 请提交 Issue

## 许可证

通过提交代码，你同意你的贡献将采用与项目相同的 [MIT 许可证](LICENSE)。
