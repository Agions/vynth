//! `/workflow` 命令 — 工作流管理
//!
//! # 优化说明
//! 当前为 stub 实现，独立成模块便于后续功能扩展。

use crate::app::App;
use crate::slash::common::sys_msg;

/// 处理 `/workflow` 命令
pub fn cmd_workflow(app: &mut App, args: Option<&str>) -> bool {
    match args {
        None => {
            sys_msg(
                app,
                "可用工作流：`code-review`、`refactor`、`debug`\n\n用法：`/workflow <name>`",
            );
        }
        Some("") => {
            sys_msg(app, "❌ 请指定工作流名称。用法：`/workflow <name>`");
        }
        Some(name) => {
            sys_msg(app, &format!("🚀 启动工作流：`{}`（功能开发中）", name));
        }
    }
    true
}
