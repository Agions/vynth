//! 会话管理命令 — `/clear`, `/reset`, `/exit`
//!
//! # 优化说明
//! 三个短小命令集中管理。/clear 和 /reset 共享
//! `clear_chat_state()` 辅助函数消除 3 行重复的 state 重置代码。

use crate::app::App;
use crate::slash::common::{sys_msg, DEFAULT_MODEL};

/// 清空 chat_state 的内置状态（不关模型）
fn clear_chat_state(app: &mut App) {
    app.chat_state.messages.clear();
    app.chat_state.streaming_text.clear();
    app.chat_state.scroll_offset = 0;
}

/// 处理 `/clear` — 仅清空对话，保留模型配置
pub fn cmd_clear(app: &mut App, _args: Option<&str>) -> bool {
    clear_chat_state(app);
    sys_msg(app, "✅ 对话已清空。");
    true
}

/// 处理 `/reset` — 清空对话 + 恢复默认模型
pub fn cmd_reset(app: &mut App, _args: Option<&str>) -> bool {
    clear_chat_state(app);
    app.settings.llm.model = DEFAULT_MODEL.to_string();
    app.status_bar.model_name = DEFAULT_MODEL.to_string();
    sys_msg(
        app,
        &format!("🔄 对话已重置，模型恢复为 `{}`。", DEFAULT_MODEL),
    );
    true
}

/// 处理 `/exit` — 标记退出
pub fn cmd_exit(app: &mut App, _args: Option<&str>) -> bool {
    app.should_quit = true;
    true
}
