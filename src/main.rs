mod app;
mod markdown;
mod render;
mod toc;
mod search;
mod watcher;
mod syntax;

use std::io::{self, Read as IoRead};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Mode};

#[derive(Parser, Debug)]
#[command(name = "md-view", about = "Terminal Markdown viewer with phosphor-green theme")]
struct Cli {
    /// Markdown file to view
    file: Option<PathBuf>,

    /// Watch file for changes and auto-reload
    #[arg(short, long)]
    watch: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let content = if let Some(ref path) = cli.file {
        std::fs::read_to_string(path)?
    } else {
        // Check if stdin is a TTY
        if crossterm::terminal::is_raw_mode_enabled().unwrap_or(false) || is_stdin_tty() {
            eprintln!("Usage: md-view <file.md> or pipe Markdown via stdin");
            std::process::exit(1);
        }
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let filename = cli
        .file
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "stdin".into());

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (reload_tx, reload_rx) = mpsc::channel::<String>();
    let _watcher_guard = if cli.watch {
        if let Some(ref path) = cli.file {
            Some(watcher::start_watching(path.clone(), reload_tx)?)
        } else {
            None
        }
    } else {
        None
    };

    let mut app = App::new(content, filename);

    let result = run_app(&mut terminal, &mut app, &reload_rx);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn is_stdin_tty() -> bool {
    #[cfg(unix)]
    {
        extern "C" {
            fn isatty(fd: i32) -> i32;
        }
        unsafe { isatty(0) != 0 }
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    reload_rx: &mpsc::Receiver<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| render::draw(f, app))?;

        // Check for file reload
        if let Ok(new_content) = reload_rx.try_recv() {
            app.reload(new_content);
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(app, key) {
                        break;
                    }
                }
                Event::Resize(_, _) => {
                    // Will be handled on next draw
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    match app.mode {
        Mode::Normal => handle_normal_key(app, key),
        Mode::Search => handle_search_key(app, key),
        Mode::Toc => handle_toc_key(app, key),
    }
}

fn handle_normal_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Char('j') | KeyCode::Down => app.scroll_down(1),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_up(1),
        KeyCode::PageDown => {
            let page = app.viewport_height.saturating_sub(2);
            app.scroll_down(page);
        }
        KeyCode::PageUp => {
            let page = app.viewport_height.saturating_sub(2);
            app.scroll_up(page);
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let half = app.viewport_height / 2;
            app.scroll_down(half);
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let half = app.viewport_height / 2;
            app.scroll_up(half);
        }
        KeyCode::Char('g') => app.scroll_to_top(),
        KeyCode::Char('G') => app.scroll_to_bottom(),
        KeyCode::Char('/') => {
            app.mode = Mode::Search;
            app.search_input.clear();
        }
        KeyCode::Char('n') => app.next_match(),
        KeyCode::Char('N') => app.prev_match(),
        KeyCode::Char('h') => app.toggle_raw(),
        KeyCode::Tab => app.toggle_toc(),
        _ => {}
    }
    false
}

fn handle_search_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter => {
            app.execute_search();
            app.mode = Mode::Normal;
        }
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.search_input.clear();
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
        }
        KeyCode::Backspace => {
            app.search_input.pop();
        }
        _ => {}
    }
    false
}

fn handle_toc_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Tab | KeyCode::Esc => app.toggle_toc(),
        KeyCode::Char('q') => return true,
        KeyCode::Char('j') | KeyCode::Down => app.toc_cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => app.toc_cursor_up(),
        KeyCode::Enter => {
            app.jump_to_toc_entry();
            app.toggle_toc();
        }
        _ => {}
    }
    false
}
