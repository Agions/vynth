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
use crate::model_catalog::infer_model_capabilities;
use crate::slash::common::{provider_display, sys_msg};

struct ModelPreset {
    key: &'static str,
    provider: Provider,
    model: &'static str,
    base_url: Option<&'static str>,
    temperature: f32,
    desc: &'static str,
}

const MODEL_PRESETS: &[ModelPreset] = &[
    ModelPreset {
        key: "deepseek",
        provider: Provider::DeepSeek,
        model: "deepseek-v4-flash",
        base_url: None,
        temperature: 0.7,
        desc: "default coding model, OpenAI-compatible endpoint",
    },
    ModelPreset {
        key: "mimo",
        provider: Provider::MiMo,
        model: "mimo-v2.5-pro",
        base_url: None,
        temperature: 0.7,
        desc: "MiMo provider preset",
    },
    ModelPreset {
        key: "openai",
        provider: Provider::Custom {
            base_url: String::new(),
        },
        model: "gpt-4.1",
        base_url: Some("https://api.openai.com/v1"),
        temperature: 0.7,
        desc: "OpenAI-compatible preset, set SYNERIX_API_KEY",
    },
];

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
            if trimmed == "list" || trimmed == "presets" {
                model_list_presets(app);
            } else if let Some(rest) = trimmed.strip_prefix("use ") {
                model_use_preset(app, rest.trim());
            } else if trimmed == "custom" {
                // 只有 custom 关键字但缺少后续参数
                sys_msg(
                    app,
                    "❌ 用法：`/model custom <model-name> <base-url>`\n例如：`/model custom gpt-4o https://api.openai.com/v1`",
                );
            } else if let Some(rest) = trimmed.strip_prefix("custom ") {
                model_handle_custom(app, rest.trim());
            } else if let Some(rest) = trimmed.strip_prefix("temp ") {
                model_set_temperature(app, rest.trim());
            } else if let Some(rest) = trimmed.strip_prefix("tokens ") {
                model_set_output_tokens(app, rest.trim());
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

fn model_list_presets(app: &mut App) {
    let mut lines = vec!["可用大模型方案：".to_string(), String::new()];
    for preset in MODEL_PRESETS {
        let caps = infer_model_capabilities(preset.model);
        let context_window =
            caps.map_or(app.settings.llm.context_window, |caps| caps.context_window);
        let max_output_tokens = caps.map_or(app.settings.llm.max_output_tokens, |caps| {
            caps.max_output_tokens
        });
        lines.push(format!(
            "  `{}` -> `{}` | ctx {} | out {} | temp {:.1}",
            preset.key, preset.model, context_window, max_output_tokens, preset.temperature
        ));
        lines.push(format!("     {}", preset.desc));
    }
    lines.push(String::new());
    lines.push("用法：`/model use <方案>`、`/model custom <model> <base-url>`、`/model temp <0.0-2.0>`、`/model tokens <n>`".to_string());
    sys_msg(app, &lines.join("\n"));
}

fn model_use_preset(app: &mut App, key: &str) {
    let Some(preset) = MODEL_PRESETS.iter().find(|preset| preset.key == key) else {
        sys_msg(app, "未知模型方案。输入 `/model list` 查看可用方案。");
        return;
    };

    app.settings.llm.provider = match &preset.provider {
        Provider::DeepSeek => Provider::DeepSeek,
        Provider::MiMo => Provider::MiMo,
        Provider::Custom { .. } => Provider::Custom {
            base_url: preset.base_url.unwrap_or_default().to_string(),
        },
    };
    app.settings.llm.base_url = preset.base_url.map(str::to_string);
    app.settings.llm.model = preset.model.to_string();
    app.settings.llm.apply_model_capabilities();
    app.settings.llm.temperature = preset.temperature;
    sync_model_status(app);

    sys_msg(
        app,
        &format!(
            "已切换模型方案 `{}`：\n  模型：`{}`\n  提供商：{}\n  上下文：{}\n  输出上限：{}\n  温度：{:.1}",
            preset.key,
            preset.model,
            provider_display(&app.settings.llm.provider),
            app.settings.llm.context_window,
            app.settings.llm.max_output_tokens,
            preset.temperature
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
    app.settings.llm.base_url = Some(base_url.to_string());
    app.settings.llm.model = model_name.to_string();
    app.settings.llm.apply_model_capabilities();
    sync_model_status(app);
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
    app.settings.llm.apply_model_capabilities();
    sync_model_status(app);
    sys_msg(app, &format!("✅ 模型已切换：`{}` → `{}`", old, name));
}

fn model_set_temperature(app: &mut App, raw: &str) {
    match raw.parse::<f32>() {
        Ok(value) if (0.0..=2.0).contains(&value) => {
            app.settings.llm.temperature = value;
            sys_msg(app, &format!("模型温度已设置为 `{value:.2}`"));
        }
        _ => sys_msg(app, "用法：`/model temp <0.0-2.0>`"),
    }
}

fn model_set_output_tokens(app: &mut App, raw: &str) {
    match raw.parse::<usize>() {
        Ok(value) if value > 0 => {
            app.settings.llm.max_output_tokens = value;
            sync_model_status(app);
            sys_msg(app, &format!("模型输出上限已设置为 `{value}` tokens"));
        }
        _ => sys_msg(app, "用法：`/model tokens <n>`"),
    }
}

fn sync_model_status(app: &mut App) {
    app.status_bar.model_name = app.settings.llm.model.clone();
    app.status_bar.tokens_total = app.settings.llm.context_window;
}
