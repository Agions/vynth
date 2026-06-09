//! `/config` 命令 — 配置查看与保存
//!
//! # 优化说明
//! 简短的 show/save 双分支，独立成单一职责模块。

use crate::app::App;
use crate::slash::common::{subcmd, sys_msg};

/// 处理 `/config` 命令
pub fn cmd_config(app: &mut App, args: Option<&str>) -> bool {
    let (sub, _rest) = subcmd(args);

    match sub {
        "" | "show" => {
            let path = crate::config::settings::Settings::config_path();
            sys_msg(
                app,
                &format!(
                    "⚙️ 配置路径：`{}`\nMCP 服务器：{} 个\n技能源：{} 个\nSkills 目录：{}\n\n用法：\n  `/config show` — 显示配置路径\n  `/config save` — 保存当前配置到文件",
                    path.display(),
                    app.settings.mcp.len(),
                    app.settings.skill_sources.len(),
                    app.settings
                        .skills_dir
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "未设置".to_string())
                ),
            );
        }
        "save" => match app.settings.save() {
            Ok(()) => {
                let path = crate::config::settings::Settings::config_path();
                sys_msg(app, &format!("💾 配置已保存到：`{}`", path.display()));
            }
            Err(e) => {
                sys_msg(app, &format!("❌ 保存配置失败：{}", e));
            }
        },
        other => {
            sys_msg(
                app,
                &format!(
                    "❌ 未知子命令 `{}`。用法：`/config show`、`/config save`",
                    other
                ),
            );
        }
    }
    true
}
