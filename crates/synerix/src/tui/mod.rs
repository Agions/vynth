//! TUI rendering layer

pub mod activity_label;
pub mod diff_renderer;
pub mod event;
pub mod layout;
pub mod renderer;
pub mod syntax;
pub mod theme;
pub mod widgets;

use crate::error::AppError;

/// Initialize the terminal (raw mode, alternate screen, mouse capture)
pub fn init(
) -> Result<ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>, AppError> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    crossterm::execute!(stdout, crossterm::event::EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let terminal = ratatui::Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal mode
pub fn restore(
    mut terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> Result<(), AppError> {
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::event::DisableMouseCapture
    )?;
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}
