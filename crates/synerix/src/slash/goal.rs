//! `/goal` 命令 — 设置/查看/清除完成目标
//!
//! # 优化说明
//! - 将原 95 行单函数拆分为 4 个独立函数：
//!   `goal_handle_clear`, `goal_show_status`, `goal_set_condition`
//! - 与 /clear, /reset 不同的是，/goal 涉及 GoalState、
//!   status_bar、dirty_flags 多个字段的联动，保持独立模块

use crate::app::{App, ChatMessage, GoalState, MessageRole};
use crate::slash::common::sys_msg;

/// 处理 `/goal` 命令
pub fn cmd_goal(app: &mut App, args: Option<&str>) -> bool {
    let args = args.unwrap_or("").trim();

    // ── /goal clear 系列别名 ──
    if is_clear_command(args) {
        goal_handle_clear(app);
        return true;
    }

    // ── /goal (无参数) — 查看状态 ──
    if args.is_empty() {
        goal_show_status(app);
        return true;
    }

    // ── /goal <condition> — 设置新目标 ──
    goal_set_condition(app, args);
    true
}

/// 判断是否为清除命令
fn is_clear_command(args: &str) -> bool {
    matches!(args, "clear" | "stop" | "off" | "reset" | "none" | "cancel")
}

/// 清除活跃目标
fn goal_handle_clear(app: &mut App) {
    if app.goal_state.is_active() {
        app.goal_state = GoalState::inactive();
        app.status_bar.goal_active = false;
        app.status_bar.goal_duration = String::new();
        app.dirty_flags.status = true;
        sys_msg(app, "◎ /goal 已清除");
    } else {
        sys_msg(app, "当前没有活跃的 /goal");
    }
}

/// 显示目标状态
fn goal_show_status(app: &mut App) {
    let gs = &app.goal_state;
    if gs.is_active() {
        sys_msg(
            app,
            &format!(
                "◎ /goal 活跃中\n  条件: {}\n  已运行: {} | 轮次: {} | 最近评估: {}",
                gs.condition.as_deref().unwrap_or(""),
                gs.duration_str(),
                gs.turns,
                if gs.last_reason.is_empty() {
                    "等待首次评估…"
                } else {
                    gs.last_reason.as_str()
                },
            ),
        );
    } else if gs.achieved {
        sys_msg(
            app,
            &format!(
                "✅ /goal 已完成\n  条件: {}\n  轮次: {}",
                gs.condition.as_deref().unwrap_or(""),
                gs.turns,
            ),
        );
    } else {
        sys_msg(
            app,
            "当前没有活跃的 /goal。使用 `/goal <条件>` 设置一个完成目标。\n  示例: `/goal all tests in test/auth pass`",
        );
    }
}

/// 设置新目标条件
fn goal_set_condition(app: &mut App, condition: &str) {
    let condition = condition.to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // 更新 GoalState
    app.goal_state = GoalState {
        condition: Some(condition.clone()),
        turns: 0,
        started_at: Some(now),
        last_reason: String::new(),
        achieved: false,
    };

    // 同步 UI 状态
    app.status_bar.goal_active = true;
    app.status_bar.goal_duration = String::new();
    app.dirty_flags.status = true;

    // 注入目标消息触发 Agent 循环
    app.chat_state.messages.push(ChatMessage {
        role: MessageRole::User,
        content: format!(
            "🎯 目标: {}\n\n请持续工作直到上述条件满足。每完成一轮，我会检查条件并决定是否继续。",
            condition,
        ),
        tool_calls: Vec::new(),
    });

    sys_msg(
        app,
        &format!(
            "◎ /goal 已设置: {}\nAgent 将持续工作直到条件满足。使用 `/goal` 查看状态，`/goal clear` 清除。",
            condition,
        ),
    );
}
