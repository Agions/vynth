# 贡献指南

欢迎贡献 Zeno！本文档帮助你了解如何参与项目开发。

---

## 行为准则

- 尊重所有贡献者，不论经验水平
-  constructive 反馈，对事���人
- 关注项目目标：本地优先、单二进制、安全可控

---

## 快速贡献

### 1. Fork 与克隆

```bash
git clone git@github.com:Agions/vynth.git
cd zeno
```

### 2. 安装依赖

```bash
bun install
```

### 3. 创建分支

```bash
# 功能分支
git checkout -b feat/my-feature

# 修复分支
git checkout -b fix/my-bug

# 文档分支
git checkout -b docs/update-readme
```

### 4. 提交更改

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
feat: 添加插件热重载支持
fix: 修复 safeResolve 在 Windows 上的路径问题
docs: 更新快速开始指南
chore: 升级 biome 至 1.9.0
```

类型说明：

| 类型 | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat: 支持 MCP 工具调用` |
| `fix` | Bug 修复 | `fix: 修复 ZENO_NET 解析错误` |
| `docs` | 文档更改 | `docs: 补充插件开发指南` |
| `refactor` | 代码重构 | `refactor: 合并 llm 与 tools 包` |
| `test` | 测试相关 | `test: 新增 sandbox 越界单测` |
| `chore` | 构建/工具 | `chore: 升级 turbo 至 2.0` |
| `security` | 安全修复 | `security: 修复路径穿越漏洞` |

### 5. 运行质量闸门

```bash
# 代码规范
bun run lint

# 测试
bun test packages

# 编译
bun run compile

# 体积门禁（MVP ≤ 61MB）
bun run check-binary-size
```

### 6. 推送与 PR

```bash
git push origin feat/my-feature
```

然后在 GitHub 上创建 Pull Request。

---

## PR 规范

### PR 标题

遵循 Conventional Commits 格式：

```
feat: 添加插件热重载支持
```

### PR 描述模板

```markdown
## 变更内容

- 添加 `PluginLoader.reload()` 方法
- 更新 `--plugin` CLI 参数支持热重载

## 关联 Issue

Closes #123

## 测试

- [x] 新增单元测试
- [x] 通过 `bun test packages`
- [x] 通过 `bun run lint`
- [x] 通过 `bun run compile`

##  checklist

- [x] 已阅读 [开发规范](dev-guide.md)
- [x] 已更新文档（如需要）
- [x] 已更新 CHANGELOG.md（如需要）
```

---

## 开发环境

### 前置要求

- **Bun >= 1.1**
- **Node.js >= 18**（用于 `biome`、`turbo`）
- **Git**

### IDE 推荐

- VS Code（TypeScript + Biome 插件）
- JetBrains Rider / WebStorm

### 调试

```bash
# 编译并运行
bun run compile
./dist/zeno --help

# 或直接用 bun 运行（开发时）
bun run apps/cli/src/main.ts --help
```

---

## CI 流水线

PR 合并前需通过以下检查：

1. **install** —— 依赖安装
2. **lint** —— biome 检查（0 error）
3. **build** —— TypeScript 编译
4. **compile** —— 单二进制打包
5. **test** —— `bun test packages`（0 fail）
6. **gitleaks** —— 密钥扫描（无硬编码）
7. **binary-size** —— 体积门禁（MVP ≤ 61MB）
8. **sign** —— 仅 tag 触发签名

---

## 安全红线

- **禁止**硬编码 API Key / 密码 / 私钥
- **禁止**向远程明文 `http` 端点发送 API Key（`localhost` 除外）
- **禁止**引入新的外部依赖未经讨论
- **禁止**删除或修改 `gitleaks.toml` 允许列表未经审批

详见 [开发规范](dev-guide.md)。

---

## 获取帮助

- [GitHub Issues](https://github.com/Agions/vynth/issues)
- [GitHub Discussions](https://github.com/Agions/vynth/discussions)

---

## 相关文档

- [开发规范](dev-guide.md) —— 分支模型、代码规范、安全红线
- [测试指南](testing.md) —— 测试策略与基准测试
- [架构总览](../architecture/index.md) —— 模块关系与数据流
