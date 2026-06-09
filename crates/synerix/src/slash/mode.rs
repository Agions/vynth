//! `/mode` 命令 — 编码模式切换
//!
//! # 优化说明
//! 将显示和切换两个职责拆分为独立函数。
//! 解析 (subcmd) 与切换 (CodingMode::parse) 解耦。

use crate::app::App;
use crate::coding_modes::CodingMode;
use crate::slash::common::{subcmd, sys_msg};

/// 处理 `/mode` 命令
pub fn cmd_mode(app: &mut App, args: Option<&str>) -> bool {
    let args = args.map(|a| a.trim()).unwrap_or("");

    if args.is_empty() {
        mode_show_current(app);
        return true;
    }

    let (subcmd_val, _) = subcmd(Some(args));
    mode_try_switch(app, subcmd_val);
    true
}

/// 显示当前编码模式及所有可用模式
fn mode_show_current(app: &mut App) {
    let mut lines = vec![format!("当前编码模式：{}", app.coding_mode)];
    lines.push("".to_string());
    for mode in CodingMode::all() {
        let mark = if *mode == app.coding_mode {
            " ✅ "
        } else {
            "   "
        };
        lines.push(format!(
            "{} {} — {}",
            mark,
            mode.label(),
            mode.description()
        ));
    }
    sys_msg(app, &lines.join("\n"));
}

/// 尝试切换到新模式
fn mode_try_switch(app: &mut App, name: &str) {
    if let Some(new_mode) = CodingMode::parse(name) {
        app.coding_mode = new_mode;
        app.status_bar.coding_mode = new_mode;
        app.dirty_flags.status = true;
        sys_msg(
            app,
            &format!(
                "✅ 编码模式已切换为：{} — {}",
                new_mode.label(),
                new_mode.description()
            ),
        );
    } else {
        sys_msg(
            app,
            &format!(
                "❌ 未知模式 `{}`。可用模式：plan, act, chat, architect",
                name
            ),
        );
    }
}
