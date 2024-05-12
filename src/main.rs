use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use std::io;

mod tui;

static LAZY_REDRAW: bool = true;
static TITLE_SCREEN_CONTENT: &str =
    include_str!("../assets/title.in");
fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = TermsweeperApp::new().run(&mut terminal);
    tui::restore()?;
    app_result
}

enum TermsweeperAppState {
    TitleScreen,
    GameScreen,
}

struct TermsweeperApp {
    state: TermsweeperAppState,
    exit: bool,
}

impl TermsweeperApp {
    fn new() -> TermsweeperApp {
        TermsweeperApp {
            state: TermsweeperAppState::TitleScreen,
            exit: false,
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
                    let event_handled = match self.state {
                        TermsweeperAppState::TitleScreen => self.handle_title_screen(key),
                        TermsweeperAppState::GameScreen => self.handle_game_screen(key),
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
                KeyCode::Char('n') => self.state = TermsweeperAppState::GameScreen,
                KeyCode::Char('q') => self.exit = true,
                _ => return false,
            }
            return true;
        }
        false
    }

    fn render_game_screen(&self, area: Rect, buf: &mut Buffer) {
        let top = Title::from(" Termsweeper - Game ".green().bold());
        let bottom = Title::from(Line::from(vec![
            " Exit current game".into(),
            "<E> ".green().bold(),
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
        Paragraph::new("Not implemented yet")
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn handle_game_screen(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => {
                    // TODO - trigger are you sure popup
                    self.exit = true;
                }
                KeyCode::Char('e') => {
                    // TODO - trigger are you sure popup
                    self.exit = true;
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
        match self.state {
            TermsweeperAppState::TitleScreen => self.render_title_screen(area, buf),
            TermsweeperAppState::GameScreen => self.render_game_screen(area, buf),
        }
    }
}
