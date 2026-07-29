# AGENTS.md — 项目 AI 辅助开发与架构规则

## 1. 项目简介与架构
- **项目名称**: Zeno 终端 AI 编程系统
- **开发模式**: 推荐在 Vibe 模式下快速迭代，Plan 模式下重构规划

## 2. 代码规范与指令
- 统一使用 TypeScript 严格模式
- 运行单元测试: `bun test packages`
- 编译可执行程序: `bun run compile`

## 3. 注意事项
- 保持 UI 组件无外框极简规范
- 所有命令错误统一输出 VC-XXXXXX 规范 6 位错误码
