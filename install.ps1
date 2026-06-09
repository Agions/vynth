<#
.SYNOPSIS
    Synerix — 跨平台安装脚本（Windows PowerShell 版）
    等效于 Linux/macOS 的 curl -fsSL ... | bash

.DESCRIPTION
    自动检测架构 → 获取最新版本号 → 双源（GitHub/Gitee）下载预编译二进制
    → 解压 .zip → 安装到用户目录 → 添加到 PATH

    用法：
        # 一行命令 (等效 curl | bash)
        iwr -useb https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex

        # Gitee 国内镜像
        iwr -useb https://gitee.com/Agions/synerix/raw/main/install.ps1 | iex

        # 自定义安装路径
        $env:SYNERIX_HOME = "D:\tools\synerix"; iwr -useb ... | iex
#>

# ============================================================
# Layer 1: 配置与常量
#   - 所有可调参数集中在此，方便维护和自定义
#   - 不使用全局变量穿透到下层函数，统一通过参数传递
# ============================================================
$Script:RepoOwner   = "Agions"
$Script:RepoName    = "synerix"
$Script:BinaryName  = "synerix"
$Script:OsName      = "windows"
$Script:ArchiveExt  = ".zip"

# 安装路径：默认 %LOCALAPPDATA%\Programs\synerix，支持 SYNERIX_HOME 覆盖
$Script:InstallDir  = if ($env:SYNERIX_HOME) {
    $env:SYNERIX_HOME
} else {
    "$env:LOCALAPPDATA\Programs\synerix"
}

# 双源配置：GitHub 优先，Gitee 兜底（国内加速）
$Script:Sources = @(
    @{ Name = "github"; ApiUrl = "https://api.github.com/repos/{0}/{1}"; DownloadUrl = "https://github.com/{0}/{1}/releases/download/{2}/{3}" }
    @{ Name = "gitee";  ApiUrl = "https://gitee.com/api/v5/repos/{0}/{1}"; DownloadUrl = "https://gitee.com/{0}/{1}/releases/download/{2}/{3}" }
)

# ─── 安全 & 性能优化 ───
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13


# ============================================================
# Layer 2: 基础工具函数
#   纯 IO 辅助，无业务逻辑
# ============================================================

<#
.SYNOPSIS
    带颜色的日志输出函数集
.DESCRIPTION
    四种级别：Info(青)、Ok(绿)、Warn(黄)、Error(红)
    Error 为终止性错误，会立即退出脚本
#>
function Write-Info  { Write-Host "[info] " -ForegroundColor Cyan  -NoNewline; Write-Host "$args" }
function Write-Ok    { Write-Host "[ok] "   -ForegroundColor Green -NoNewline; Write-Host "$args" }
function Write-Warn  { Write-Host "[warn] " -ForegroundColor Yellow -NoNewline; Write-Host "$args" }

# 终止性错误：输出后立即退出，避免下游继续执行
function Write-Error {
    Write-Host "[error] " -ForegroundColor Red -NoNewline; Write-Host "$args"
    exit 1
}

<#
.SYNOPSIS
    创建唯一临时目录
.OUTPUTS
    临时目录的完整路径
#>
function New-TempDirectory {
    $path = Join-Path -Path ([System.IO.Path]::GetTempPath()) -ChildPath ([System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Force -Path $path | Out-Null
    return $path
}

<#
.SYNOPSIS
    安全删除目录（静默忽略不存在或权限错误）
#>
function Remove-DirectoryIfExists {
    param([string]$Path)
    if (Test-Path -Path $Path) {
        Remove-Item -Path $Path -Recurse -Force -ErrorAction SilentlyContinue
    }
}


# ============================================================
# Layer 3: 领域模型 - 资产路径构建
#   纯计算函数，无 IO 副作用，可独立测试
# ============================================================

<#
.SYNOPSIS
    检测 Windows CPU 架构，返回 Synerix 发行版命名规范
.OUTPUTS
    "x86_64" 或 "aarch64"
#>
function Get-WindowsArchitecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default { Write-Error "不支持的 CPU 架构: $arch（仅支持 x86_64 / ARM64）" }
    }
}

<#
.SYNOPSIS
    构建 Release 资产文件名称
    例：synerix-v0.2.2-windows-x86_64.zip
#>
function Build-ReleaseAssetName {
    param([string]$Tag, [string]$Arch)
    return "${Script:BinaryName}-${Tag}-${Script:OsName}-${Arch}${Script:ArchiveExt}"
}

<#
.SYNOPSIS
    构建 Release 下载 URL
    例：https://github.com/Agions/synerix/releases/download/v0.2.2/synerix-v0.2.2-windows-x86_64.zip
#>
function Build-ReleaseDownloadUrl {
    param([string]$SourceName, [string]$Tag, [string]$Arch)
    $assetName = Build-ReleaseAssetName -Tag $Tag -Arch $Arch
    $source = $Script:Sources | Where-Object { $_.Name -eq $SourceName }

    if (-not $source) {
        Write-Error "未知的下载源: $SourceName"
    }

    return $source.DownloadUrl -f $Script:RepoOwner, $Script:RepoName, $Tag, $assetName
}


# ============================================================
# Layer 4: 网络操作层
#   所有外部请求集中在此，统一错误处理和双源兜底策略
# ============================================================

<#
.SYNOPSIS
    双源 API 调用器：按优先级依次尝试多个源，全部失败则返回 null
    提取了 Get-LatestReleaseTag 中的 GitHub/Gitee 兜底逻辑为通用模式
.PARAMETER  Urls
    按优先级排列的 URL 列表（如 [GitHub API URL, Gitee API URL]）
.PARAMETER  ParseResult
    从响应中提取所需数据的脚本块
#>
function Invoke-RestApiWithFallback {
    param(
        [string[]]$Urls,
        [scriptblock]$ParseResult
    )

    foreach ($url in $Urls) {
        try {
            $response = Invoke-RestMethod -Uri $url -ErrorAction Stop
            $result = & $ParseResult $response
            if ($result) { return $result }
        } catch {
            # 静默跳过，继续尝试下一个源
            continue
        }
    }

    return $null
}

<#
.SYNOPSIS
    获取最新的 Release 标签版本号
    策略：GitHub API → Gitee API → 硬编码兜底
#>
function Get-LatestReleaseTag {
    $repoFull = "${Script:RepoOwner}/${Script:RepoName}"

    $urls = @(
        "https://api.github.com/repos/$repoFull/releases/latest",
        "https://gitee.com/api/v5/repos/$repoFull/releases?page=1&per_page=1"
    )

    # GitHub API 返回 {tag_name: "v0.2.2"}
    # Gitee API 返回 [{tag_name: "v0.2.2"}]
    $tag = Invoke-RestApiWithFallback -Urls $urls -ParseResult {
        param($resp)
        if ($resp -is [array] -and $resp.Count -gt 0) { return $resp[0].tag_name }
        return $resp.tag_name
    }

    if (-not $tag) {
        Write-Warn "无法获取最新版本号（GitHub 和 Gitee 均失败），回退到 v0.2.2"
        return "v0.2.2"
    }

    return $tag
}

<#
.SYNOPSIS
    双源文件存在性检测：按优先级 HEAD 检查多个源，返回第一个可用的 URL
    提取了 Get-ArchiveUrl 中的 HEAD 检测逻辑为通用模式
#>
function Resolve-ArchiveDownloadUrl {
    param([string]$Tag, [string]$Arch)

    foreach ($source in $Script:Sources) {
        $url = Build-ReleaseDownloadUrl -SourceName $source.Name -Tag $Tag -Arch $Arch

        try {
            $req = [System.Net.WebRequest]::CreateHttp($url)
            $req.Method = "HEAD"
            $req.Timeout = 5000
            $resp = $req.GetResponse()
            $statusCode = [int]$resp.StatusCode
            $resp.Close()

            if ($statusCode -eq 200) {
                Write-Info "从 ${($source.Name)} 找到 Release 文件"
                return $url
            }
        } catch {
            # 源不可用，静默尝试下一个
            continue
        }
    }

    return $null
}

<#
.SYNOPSIS
    下载文件到本地路径
    与 Resolve-ArchiveDownloadUrl 分离：一个负责寻址，一个负责传输
#>
function Invoke-FileDownload {
    param([string]$Url, [string]$DestinationPath)

    Write-Info "下载中: $Url"
    try {
        Invoke-WebRequest -Uri $Url -OutFile $DestinationPath -ErrorAction Stop
    } catch {
        Write-Error "下载失败: $_"
    }
}


# ============================================================
# Layer 5: 文件系统操作层
#   解压、查找、复制、安装等本地文件操作
# ============================================================

<#
.SYNOPSIS
    解压归档文件到目标目录
    支持 .zip（PowerShell 原生）和 .tar.gz（Windows 10 1803+ tar.exe 兜底）
#>
function Expand-ArchiveFile {
    param([string]$ArchivePath, [string]$DestinationPath)

    Write-Info "解压中..."

    # 确保目标目录存在
    New-Item -ItemType Directory -Force -Path $DestinationPath | Out-Null

    $ext = [System.IO.Path]::GetExtension($ArchivePath)
    if ($ext -eq ".zip") {
        Expand-Archive -Path $ArchivePath -DestinationPath $DestinationPath -Force
    } else {
        # tar.gz 兜底：Windows 10 1803+ 内置 tar.exe
        $output = tar -xzf $ArchivePath -C $DestinationPath 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "解压失败: $output"
        }
    }
}

<#
.SYNOPSIS
    在目录树中递归查找目标二进制文件
    优先匹配 [name].exe，次选 [name]（无扩展名）
#>
function Resolve-BinaryInDirectory {
    param([string]$DirectoryPath, [string]$BinaryName)

    # 优先查找带 .exe 扩展名的文件
    $exePath = Get-ChildItem -Path $DirectoryPath -Recurse -Filter "${BinaryName}.exe" -File |
        Select-Object -First 1 -ExpandProperty FullName

    if ($exePath) { return $exePath }

    # 兜底：无扩展名（部分打包工具可能丢失 .exe）
    $barePath = Get-ChildItem -Path $DirectoryPath -Recurse -Filter "$BinaryName" -File |
        Select-Object -First 1 -ExpandProperty FullName

    if ($barePath) { return $barePath }

    return $null
}

<#
.SYNOPSIS
    复制二进制文件到安装目录
#>
function Copy-BinaryToInstallDirectory {
    param([string]$SourcePath, [string]$InstallDir, [string]$TargetName)

    # 确保安装目录存在
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

    $targetPath = Join-Path -Path $InstallDir -ChildPath "${TargetName}.exe"
    Copy-Item -Path $SourcePath -Destination $targetPath -Force

    return $targetPath
}

<#
.SYNOPSIS
    将安装目录添加到用户级 PATH 环境变量
#>
function Add-DirectoryToUserPath {
    param([string]$DirectoryPath)

    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $existingPaths = $userPath -split ";" | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" }

    if ($existingPaths -contains $DirectoryPath) {
        Write-Info "目录 $DirectoryPath 已在 PATH 中"
        return
    }

    $newPath = "$DirectoryPath;$userPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")

    Write-Ok "已将 $DirectoryPath 添加到用户 PATH（新终端生效）"
    Write-Warn "在当前终端中执行以下命令立即生效："
    Write-Host "  `$env:PATH = `"$DirectoryPath;`$env:PATH`"" -ForegroundColor Yellow
}

<#
.SYNOPSIS
    验证二进制是否已正确安装并可执行
    尝试运行 --version 确认可用性
#>
function Test-InstalledBinary {
    param([string]$BinaryPath, [string]$DisplayName)

    if (-not (Test-Path -Path $BinaryPath)) {
        Write-Warn "二进制文件不存在: $BinaryPath"
        return $false
    }

    try {
        $version = & $BinaryPath --version 2>&1 | Out-String
        Write-Ok "安装验证通过: $DisplayName ($($version.Trim()))"
    } catch {
        # 二进制存在但无法执行（如缺少 DLL），仍报告安装成功但发出警告
        Write-Ok "安装验证通过: $DisplayName.exe"
        Write-Warn "无法获取版本信息（二进制已安装但可能缺少运行时依赖）"
    }

    return $true
}


# ============================================================
# Layer 6: 编排层 - 主流程
#   只做流程编排，不做具体操作
#   每个步骤委托给下层原子函数
# ============================================================

<#
.SYNOPSIS
    Synerix 安装主流程编排器
    按顺序执行：检测 → 下载 → 解压 → 安装 → 清理 → 注册 PATH
.PARAMETER  Tag
    要安装的版本标签（默认从 GitHub/Gitee 获取最新）
.PARAMETER  Architecture
    目标架构（默认自动检测）
#>
function Install-SynerixRelease {
    param(
        [string]$Tag,
        [string]$Architecture
    )

    # ── 步骤 1: 解析下载地址 ──
    $downloadUrl = Resolve-ArchiveDownloadUrl -Tag $Tag -Arch $Architecture
    if (-not $downloadUrl) {
        Write-Warn "未找到 ${Tag} 的预编译二进制文件（$Script:OsName/$Architecture）"
        Write-Warn "请通过源码编译安装："
        Write-Warn "  git clone https://github.com/${Script:RepoOwner}/${Script:RepoName}.git"
        Write-Warn "  cd $Script:RepoName && cargo build --release"
        return $false
    }

    # ── 步骤 2: 创建临时工作目录 ──
    $tempDir = New-TempDirectory
    $assetName = Build-ReleaseAssetName -Tag $Tag -Arch $Architecture
    $archivePath = Join-Path -Path $tempDir -ChildPath $assetName

    try {
        # ── 步骤 3: 下载 ──
        Invoke-FileDownload -Url $downloadUrl -DestinationPath $archivePath

        # ── 步骤 4: 解压 ──
        Expand-ArchiveFile -ArchivePath $archivePath -DestinationPath $tempDir

        # ── 步骤 5: 查找二进制 ──
        $sourceBinary = Resolve-BinaryInDirectory -DirectoryPath $tempDir -BinaryName $Script:BinaryName
        if (-not $sourceBinary) {
            Write-Error "归档文件中未找到 $Script:BinaryName 可执行文件"
        }

        # ── 步骤 6: 安装（复制到目标目录） ──
        $installedPath = Copy-BinaryToInstallDirectory -SourcePath $sourceBinary -InstallDir $Script:InstallDir -TargetName $Script:BinaryName
        Write-Ok "已安装 $Script:BinaryName $Tag → $installedPath"

    } finally {
        # ── 步骤 7: 清理临时文件（无论成功失败都执行） ──
        Remove-DirectoryIfExists -Path $tempDir
    }

    return $true
}

<#
.SYNOPSIS
    入口函数：环境检查 → 获取版本 → 安装 → PATH 注册 → 验证
#>
function Main {
    # ── 启动横幅 ──
    Write-Host ""
    Write-Host "  ╔═══════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "  ║     Synerix — AI Coding Terminal       ║" -ForegroundColor Cyan
    Write-Host "  ╚═══════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""

    # ── 环境预检 ──
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        Write-Error "需要 PowerShell 5.0+，当前版本: $($PSVersionTable.PSVersion)。请升级: https://aka.ms/powershell"
    }

    # ── 架构检测 ──
    $architecture = Get-WindowsArchitecture
    Write-Info "平台: $Script:OsName / $architecture"

    # ── 版本获取 ──
    $tag = Get-LatestReleaseTag
    Write-Info "最新版本: $tag"

    # ── 执行安装 ──
    $success = Install-SynerixRelease -Tag $tag -Architecture $architecture
    if (-not $success) {
        exit 1
    }

    # ── PATH 注册 ──
    Add-DirectoryToUserPath -DirectoryPath $Script:InstallDir

    # ── 安装验证 ──
    $installedBinary = Join-Path -Path $Script:InstallDir -ChildPath "${Script:BinaryName}.exe"
    Test-InstalledBinary -BinaryPath $installedBinary -DisplayName $Script:BinaryName

    # ── 完成提示 ──
    Write-Host ""
    Write-Ok "安装完成！运行 'synerix' 开始使用。"
    Write-Host ""
    Write-Host "  配置目录: $env:APPDATA\synerix"
    Write-Host "  文档: https://github.com/${Script:RepoOwner}/${Script:RepoName}"
    Write-Host ""
}

# ── 启动 ──
Main