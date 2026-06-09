//! `/model` 命令 — 切换 / 配置 LLM 模型
//!
//! # 优化说明
//! 将复杂的 match 分支拆分为三个独立函数：
//! - `model_show_current` — 显示当前模型
//! - `model_handle_custom` — 配置自定义模型
//! - `model_switch_name` — 直接切换模型名
//!
//! 消除 cmd_model 单函数 58 行的长路径问题。

use crate::app::App;
use crate::config::Provider;
use crate::slash::common::{provider_display, sys_msg};

/// 处理 `/model` 命令
pub fn cmd_model(app: &mut App, args: Option<&str>) -> bool {
    match args {
        None => {
            // ── 无参数：显示当前模型 ──
            model_show_current(app);
        }
        Some("") => {
            sys_msg(
                app,
                "❌ 请指定参数。用法：`/model <name>` 或 `/model custom <name> <base-url>`",
            );
        }
        Some(args) => {
            let trimmed = args.trim();
            if trimmed == "custom" {
                // 只有 custom 关键字但缺少后续参数
                sys_msg(
                    app,
                    "❌ 用法：`/model custom <model-name> <base-url>`\n例如：`/model custom gpt-4o https://api.openai.com/v1`",
                );
            } else if let Some(rest) = trimmed.strip_prefix("custom ") {
                model_handle_custom(app, rest.trim());
            } else {
                model_switch_name(app, trimmed);
            }
        }
    }
    true
}

/// 显示当前模型及提供商信息
fn model_show_current(app: &mut App) {
    let provider_str = provider_display(&app.settings.llm.provider);
    sys_msg(
        app,
        &format!(
            "当前模型：`{}`\n提供商：{}\n\n用法：\n  `/model <name>` — 切换模型名称\n  `/model custom <name> <base-url>` — 配置自定义模型",
            app.settings.llm.model, provider_str
        ),
    );
}

/// 处理 `custom <name> <base-url>` 子命令
fn model_handle_custom(app: &mut App, rest: &str) {
    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
    if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
        sys_msg(
            app,
            "❌ 用法：`/model custom <model-name> <base-url>`\n例如：`/model custom gpt-4o https://api.openai.com/v1`",
        );
        return;
    }
    let model_name = parts[0].trim();
    let base_url = parts[1].trim();
    app.settings.llm.provider = Provider::Custom {
        base_url: base_url.to_string(),
    };
    app.settings.llm.model = model_name.to_string();
    app.status_bar.model_name = model_name.to_string();
    let provider_str = provider_display(&app.settings.llm.provider);
    sys_msg(
        app,
        &format!(
            "✅ 已配置自定义模型：\n  模型：`{}`\n  提供商：{}\n  API Base URL：`{}`",
            model_name, provider_str, base_url
        ),
    );
}

/// 直接切换模型名（不改变提供商类型）
fn model_switch_name(app: &mut App, name: &str) {
    let old = std::mem::replace(&mut app.settings.llm.model, name.to_string());
    app.status_bar.model_name = name.to_string();
    sys_msg(app, &format!("✅ 模型已切换：`{}` → `{}`", old, name));
}
