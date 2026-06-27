//! `/help` 命令 — 分类查看 / 详细用法
//!
//! # 优化说明
//! 从 mod.rs 独立出来，单一职责：只做命令帮助信息的展示。
//! 分类顺序在 cat_order 数组中集中管理，便于扩展。

use crate::app::App;
use crate::slash::common::sys_msg;
use crate::slash::{find_cmd, CmdCategory, CmdDef, COMMANDS};

/// 处理 `/help` 命令
pub fn cmd_help(app: &mut App, args: Option<&str>) -> bool {
    let (target, _) = super::common::subcmd(args);

    if !target.is_empty() {
        return show_cmd_detail(app, target);
    }

    show_categorized_overview(app);
    true
}

/// 显示单个命令的详细用法
fn show_cmd_detail(app: &mut App, target: &str) -> bool {
    let lookup = if target.starts_with('/') {
        target
    } else {
        // 允许 /help model（带或不带 / 前缀）
        return cmd_help(app, Some(&format!("/{}", target)));
    };

    if let Some(cmd) = find_cmd(lookup) {
        let aliases_str = if cmd.aliases.is_empty() {
            String::new()
        } else {
            format!("\n  别名：{}", cmd.aliases.join(", "))
        };
        sys_msg(
            app,
            &format!(
                "{} - {}{}\n\n用法: {}",
                cmd.name, cmd.desc, aliases_str, cmd.usage,
            ),
        );
    } else {
        sys_msg(
            app,
            &format!("ERROR 没有 {} 命令。输入 /help 查看所有命令。", target),
        );
    }
    true
}

/// 按分类展示所有命令的概览
fn show_categorized_overview(app: &mut App) {
    let cat_order = [
        CmdCategory::Help,
        CmdCategory::Session,
        CmdCategory::Model,
        CmdCategory::Config,
        CmdCategory::Goal,
        CmdCategory::Workflow,
        CmdCategory::Mode,
    ];

    let mut lines = vec!["可用斜杠命令:".to_string(), String::new()];
    for cat in &cat_order {
        let cmds: Vec<&CmdDef> = COMMANDS.iter().filter(|c| c.category == *cat).collect();
        if cmds.is_empty() {
            continue;
        }
        lines.push(cat.label().to_string());
        for cmd in &cmds {
            lines.push(format!("  {} - {}", cmd.name, cmd.desc));
        }
        lines.push(String::new());
    }
    lines.push("TIP 使用 /help <命令名> 查看单个命令的详细用法和别名。".to_string());
    sys_msg(app, &lines.join("\n"));
}
