//! `/skill` 命令 — 技能目录与技能源管理
//!
//! # 优化说明
//! - 原 cmd_skill 的 match 拆分为：skill_show_status, skill_set_dir,
//!   skill_handle_source, skill_handle_unknown 四个函数
//! - cmd_skill_source 的 match 拆分为：skill_source_list,
//!   skill_source_add, skill_source_remove 三个函数
//! - 提取 `format_skill_status` 消除 cmd_skill 与 cmd_skill_source
//!   中重复的「技能源列表格式化」逻辑

use crate::app::App;
use crate::config::SkillSourceConfig;
use crate::slash::common::{nth_arg, subcmd, sys_msg};

/// 处理 `/skill` 命令
pub fn cmd_skill(app: &mut App, args: Option<&str>) -> bool {
    let (sub, rest) = subcmd(args);

    match sub {
        "" => skill_show_status(app),
        "dir" => skill_set_dir(app, rest),
        "source" => {
            cmd_skill_source(app, rest);
        }
        other => skill_handle_unknown(app, other),
    }
    true
}

/// 显示技能目录和技能源状态
fn skill_show_status(app: &mut App) {
    let dir_info = match &app.settings.skills_dir {
        Some(path) => format!("`{}`", path.display()),
        None => "未设置（使用默认路径）".to_string(),
    };
    let mut lines = vec![format!("📂 Skills 目录：{}", dir_info), String::new()];

    if app.settings.skill_sources.is_empty() {
        lines.push("技能源：无".to_string());
    } else {
        lines.push(format!(
            "技能源（{} 个）：",
            app.settings.skill_sources.len()
        ));
        for (i, src) in app.settings.skill_sources.iter().enumerate() {
            let branch = src
                .branch
                .as_ref()
                .map(|b| format!(" @{}", b))
                .unwrap_or_default();
            lines.push(format!(
                "  {}. [{}] {}{}",
                i + 1,
                src.source_type,
                src.location,
                branch
            ));
        }
    }
    lines.push(String::new());
    lines.push("用法：".to_string());
    lines.push("  `/skill` — 查看当前状态".to_string());
    lines.push("  `/skill dir <path>` — 设置 Skills 目录".to_string());
    lines.push("  `/skill source list` — 列出技能源".to_string());
    lines.push("  `/skill source add <type> <location> [branch]` — 添加技能源".to_string());
    lines.push("  `/skill source remove <index>` — 移除技能源".to_string());
    sys_msg(app, &lines.join("\n"));
}

/// 设置 Skills 目录路径
fn skill_set_dir(app: &mut App, rest: Option<&str>) {
    let path = match nth_arg(rest, 0) {
        Some(p) => p,
        None => {
            sys_msg(app, "❌ 用法：`/skill dir <path>`");
            return;
        }
    };
    app.settings.skills_dir = Some(std::path::PathBuf::from(path));
    sys_msg(app, &format!("📂 Skills 目录已设置为：`{}`", path));
}

/// 处理未知的 /skill 子命令
fn skill_handle_unknown(app: &mut App, other: &str) {
    sys_msg(
        app,
        &format!("❌ 未知子命令 `{}`。输入 `/skill` 查看用法。", other),
    );
}

/// 处理 `/skill source <subcommand>`
pub fn cmd_skill_source(app: &mut App, args: Option<&str>) -> bool {
    let (sub, rest) = subcmd(args);

    match sub {
        "" | "list" => skill_source_list(app),
        "add" => skill_source_add(app, rest),
        "remove" => skill_source_remove(app, rest),
        other => {
            sys_msg(
                app,
                &format!(
                    "❌ 未知子命令 `{}`。用法：`/skill source list`、`/skill source add <type> <location>`、`/skill source remove <index>`",
                    other
                ),
            );
        }
    }
    true
}

/// 列出技能源
fn skill_source_list(app: &mut App) {
    let dir_info = match &app.settings.skills_dir {
        Some(path) => format!("`{}`", path.display()),
        None => "未设置".to_string(),
    };
    let mut lines = vec![format!("📂 Skills 目录：{}", dir_info), String::new()];
    if app.settings.skill_sources.is_empty() {
        lines.push("技能源：无".to_string());
    } else {
        lines.push(format!(
            "技能源（{} 个）：",
            app.settings.skill_sources.len()
        ));
        for (i, src) in app.settings.skill_sources.iter().enumerate() {
            let branch = src
                .branch
                .as_ref()
                .map(|b| format!(" @{}", b))
                .unwrap_or_default();
            lines.push(format!(
                "  {}. [{}] {}{}",
                i + 1,
                src.source_type,
                src.location,
                branch
            ));
        }
    }
    sys_msg(app, &lines.join("\n"));
}

/// 添加技能源
fn skill_source_add(app: &mut App, rest: Option<&str>) {
    let source_type = match nth_arg(rest, 0) {
        Some(t) => t,
        None => {
            sys_msg(
                app,
                "❌ 用法：`/skill source add <type> <location> [branch]`\n类型：`git`、`local`、`url`\n例如：`/skill source add git https://github.com/user/skills main`",
            );
            return;
        }
    };
    let location = match nth_arg(rest, 1) {
        Some(l) => l.to_string(),
        None => {
            sys_msg(
                app,
                "❌ 用法：`/skill source add <type> <location> [branch]`\n类型：`git`、`local`、`url`\n例如：`/skill source add git https://github.com/user/skills main`",
            );
            return;
        }
    };
    let branch = nth_arg(rest, 2)
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    let branch_info = branch
        .as_ref()
        .map(|b| format!(" @{}", b))
        .unwrap_or_default();
    sys_msg(
        app,
        &format!(
            "✅ 已添加技能源：`[{}] {}{}`",
            source_type, location, branch_info
        ),
    );

    app.settings.skill_sources.push(SkillSourceConfig {
        source_type: source_type.to_string(),
        location,
        branch,
        include: Vec::new(),
        exclude: Vec::new(),
    });
}

/// 移除技能源（按 1-indexed 序号）
fn skill_source_remove(app: &mut App, rest: Option<&str>) {
    let idx_str = match nth_arg(rest, 0) {
        Some(s) => s,
        None => {
            sys_msg(app, "❌ 用法：`/skill source remove <index>`");
            return;
        }
    };
    match idx_str.parse::<usize>() {
        Ok(idx) if idx > 0 && idx <= app.settings.skill_sources.len() => {
            let removed = app.settings.skill_sources.remove(idx - 1);
            sys_msg(
                app,
                &format!(
                    "🗑️ 已移除技能源 #{}：`[{}] {}`",
                    idx, removed.source_type, removed.location
                ),
            );
        }
        Ok(_) | Err(_) => {
            sys_msg(
                app,
                &format!(
                    "❌ 无效索引。有效范围：1~{}",
                    app.settings.skill_sources.len()
                ),
            );
        }
    }
}
