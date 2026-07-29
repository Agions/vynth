# 安装指南

三种安装方式，按推荐顺序排列。装完统一得到一个 `zeno` 命令（单二进制，~61 MB，零外部依赖）。

## 方式一：快捷安装脚本（推荐）

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/Agions/vynth/main/scripts/install.sh | bash
```

脚本会自动完成：

1. 检测 Bun，缺失时自动安装
2. 克隆源码（或复用当前仓库）→ `bun install` → 编译单二进制
3. 安装到 `~/.local/bin/zeno`（免 sudo；不可写时降级 `/usr/local/bin`）
4. PATH 检测与追加提示

可选参数：

| 参数 | 作用 |
| --- | --- |
| `--prefix <dir>` | 自定义安装目录 |
| `--no-build` | 跳过编译，直接安装已有的 `dist/zeno` |
| `--uninstall` | 卸载（保留 `~/.zeno` 配置目录） |

### Windows（PowerShell）

```powershell
irm https://raw.githubusercontent.com/Agions/vynth/main/scripts/install.ps1 | iex
```

安装到 `%USERPROFILE%\.zeno\bin\zeno.exe` 并自动写入用户级 PATH（重开终端生效）。支持 `-Prefix`、`-NoBuild`、`-Uninstall` 参数。

## 方式二：源码手动构建

```bash
git clone https://github.com/Agions/vynth.git && cd zeno
bun install
bun run compile          # → dist/zeno
./dist/zeno --version
```

想全局可用：`cp dist/zeno ~/.local/bin/` 或 `bun link`。

## 方式三：直接分发二进制

`dist/zeno` 是自包含单文件——`scp` 到任何同架构机器即可运行，无需 Bun、无需 `node_modules`：

```bash
scp dist/zeno server:/usr/local/bin/zeno
ssh server 'zeno --version'
```

## 前置要求

| 依赖 | 版本 | 用途 |
| --- | --- | --- |
| Bun | >= 1.1 | 构建 + 运行时（仅构建机需要） |
| Git | 任意 | 获取源码 |
| Node.js | >= 18（可选） | 仅 `biome` / `turbo` 等开发辅助工具 |

## 平台支持矩阵

| 平台 | 运行 | OS 硬隔离（`ZENO_HARDEN=1`） |
| --- | --- | --- |
| Linux | ✅ | ✅ bubblewrap（推荐生产环境） |
| macOS | ✅ | ⚠️ seatbelt 需 root（macOS 15+ 限制），非 root 时 Fail-Closed 报错 |
| Windows | ✅ | ❌ 暂不支持硬隔离，路径守卫与网络开关仍生效 |

## 安装后验证

```bash
zeno --version               # 0.1.1
export ZENO_API_KEY="sk-..."
zeno -g 'echo 一句话确认你能工作'
```

出问题先查 [FAQ](../faq/index.md)；下一步阅读 [快速开始](getting-started.md)。
