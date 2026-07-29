param(
    [string]$Prefix = "",
    [switch]$NoBuild,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"
$RepoUrl = "https://github.com/Agions/vynth.git"
$BinName = "vynth.exe"

function Info($msg) { Write-Host "> $msg" -ForegroundColor Cyan }
function Ok($msg)   { Write-Host " v $msg" -ForegroundColor Green }
function Warn($msg) { Write-Host " ! $msg" -ForegroundColor Yellow }
function Fail($msg) { Write-Host " x $msg" -ForegroundColor Red; exit 1 }

$InstallDir = if ($Prefix) { $Prefix } else { Join-Path $env:USERPROFILE ".vynth\bin" }

if ($Uninstall) {
    $target = Join-Path $InstallDir $BinName
    if (Test-Path $target) {
        Remove-Item $target -Force
        Ok "removed $target"
    } else {
        Warn "$BinName not found"
    }
    Info "config dir $env:USERPROFILE\.vynth not deleted; remove manually if needed"
    exit 0
}

Write-Host ""
Write-Host "  vynth -- terminal-first AI coding agent" -ForegroundColor Cyan
Write-Host "  ------------------------------------------"
Write-Host ""

# -- download prebuilt binary for fastest install --
$dlName = ""
if (-not $NoBuild) {
    if ($IsMacOS) { $dlName = "vynth" }
    elseif ($IsLinux) { $dlName = "vynth-linux" }
    if ($dlName) {
        try {
            $releaseJson = (Invoke-WebRequest -Uri "https://api.github.com/repos/Agions/vynth/releases/latest" -ErrorAction SilentlyContinue).Content | ConvertFrom-Json
            $tag = $releaseJson.tag_name
            if ($tag) {
                $dlUrl = "https://github.com/Agions/vynth/releases/download/$tag/$dlName"
                Info "downloading prebuilt $BinName ($tag)..."
                $tmpBin = Join-Path ([System.IO.Path]::GetTempPath()) $BinName
                Invoke-WebRequest -Uri $dlUrl -OutFile $tmpBin -ErrorAction Stop
                # verify sha256 if available
                try {
                    $shaUrl = "https://github.com/Agions/vynth/releases/download/$tag/vynth.sha256"
                    $shaContent = (Invoke-WebRequest -Uri $shaUrl -ErrorAction Stop).Content
                    $expected = ($shaContent -split "`n" | Select-String $dlName).Line.Split(" ")[0]
                    $actual = (Get-FileHash $tmpBin -Algorithm SHA256).Hash.ToLower()
                    if ($expected -eq $actual) { Ok "sha256 verified" }
                    else { Warn "sha256 mismatch; falling back to build from source"; throw }
                } catch { throw }
                New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
                Copy-Item $tmpBin (Join-Path $InstallDir $BinName) -Force
                Remove-Item $tmpBin -Force
                $sizeMB = [math]::Round((Get-Item (Join-Path $InstallDir $BinName)).Length / 1MB, 1)
                Ok "installed: $InstallDir\$BinName (${sizeMB} MB)"
                EnsurePath
                exit 0
            }
        } catch {
            Warn "download failed; building from source..."
        }
    }
}

# -- build from source --
if (-not (Get-Command "bun" -ErrorAction SilentlyContinue)) {
    Warn "bun not found, auto-installing (https://bun.sh)..."
    try {
        irm bun.sh/install.ps1 | iex
        $env:Path = "$env:USERPROFILE\.bun\bin;$env:Path"
    } catch {
        Fail "bun auto-install failed: irm bun.sh/install.ps1 | iex"
    }
    if (-not (Get-Command "bun" -ErrorAction SilentlyContinue)) {
        Fail "bun still not available after install; reopen terminal and retry"
    }
}
Ok "bun $(bun --version)"

$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { "" }
$SrcDir = ""
if ($ScriptDir -and (Test-Path (Join-Path $ScriptDir "..\package.json"))) {
    $pkg = Get-Content (Join-Path $ScriptDir "..\package.json") -Raw
    if ($pkg -match '"name": "vynth"') {
        $SrcDir = (Resolve-Path (Join-Path $ScriptDir "..")).Path
        Info "using local repo: $SrcDir"
    }
}
if (-not $SrcDir) {
    if (-not (Get-Command "git" -ErrorAction SilentlyContinue)) { Fail "git required; install git first" }
    $SrcDir = Join-Path ([System.IO.Path]::GetTempPath()) "vynth-install\vynth"
    if (Test-Path $SrcDir) { Remove-Item $SrcDir -Recurse -Force }
    Info "cloning source..."
    git clone --depth 1 $RepoUrl $SrcDir
    if ($LASTEXITCODE -ne 0) { Fail "clone failed: $RepoUrl" }
}
Set-Location $SrcDir

$DistBin = Join-Path $SrcDir "dist\$BinName"
if ($NoBuild -and (Test-Path $DistBin)) {
    Info "skipping build, using existing dist\$BinName"
} else {
    Info "installing dependencies..."
    bun install
    if ($LASTEXITCODE -ne 0) { Fail "bun install failed" }
    Info "compiling single binary (bun build --compile)..."
    bun run compile
    if ($LASTEXITCODE -ne 0) { Fail "compile failed" }
}
if (-not (Test-Path $DistBin)) { Fail "compile output dist\$BinName not found" }

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item $DistBin (Join-Path $InstallDir $BinName) -Force
$sizeMB = [math]::Round((Get-Item (Join-Path $InstallDir $BinName)).Length / 1MB, 1)
Ok "installed: $InstallDir\$BinName (${sizeMB} MB)"

function EnsurePath {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$userPath", "User")
        Ok "added $InstallDir to user PATH (reopen terminal to apply)"
    }
    $env:Path = "$InstallDir;$env:Path"
}
EnsurePath

Write-Host ""
Write-Host "  install complete! to get started:" -ForegroundColor Green
Write-Host ""
Write-Host '    $env:VYNTH_API_KEY = "sk-..."        # 1. set LLM API key'
Write-Host "    vynth                                # 2. interactive TUI"
Write-Host "    vynth -g 'write unit tests'          # 3. headless mode"
Write-Host ""
Write-Host "  docs: https://github.com/Agions/vynth/tree/main/docs"
Write-Host ""
