use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use std::io;

mod termsweeper;
mod tui;

static LAZY_REDRAW: bool = true;
static TITLE_SCREEN_CONTENT: &str = include_str!("../assets/title.in");
fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = TermsweeperApp::new().run(&mut terminal);
    tui::restore()?;
    app_result
}

struct TermsweeperApp {
    exit: bool,
    app_state: termsweeper::AppState,
    game: Option<termsweeper::Termsweeper>,
}

impl TermsweeperApp {
    fn new() -> TermsweeperApp {
        TermsweeperApp {
            exit: false,
            app_state: termsweeper::AppState::TitleScreen,
            game: None,
        }
    }
    fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(120),
                Constraint::Min(0),
            ])
            .split(frame.size());
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(42),
                Constraint::Min(0),
            ])
            .split(horizontal_layout[1]);
        frame.render_widget(self, vertical_layout[1]);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        loop {
            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    let event_handled = match self.app_state {
                        termsweeper::AppState::TitleScreen => self.handle_title_screen(key),
                        termsweeper::AppState::GameScreen => self.handle_game_screen(key),
                    };
                    if event_handled
                        || (key.kind == KeyEventKind::Press && key.code == KeyCode::F(5))
                    {
                        break;
                    }
                }
                if !LAZY_REDRAW {
                    break;
                }
            }
        }
        Ok(())
    }

    fn render_title_screen(&self, area: Rect, buf: &mut Buffer) {
        let top = Title::from(" Termsweeper - Title Screen ".green().bold());
        let bottom = Title::from(Line::from(vec![
            " New Game".into(),
            "<N> ".green().bold(),
            "Quit".into(),
            "<Q> ".green().bold(),
        ]));

        let block = Block::default()
            .title(top.alignment(Alignment::Center))
            .title(
                bottom
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);
        Paragraph::new(TITLE_SCREEN_CONTENT)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn handle_title_screen(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('n') => {
                    self.app_state = termsweeper::AppState::GameScreen;
                    self.game = Some(termsweeper::Termsweeper::default());
                }
                KeyCode::Char('q') => self.exit = true,
                _ => return false,
            }
            return true;
        }
        false
    }

    fn handle_game_screen(&mut self, key: KeyEvent) -> bool {
        let handled = match &mut self.game {
            Some(game_state) => game_state.handle_event(key),
            _ => false,
        };
        if handled {
            return handled;
        }
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => {
                    self.exit = true;
                }
                KeyCode::Char('e') => {
                    self.app_state = termsweeper::AppState::TitleScreen;
                }
                _ => return false,
            }
            return true;
        }
        false
    }
}

impl Widget for &TermsweeperApp {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.app_state {
            termsweeper::AppState::TitleScreen => self.render_title_screen(area, buf),
            termsweeper::AppState::GameScreen => {
                match &self.game {
                    Some(game) => game.render_game_screen(area, buf),
                    None => (),
                }
            }
        }
    }
}
