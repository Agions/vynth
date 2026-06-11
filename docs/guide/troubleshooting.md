# 故障排除

本指南帮助你解决常见的 Synerix 问题。

## 常见问题

### 安装问题

#### 问题: 安装脚本失败

**症状**: 运行安装脚本时出现错误。

**解决方案**:

```bash
# 检查网络连接
curl -I https://github.com

# 使用 sudo 运行（Linux）
sudo curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash

# 手动下载安装
wget https://github.com/Agions/synerix/releases/latest/download/synerix-linux-x86_64.tar.gz
tar -xzf synerix-linux-x86_64.tar.gz
sudo mv synerix /usr/local/bin/
```

#### 问题: 权限错误

**症状**: `Permission denied` 错误。

**解决方案**:

```bash
# 设置可执行权限
chmod +x ~/.local/bin/synerix

# 或者添加到 PATH
export PATH="$HOME/.local/bin:$PATH"
```

### 运行问题

#### 问题: 启动失败

**症状**: 运行 `synerix` 时无响应或崩溃。

**解决方案**:

```bash
# 检查日志
synerix --log-level debug

# 重置配置
synerix config reset

# 检查依赖
synerix --check-deps
```

#### 问题: AI 连接失败

**症状**: 无法连接到 AI 服务。

**解决方案**:

```bash
# 检查 API 密钥
synerix config show

# 测试连接
synerix --test-connection

# 检查网络
curl https://api.openai.com/v1/models
```

### 性能问题

#### 问题: 响应缓慢

**症状**: AI 响应时间过长。

**解决方案**:

```bash
# 使用更快的模型
synerix config set model gpt-3.5-turbo

# 限制上下文长度
synerix config set max_context 4096

# 启用缓存
synerix config set cache_enabled true
```

### 终端问题

#### 问题: 显示异常

**症状**: 终端显示乱码或格式错误。

**解决方案**:

```bash
# 检查终端类型
echo $TERM

# 设置正确的终端
export TERM=xterm-256color

# 更新终端字体
synerix config set font_family "Fira Code"
```

## 调试模式

```bash
# 启用详细日志
synerix --log-level debug --log-file /tmp/synerix.log

# 运行诊断
synerix --diagnose

# 检查系统信息
synerix --system-info
```

## 获取帮助

### 社区支持

- **GitHub Issues**: https://github.com/Agions/synerix/issues
- **Discussions**: https://github.com/Agions/synerix/discussions

### 日志文件

日志文件位置：

- **Linux/macOS**: `~/.local/share/synerix/logs/`
- **Windows**: `%APPDATA%\synerix\logs\`

### 报告问题

报告问题时请包含：

1. Synerix 版本 (`synerix --version`)
2. 操作系统和版本
3. 终端类型
4. 错误信息和日志
5. 重现步骤

## 下一步

- [API 文档](/api/overview) - 详细了解 API
- [配置](/guide/configuration) - 自定义你的 Synerix
