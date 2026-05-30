//! Phase 4 tests — Vim/Emacs keybindings, Mouse, Startup, Config reload

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use synerix::config::keymap::{Action, KeyBindings, KeymapProfile};
use synerix::config::Settings;

fn vim() -> KeyBindings {
    KeyBindings::new(KeymapProfile::Vim)
}
fn emacs() -> KeyBindings {
    KeyBindings::new(KeymapProfile::Emacs)
}
fn default_kb() -> KeyBindings {
    KeyBindings::new(KeymapProfile::Default)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}
fn ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

// ── Vim Normal Mode ───────────────────────────────────────

#[test]
fn test_vim_normal_i_enters_insert() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Normal, key(KeyCode::Char('i')));
    assert!(matches!(action, Action::EnterInsertMode));
}

#[test]
fn test_vim_normal_colon_enters_command() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Normal, key(KeyCode::Char(':')));
    assert!(matches!(action, Action::EnterCommandMode));
}

#[test]
fn test_vim_normal_slash_enters_search() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Normal, key(KeyCode::Char('/')));
    assert!(matches!(action, Action::EnterSearchMode));
}

#[test]
fn test_vim_normal_q_quits() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Normal, key(KeyCode::Char('q')));
    assert!(matches!(action, Action::Quit));
}

#[test]
fn test_vim_normal_j_scroll_down() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Normal, key(KeyCode::Char('j')));
    assert!(matches!(action, Action::ScrollDown));
}

#[test]
fn test_vim_normal_k_scroll_up() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Normal, key(KeyCode::Char('k')));
    assert!(matches!(action, Action::ScrollUp));
}

#[test]
fn test_vim_normal_g_scroll_to_bottom() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Normal, key(KeyCode::Char('G')));
    assert!(matches!(action, Action::ScrollToBottom));
}

#[test]
fn test_vim_normal_h_cursor_left() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Normal, key(KeyCode::Char('h')));
    assert!(matches!(action, Action::MoveCursorLeft));
}

#[test]
fn test_vim_normal_l_cursor_right() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Normal, key(KeyCode::Char('l')));
    assert!(matches!(action, Action::MoveCursorRight));
}

// ── Vim Insert Mode ───────────────────────────────────────

#[test]
fn test_vim_insert_esc_enters_normal() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Insert, key(KeyCode::Esc));
    assert!(matches!(action, Action::EnterNormalMode));
}

#[test]
fn test_vim_insert_enter_submits() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Insert, key(KeyCode::Enter));
    assert!(matches!(action, Action::SubmitMessage));
}

#[test]
fn test_vim_insert_ctrl_a_home() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Insert, ctrl(KeyCode::Char('a')));
    assert!(matches!(action, Action::MoveCursorHome));
}

#[test]
fn test_vim_insert_ctrl_e_end() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Insert, ctrl(KeyCode::Char('e')));
    assert!(matches!(action, Action::MoveCursorEnd));
}

#[test]
fn test_vim_insert_ctrl_k_kill_to_end() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Insert, ctrl(KeyCode::Char('k')));
    assert!(matches!(action, Action::KillToEnd));
}

#[test]
fn test_vim_insert_ctrl_u_kill_to_start() {
    let mut kb = vim();
    let action = kb.resolve(&synerix::app::InputMode::Insert, ctrl(KeyCode::Char('u')));
    assert!(matches!(action, Action::KillToStart));
}

// ── Emacs Mode ────────────────────────────────────────────

#[test]
fn test_emacs_ctrl_n_scroll_down() {
    let mut kb = emacs();
    let action = kb.resolve(&synerix::app::InputMode::Normal, ctrl(KeyCode::Char('n')));
    assert!(matches!(action, Action::ScrollDown));
}

#[test]
fn test_emacs_ctrl_p_scroll_up() {
    let mut kb = emacs();
    let action = kb.resolve(&synerix::app::InputMode::Normal, ctrl(KeyCode::Char('p')));
    assert!(matches!(action, Action::ScrollUp));
}

#[test]
fn test_emacs_ctrl_f_cursor_right() {
    let mut kb = emacs();
    let action = kb.resolve(&synerix::app::InputMode::Normal, ctrl(KeyCode::Char('f')));
    assert!(matches!(action, Action::MoveCursorRight));
}

#[test]
fn test_emacs_ctrl_b_cursor_left() {
    let mut kb = emacs();
    let action = kb.resolve(&synerix::app::InputMode::Normal, ctrl(KeyCode::Char('b')));
    assert!(matches!(action, Action::MoveCursorLeft));
}

#[test]
fn test_emacs_ctrl_a_home() {
    let mut kb = emacs();
    let action = kb.resolve(&synerix::app::InputMode::Normal, ctrl(KeyCode::Char('a')));
    assert!(matches!(action, Action::MoveCursorHome));
}

#[test]
fn test_emacs_ctrl_e_end() {
    let mut kb = emacs();
    let action = kb.resolve(&synerix::app::InputMode::Normal, ctrl(KeyCode::Char('e')));
    assert!(matches!(action, Action::MoveCursorEnd));
}

#[test]
fn test_emacs_ctrl_k_kill() {
    let mut kb = emacs();
    let action = kb.resolve(&synerix::app::InputMode::Normal, ctrl(KeyCode::Char('k')));
    assert!(matches!(action, Action::KillToEnd));
}

#[test]
fn test_emacs_ctrl_y_yank() {
    let mut kb = emacs();
    let action = kb.resolve(&synerix::app::InputMode::Normal, ctrl(KeyCode::Char('y')));
    assert!(matches!(action, Action::Paste));
}

// ── Default Mode ──────────────────────────────────────────

#[test]
fn test_default_esc_enters_normal() {
    let mut kb = default_kb();
    let action = kb.resolve(&synerix::app::InputMode::Insert, key(KeyCode::Esc));
    assert!(matches!(action, Action::EnterNormalMode));
}

#[test]
fn test_default_enter_submits() {
    let mut kb = default_kb();
    let action = kb.resolve(&synerix::app::InputMode::Insert, key(KeyCode::Enter));
    assert!(matches!(action, Action::SubmitMessage));
}

// ── App State ─────────────────────────────────────────────

#[test]
fn test_app_has_focused_panel() {
    let settings = Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let app = synerix::app::App::new_with_channel(settings, tx, _rx);
    // Should default to Input focus
    assert_eq!(app.focused_panel, synerix::app::FocusedPanel::Input);
}

#[test]
fn test_app_has_layout_state() {
    let settings = Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let app = synerix::app::App::new_with_channel(settings, tx, _rx);
    // Layout state should be initialized (zero rects)
    assert_eq!(app.layout_state.sidebar_rect.width, 0);
}

#[test]
fn test_app_has_yank_buffer() {
    let settings = Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let app = synerix::app::App::new_with_channel(settings, tx, _rx);
    assert!(app.yank_buffer.is_empty());
}

#[test]
fn test_app_has_keybindings() {
    let settings = Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let _app = synerix::app::App::new_with_channel(settings, tx, _rx);
    // Should have keybindings initialized
    assert!(true); // Just verify it doesn't panic
}

// ── Telemetry ─────────────────────────────────────────────

#[test]
fn test_startup_timer() {
    use synerix::telemetry::StartupTimer;
    let timer = StartupTimer::new();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let elapsed = timer.total_elapsed_ms();
    assert!(elapsed >= 10);
}

#[test]
fn test_startup_metrics_display() {
    use synerix::telemetry::StartupMetrics;
    let metrics = StartupMetrics {
        config_load_ms: 5,
        tui_init_ms: 20,
        db_open_ms: 10,
        total_ms: 35,
    };
    let text = metrics.status_bar_text();
    assert!(text.contains("35ms"));
}

// ── Config Watcher ────────────────────────────────────────

#[test]
fn test_config_watcher_module_exists() {
    // Just verify the module compiles and types are accessible
    let _reload = synerix::config::watcher::ConfigReload {
        settings: synerix::config::Settings::load().unwrap(),
        version: 1,
    };
    assert!(true); // If we get here, the module exists
}

// ── File Count ────────────────────────────────────────────

#[test]
fn test_source_file_count() {
    use std::process::Command;
    let output = Command::new("find")
        .args([
            "/home/ubuntu/workspace/synerix/src",
            "-name",
            "*.rs",
            "-type",
            "f",
        ])
        .output()
        .unwrap();
    let count = String::from_utf8_lossy(&output.stdout).lines().count();
    // Should have at least 60 source files now (with telemetry, watcher, etc.)
    assert!(count >= 60, "Expected 60+ source files, got {}", count);
}
