mod game;
mod renderer;
mod songs;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use game::{Game, GamePhase};

const TARGET_FPS: u64 = 60;

fn main() -> io::Result<()> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_game(&mut terminal);

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    result
}

fn run_game(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut game = Game::new();
    let frame_duration = Duration::from_micros(1_000_000 / TARGET_FPS);

    loop {
        let frame_start = Instant::now();

        // --- Input ---
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => match game.phase {
                            GamePhase::Playing => game.phase = GamePhase::Results,
                            _ => return Ok(()),
                        },
                        KeyCode::Char('q') if game.phase != GamePhase::Playing => {
                            return Ok(());
                        }
                        _ => game.handle_key(key_event.code),
                    }
                }
            }
        }

        // --- Update ---
        game.update();

        if game.phase == GamePhase::Quit {
            return Ok(());
        }

        // --- Render ---
        terminal.draw(|frame| {
            renderer::render(frame, &game);
        })?;

        // --- Frame timing ---
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}
