param(
    [string]$Prefix = "",
    [switch]$NoBuild,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"
$RepoUrl = "https://github.com/Agions/zeno.git"
$BinName = "zeno.exe"

function Info($msg) { Write-Host "> $msg" -ForegroundColor Cyan }
function Ok($msg)   { Write-Host "√ $msg" -ForegroundColor Green }
function Warn($msg) { Write-Host "! $msg" -ForegroundColor Yellow }
function Fail($msg) { Write-Host "x $msg" -ForegroundColor Red; exit 1 }

$InstallDir = if ($Prefix) { $Prefix } else { Join-Path $env:USERPROFILE ".zeno\bin" }

if ($Uninstall) {
    $target = Join-Path $InstallDir $BinName
    if (Test-Path $target) {
        Remove-Item $target -Force
        Ok "已删除 $target"
    } else {
        Warn "未找到已安装的 $BinName"
    }
    Info "配置目录 $env:USERPROFILE\.zeno 未删除（含 config.json），如需彻底清理请手动删除"
    exit 0
}

Write-Host ""
Write-Host "  Zeno Installer -- Terminal-first AI coding agent" -ForegroundColor Cyan
Write-Host "  -------------------------------------------------"
Write-Host ""

if (-not (Get-Command "bun" -ErrorAction SilentlyContinue)) {
    Warn "未检测到 Bun，正在自动安装（https://bun.sh）..."
    try {
        irm bun.sh/install.ps1 | iex
        $env:Path = "$env:USERPROFILE\.bun\bin;$env:Path"
    } catch {
        Fail "Bun 自动安装失败，请手动安装后重试：irm bun.sh/install.ps1 | iex"
    }
    if (-not (Get-Command "bun" -ErrorAction SilentlyContinue)) {
        Fail "Bun 安装后仍不可用，请重开终端后重试"
    }
}
Ok "Bun $(bun --version)"

$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { "" }
$SrcDir = ""
if ($ScriptDir -and (Test-Path (Join-Path $ScriptDir "..\package.json"))) {
    $pkg = Get-Content (Join-Path $ScriptDir "..\package.json") -Raw
    if ($pkg -match '"name": "zeno"') {
        $SrcDir = (Resolve-Path (Join-Path $ScriptDir "..")).Path
        Info "使用当前仓库: $SrcDir"
    }
}
if (-not $SrcDir) {
    if (-not (Get-Command "git" -ErrorAction SilentlyContinue)) { Fail "需要 git 来获取源码，请先安装 git" }
    $SrcDir = Join-Path ([System.IO.Path]::GetTempPath()) "zeno-install\zeno"
    if (Test-Path $SrcDir) { Remove-Item $SrcDir -Recurse -Force }
    Info "克隆源码到临时目录..."
    git clone --depth 1 $RepoUrl $SrcDir
    if ($LASTEXITCODE -ne 0) { Fail "克隆失败: $RepoUrl" }
}
Set-Location $SrcDir

$DistBin = Join-Path $SrcDir "dist\$BinName"
if ($NoBuild -and (Test-Path $DistBin)) {
    Info "跳过编译，使用现有 dist\$BinName"
} else {
    Info "安装依赖..."
    bun install
    if ($LASTEXITCODE -ne 0) { Fail "bun install 失败" }
    Info "编译单二进制（bun build --compile）..."
    bun run compile
    if ($LASTEXITCODE -ne 0) { Fail "编译失败" }
    if (-not (Test-Path $DistBin) -and (Test-Path (Join-Path $SrcDir "dist\zeno"))) {
        Copy-Item (Join-Path $SrcDir "dist\zeno") $DistBin -Force
    }
}
if (-not (Test-Path $DistBin)) { Fail "编译产物 dist\$BinName 不存在" }

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item $DistBin (Join-Path $InstallDir $BinName) -Force
$sizeMB = [math]::Round((Get-Item (Join-Path $InstallDir $BinName)).Length / 1MB, 1)
Ok "已安装: $InstallDir\$BinName (${sizeMB} MB)"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$userPath", "User")
    Ok "已将 $InstallDir 追加到用户 PATH（重开终端后生效）"
}

Write-Host ""
Write-Host "  安装完成！三步开跑：" -ForegroundColor Green
Write-Host ""
Write-Host '    $env:ZENO_API_KEY = "sk-..."       # 1. 设置 LLM API Key'
Write-Host "    zeno                                # 2. 交互式 TUI"
Write-Host "    zeno -g '给 src 写单元测试'          # 3. 或无头模式直接干活"
Write-Host ""
Write-Host "  文档: https://github.com/Agions/zeno/tree/main/docs"
Write-Host ""
