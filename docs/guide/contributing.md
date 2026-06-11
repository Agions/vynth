# 贡献指南

感谢你对 Synerix 的关注！我们欢迎各种形式的贡献。

## 如何贡献

### 报告问题

如果你发现了 bug 或有功能建议，请在 [GitHub Issues](https://github.com/Agions/synerix/issues) 中创建一个新的 issue。

### 提交代码

1. Fork 仓库
2. 创建你的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交你的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建一个 Pull Request

## 开发环境设置

### 前置要求

- Rust 1.75+
- Node.js 18+
- pnpm 或 npm

### 克隆仓库

```bash
git clone https://github.com/Agions/synerix.git
cd synerix
```

### 安装依赖

```bash
# Rust 依赖
cargo build

# 文档依赖
npm install
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name
```

## 代码规范

### Rust 代码

- 遵循 Rust 官方代码风格
- 使用 `rustfmt` 格式化代码
- 使用 `clippy` 检查代码质量
- 为公共 API 编写文档注释

### 提交信息

使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

类型：
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响代码运行的变动）
- `refactor`: 重构（既不是新增功能，也不是修改 bug 的代码变动）
- `perf`: 性能优化
- `test`: 增加测试
- `chore`: 构建过程或辅助工具的变动

## Pull Request 规范

### PR 标题

使用与提交信息相同的格式。

### PR 描述

请包含以下信息：

1. **变更说明**: 简要描述你的更改
2. **相关 Issue**: 如果有相关 issue，请链接它
3. **测试**: 描述你如何测试了这些更改
4. **截图**: 如果有 UI 更改，请提供截图

### 代码审查

所有 PR 都需要经过代码审查。请耐心等待维护者的反馈。

## 文档贡献

我们欢迎对文档的贡献：

1. 修复错别字
2. 改进说明
3. 添加示例
4. 翻译文档

## 社区行为准则

请保持友好和尊重。

## 获取帮助

如果你有任何问题，可以通过以下方式联系我们：

- [GitHub Discussions](https://github.com/Agions/synerix/discussions)
- [Discord](https://discord.gg/synerix)

## 感谢

感谢所有贡献者的支持！
