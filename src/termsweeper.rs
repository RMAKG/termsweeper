use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

pub enum AppState {
    TitleScreen,
    GameScreen,
}

enum GameState {
    Playing,
    GameOver,
    Won,
}

#[derive(Clone)]
pub struct Field {
    revealed: bool,
    marked: bool,
    is_mine: bool,
    adjacent_mines: u8,
}

impl Field {
    fn new() -> Field {
        Field {
            revealed: false,
            marked: false,
            is_mine: false,
            adjacent_mines: 0,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, borders: Borders, cursor: bool) {
        const SYMBOL_DEFAULT: &str = "?"; // ⣿ ⠶
        const SYMBOL_MARKED: &str = "X";
        const SYMBOL_MINE: &str = "*";
        let border_set = symbols::border::Set {
            bottom_right: symbols::line::CROSS,
            ..symbols::border::PLAIN
        };
        let border = Block::default()
            .border_set(border_set)
            .borders(borders)
            .border_style(Style::new().dark_gray());
        let (text, mut style) = if self.revealed {
            if self.is_mine {
                (SYMBOL_MINE, Style::default().fg(Color::Red))
            } else {
                match self.adjacent_mines {
                    0 => (" ", Style::default()),
                    1 => ("1", Style::default().fg(Color::LightBlue)),
                    2 => ("2", Style::default().fg(Color::LightGreen)),
                    3 => ("3", Style::default().fg(Color::LightYellow)),
                    4 => ("4", Style::default().fg(Color::LightRed)),
                    5 => ("5", Style::default().fg(Color::Red)),
                    6 => ("6", Style::default().fg(Color::LightMagenta)),
                    7 => ("7", Style::default().fg(Color::Magenta)),
                    8 => ("8", Style::default().fg(Color::Magenta)),
                    _ => (SYMBOL_DEFAULT, Style::default()),
                }
            }
        } else if self.marked {
            (SYMBOL_MARKED, Style::default().fg(Color::Red))
        } else {
            (SYMBOL_DEFAULT, Style::default().fg(Color::DarkGray))
        };
        if cursor {
            style = style.bg(Color::Green);
        }
        if self.revealed &&  self.marked {
            if self.is_mine {
                style = style.bg(Color::LightGreen)
            }
            else {
                style = style.bg(Color::LightBlue)
            }
        }
        Paragraph::new(Span::styled(text, style))
            .block(border)
            .render(area, buf);
    }
}

#[derive(Clone)]
pub struct Row {
    fields: Vec<Field>,
}

impl Row {
    fn new(entries: u8) -> Row {
        Row {
            fields: vec![Field::new(); entries.into()],
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, borders: Borders, cursor_location: Option<u8>) {
        const FIELD_SIZE: u16 = 2;
        let fields = self.fields.len();
        let mut constraints = vec![Constraint::Min(0)];
        constraints.append(&mut Constraint::from_maxes(vec![FIELD_SIZE; fields - 1]));
        constraints.push(Constraint::Max(FIELD_SIZE - 1));
        constraints.push(Constraint::Min(0));
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);
        let mut i = 1;
        for field in &self.fields {
            let field_border = if i == self.fields.len() {
                Borders::NONE
            } else {
                Borders::RIGHT | borders
            };
            let cursor = match cursor_location {
                Some(field_location) if i - 1 == field_location.into() => true,
                _ => false,
            };
            field.render(layout[i], buf, field_border | borders, cursor);
            i += 1;
        }
    }
}

pub struct Termsweeper {
    columns: u8,
    rows: u8,
    number_of_mines: u16,
    fields_left_to_reveal: u16,
    board: Vec<Row>,
    cursor: (u8, u8),
    initialized: bool,
    game_state: GameState,
}

impl Termsweeper {
    pub fn default() -> Termsweeper {
        Self::new(45, 18, 75)
    }

    pub fn new(columns: u8, rows: u8, number_of_mines: u16) -> Termsweeper {
        Termsweeper {
            columns,
            rows,
            number_of_mines,
            fields_left_to_reveal: 0,
            board: vec![Row::new(columns); rows.into()],
            cursor: (0, 0),
            initialized: false,
            game_state: GameState::Playing,
        }
    }

    fn initialize(&mut self) {
        if !self.initialized {
            let valid_adjacent = self.get_valid_adjacent_fields(self.cursor);
            let max_mines =
                self.columns as u16 * self.rows as u16 - 1 - valid_adjacent.len() as u16;
            if self.number_of_mines > max_mines {
                self.number_of_mines = max_mines;
            }
            let mut mine_locations: Vec<(u8, u8)> = vec![];
            let mut rng = rand::thread_rng();
            let mut i: u16 = 0;
            self.fields_left_to_reveal =
                self.columns as u16 * self.rows as u16 - self.number_of_mines;
            while i < self.number_of_mines {
                let row = rng.gen_range(0..self.rows);
                let column = rng.gen_range(0..self.columns);
                if (row, column) != self.cursor
                    && !valid_adjacent.contains(&(row, column))
                    && !mine_locations.contains(&(row, column))
                {
                    mine_locations.push((row, column));
                    i += 1;
                }
            }
            for mine_location in mine_locations {
                self.get_field_mut(mine_location).is_mine = true;
            }
            for row_index in 0..self.rows {
                for column_index in 0..self.columns {
                    let current_field_location = (row_index, column_index);
                    for location in self.get_valid_adjacent_fields((row_index, column_index)) {
                        if self.get_field(location).is_mine {
                            self.get_field_mut(current_field_location).adjacent_mines += 1;
                        }
                    }
                }
            }
            self.initialized = true
        }
    }

    pub fn handle_event(&mut self, key: KeyEvent) -> bool {
        match self.game_state {
            GameState::Playing => match key.code {
                KeyCode::Char('h') | KeyCode::Left => self.move_cursor_left(),
                KeyCode::Char('j') | KeyCode::Down => self.move_cursor_down(),
                KeyCode::Char('k') | KeyCode::Up => self.move_cursor_up(),
                KeyCode::Char('l') | KeyCode::Right => self.move_cursor_right(),
                KeyCode::Char('m') | KeyCode::Enter => self.toggle_mark(),
                KeyCode::Char(' ') => self.reveal(),
                _ => false,
            },
            _ => false,
        }
    }

    fn get_field(&self, location: (u8, u8)) -> &Field {
        &self.board[location.0 as usize].fields[location.1 as usize]
    }

    fn get_field_mut(&mut self, location: (u8, u8)) -> &mut Field {
        &mut self.board[location.0 as usize].fields[location.1 as usize]
    }

    fn get_valid_adjacent_fields(&self, location: (u8, u8)) -> Vec<(u8, u8)> {
        self.get_ordered_adjacent_fields(location)
            .to_vec()
            .into_iter()
            .flatten()
            .collect()
    }

    fn get_ordered_adjacent_fields(&self, location: (u8, u8)) -> [Option<(u8, u8)>; 8] {
        let mut return_values: [Option<(u8, u8)>; 8] = [None; 8];
        let column_index = location.1;
        let row_index = location.0;
        let left_field_index = column_index.checked_sub(1);
        let right_field_index = column_index + 1;
        let top_row_index = row_index.checked_sub(1);
        let bottowm_row_index = row_index + 1;
        match left_field_index {
            Some(left_column_value) => {
                return_values[0] = Some((row_index, left_column_value));
                return_values[1] = match top_row_index {
                    Some(top_row_value) => Some((top_row_value, left_column_value)),
                    None => None,
                };

                return_values[2] = if bottowm_row_index < self.rows {
                    Some((bottowm_row_index, left_column_value))
                } else {
                    None
                }
            }
            None => (),
        }
        return_values[3] = match top_row_index {
            Some(top_row_value) => Some((top_row_value, column_index)),
            None => None,
        };
        if bottowm_row_index < self.rows {
            return_values[4] = Some((bottowm_row_index, column_index));
        }
        if right_field_index < self.columns {
            return_values[5] = Some((row_index, right_field_index));
            return_values[6] = match top_row_index {
                Some(top_row_value) => Some((top_row_value, right_field_index)),
                None => None,
            };
            if bottowm_row_index < self.rows {
                return_values[7] = Some((bottowm_row_index, right_field_index));
            }
        }
        return_values
    }

    fn move_cursor_left(&mut self) -> bool {
        if self.cursor.1 != 0 {
            self.cursor.1 -= 1;
            true
        } else {
            false
        }
    }

    fn move_cursor_down(&mut self) -> bool {
        if self.cursor.0 != self.rows - 1 {
            self.cursor.0 += 1;
            true
        } else {
            false
        }
    }

    fn move_cursor_up(&mut self) -> bool {
        if self.cursor.0 != 0 {
            self.cursor.0 -= 1;
            true
        } else {
            false
        }
    }

    fn move_cursor_right(&mut self) -> bool {
        if self.cursor.1 != self.columns - 1 {
            self.cursor.1 += 1;
            true
        } else {
            false
        }
    }

    fn toggle_mark(&mut self) -> bool {
        if !self.get_field(self.cursor).revealed {
            self.get_field_mut(self.cursor).marked = !self.get_field(self.cursor).marked;
            true
        } else {
            false
        }
    }

    fn reveal(&mut self) -> bool {
        if !self.initialized {
            self.initialize();
        }
        if !self.get_field(self.cursor).marked && !self.get_field(self.cursor).revealed {
            self.get_field_mut(self.cursor).revealed = true;
            if self.get_field(self.cursor).is_mine {
                self.game_state = GameState::GameOver;
                self.reveal_all();
            } else {
                self.fields_left_to_reveal -= 1;
                if self.get_field(self.cursor).adjacent_mines == 0 {
                    let mut adjacent_fields = self.get_valid_adjacent_fields(self.cursor).to_vec();
                    while let Some(location) = adjacent_fields.pop() {
                        if !self.get_field(location).revealed {
                            self.get_field_mut(location).revealed = true;
                            self.fields_left_to_reveal -= 1;
                            if self.get_field(location).adjacent_mines == 0 {
                                adjacent_fields
                                    .append(&mut self.get_valid_adjacent_fields(location).to_vec());
                            }
                        }
                    }
                }
                if self.fields_left_to_reveal == 0 {
                    self.game_state = GameState::Won;
                    self.reveal_all();
                }
            }
            true
        } else {
            false
        }
    }

    fn reveal_all(&mut self) {
        for row in &mut self.board {
            for field in &mut row.fields {
                field.revealed = true;
            }
        }
    }

    pub fn render_game_screen(&self, area: Rect, buf: &mut Buffer) {
        let top = match self.game_state {
            GameState::Won => Title::from(" Termsweeper - VICTORY ".yellow().bold()),
            GameState::GameOver => Title::from(" Termsweeper - GAME OVER ".red().bold()),
            _ => Title::from(" Termsweeper - Game ".green().bold()),
        };
        let mut navigation = match self.game_state {
            GameState::Playing => vec![
                " Left".into(),
                "<H/←> ".green().bold(),
                "Down".into(),
                "<J/↓> ".green().bold(),
                "Up".into(),
                "<K/↑> ".green().bold(),
                "Right".into(),
                "<L/→> ".green().bold(),
                "Mark".into(),
                "<M/Enter> ".green().bold(),
                "Reveal".into(),
                "<Space> ".green().bold(),
            ],
            _ => vec![" ".into()],
        };
        navigation.append(&mut vec![
            "Exit to menu".into(),
            "<E> ".green().bold(),
            "Quit".into(),
            "<Q> ".green().bold(),
        ]);
        let bottom = Title::from(Line::from(navigation));

        let outer_border = Block::default()
            .title(top.alignment(Alignment::Center))
            .title(
                bottom
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let inner_area = outer_border.inner(area);
        outer_border.render(area, buf);
        self.render_playing_board(inner_area, buf);
    }

    fn render_playing_board(&self, area: Rect, buf: &mut Buffer) {
        const ROW_SIZE: u16 = 2;
        let rows = self.board.len();
        let mut constraints = vec![Constraint::Min(0)];
        constraints.append(&mut Constraint::from_maxes(vec![ROW_SIZE; rows - 1]));
        constraints.push(Constraint::Max(ROW_SIZE - 1));
        constraints.push(Constraint::Min(0));
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(constraints)
            .split(area);
        let mut i = 1;
        for row in &self.board {
            let row_border = if i == self.board.len() {
                Borders::NONE
            } else {
                Borders::BOTTOM
            };
            let cursor_location = if i - 1 == self.cursor.0.into() {
                Some(self.cursor.1)
            } else {
                None
            };
            row.render(layout[i], buf, row_border, cursor_location);
            i += 1;
        }
    }
}
