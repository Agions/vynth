# Vynth 回滚 Runbook（v0.1.0）

> 适用范围：Vynth 单二进制分发（`dist/vynth`）。升级 = 覆盖；回滚 = 恢复上一版本快照。
> 本 runbook 与 `release-notes-v0.1.0.md` 的「升级与回滚」一节配套，提供更可操作的步骤。

## 1. 前置条件

- 当前运行的二进制位于 `dist/vynth`（或已放入 `PATH`，如 `/usr/local/bin/vynth`）。
- 升级前**已保留上一版本快照** `dist/vynth.prev`（见下方「快照约定」）。
- 若二进制在 `PATH` 中且路径非 `dist/vynth`，请将下方命令中的路径替换为实际路径。

### 快照约定（每次升级前必做）

```bash
# 升级前先备份当前版本，命名为 .prev
cp dist/vynth dist/vynth.prev
```

> 若发布前尚无 `.prev`，可用 git 中记录的已发布版本重建：
> `git show v0.1.0:dist/vynth > dist/vynth.prev`（注意：仓库默认忽略 `dist/`，需从另一份已发布二进制复制）。

## 2. 回滚步骤（单二进制覆盖）

```bash
cd /path/to/vynth          # 项目根目录

# 步骤 1：用上一版本快照覆盖当前二进制
cp dist/vynth.prev dist/vynth

# （可选）若二进制在 PATH 中，同步覆盖
# cp dist/vynth.prev /usr/local/bin/vynth
```

回滚是**单文件操作**：无数据库、无配置文件迁移、无用户态状态变更。数据目录 `~/.vynth`（`VYNTH_DATA_DIR` 可改）跨版本兼容，回滚一般不影响。

## 3. 回滚后验证（必做）

回滚完成后必须执行以下两项验证，任一项失败视为回滚未完成。

### 3.1 版本校验

```bash
./dist/vynth --version
```

- 预期输出：`0.1.0`（或你回滚到的目标版本号，与 `dist/vynth.prev` 对应的发布版本一致）。
- 退出码应为 `0`。

### 3.2 一次 `-g` 冒烟验证（无头模式）

```bash
./dist/vynth -g "用一句话介绍 vynth"
```

- 预期：进入 agent 循环并以 Demo（`EchoProvider`）流式输出（未设置 `VYNTH_API_KEY` 时），或接入真实 LLM 输出结果；最终正常退出，退出码 `0`。
- 若设置真实 key，请确认 `VYNTH_LLM_BASE_URL` 指向可信 `https` 端点。
- 验证目的：确认二进制可正常启动、agent 循环与内置工具（含 `read_file`/`run_shell`）工作正常。

## 4. 紧急快速回滚（一行）

```bash
cp dist/vynth.prev dist/vynth && ./dist/vynth --version
```

## 5. 多版本并存（可选）

无需覆盖，可保留多个版本并分别放入 `PATH`：

```bash
cp dist/vynth.prev dist/vynth-0.1.0
cp dist/vynth      dist/vynth-0.2.0   # 假设新版本为 0.2.0
# 使用时按需软链 / 直接调用对应版本
```

## 6. 故障排查

| 现象 | 可能原因 | 处理 |
|------|----------|------|
| `cp` 提示无权限 | 目标路径在系统目录 | 用 `sudo cp`，或回滚到项目内 `dist/` 后用 `PATH` 切换 |
| `--version` 输出版本不符 | 快照非预期版本 | 重新确认 `dist/vynth.prev` 来源，或从 git tag 重建 |
| `-g` 冒烟卡住/报错 | 数据目录损坏或环境变量异常 | 临时指定 `VYNTH_DATA_DIR=/tmp/vynth-verify` 隔离数据目录重试 |
| 回滚后行为仍异常 | 实际运行的是 `PATH` 中其他位置的旧二进制 | `which vynth` 确认实际路径，统一覆盖 |
