use std::{
    cmp,
    fs::File,
    io::{Read, Write}, path::{Path, PathBuf},
};

use crossterm::{
    cursor::SetCursorStyle,
    event::{KeyCode, KeyEvent},
    style::Stylize,
};
use log::{debug, error, info};

use crate::client::{
    console::{IncomingConsoleEvent, OutcomingConsoleEvent},
    Action, Container, DrawAction, Mode, Movement, Redraw,
};
use crate::utils::TruncAt;

use super::{Module, ModuleEvent, ModuleView};

pub mod vector;

pub trait EditorIO {
    fn open_file(&mut self, path: &Path) -> Result<(), std::io::Error>;
    fn save_file(&self) -> Result<(), std::io::Error>;
    fn write_file(&self, path: &Path) -> Result<(), std::io::Error>;
}

// pub trait EditorEvent {
//     fn on_load_file(&mut self, path: String);
//     fn on_action(&mut self, action: Vec<Action>);
// }

pub struct EditorContent<T> {
    data: T,
    is_crlf: bool,
}

pub struct Editor<T: EditorContentTrait> {
    pub file_path: Option<PathBuf>,
    pub content: T,
    pub render_row: u32,
    pub row: u32,
    pub render_col: u32,
    pub col: u32,
    pub should_redraw: Option<Redraw>,
    pub view: Container,
    pub line_numbered: bool,

    mode: Mode,
    // pub view_start: u32,
    // pub view_end: u32,
}

pub trait EditorContentTrait {
    fn new() -> Self;

    fn load_data(&mut self, raw_data: Vec<u8>);
    fn read_data(&self, buffer: &mut Vec<u8>);
    fn get_line(&self, i: u32) -> Option<String>;
    fn get_line_len(&self, i: u32) -> Option<u32>;
    fn write_char(&mut self, c: char, col: u32, row: u32);
    fn delete_char(&mut self, col: u32, row: u32) -> Option<char>;
}

impl<T: EditorContentTrait> Editor<T> {
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: T::new(),
            render_row: 0,
            row: 0,
            render_col: 0,
            col: 0,
            should_redraw: None,
            view: Container::default(),
            line_numbered: true,
            mode: Mode::Normal, // view_start: 0,
                                // view_end: 0,
        }
    }

    fn convert_key_to_actions(&mut self, key: KeyEvent) -> Vec<Action> {
        match self.mode {
            Mode::Normal => normal_mode_keybinding(key),
            Mode::Insert => insert_mode_keybinding(key),
            _ => todo!(),
        }
    }

    fn trigger_actions(&mut self, actions: &Vec<Action>) -> Option<Vec<OutcomingConsoleEvent>> {
        self.should_redraw = None;

        let return_vec = Vec::<OutcomingConsoleEvent>::new();

        for action in actions.iter() {
            match action {
                Action::Move(mov) => {
                    self.move_cursor(*mov);
                }
                Action::InsertChar(c) => {
                    self.write_char(*c);
                    self.move_cursor(Movement::Right);

                    if (*c) == '\n' {
                        self.should_redraw = Some(Redraw::All);
                    } else {
                        let modified_line = self.content.get_line(self.render_row);
                        if let Some(line) = modified_line {
                            self.should_redraw = Some(Redraw::Line(self.render_row, line));
                        }
                    }
                }
                Action::Backspace => {
                    if self.render_col == 0 && self.render_row == 0 {
                        self.should_redraw = Some(Redraw::Cursor);
                        continue;
                    }

                    self.move_cursor(Movement::Left);
                    let deleted_char = self.delete_char();
                    debug!("{:?}", deleted_char);
                    match deleted_char {
                        Some('\n') => {
                            self.should_redraw = Some(Redraw::All);
                        }
                        _ => {
                            let modified_line = self.content.get_line(self.render_row);
                            if let Some(line) = modified_line {
                                self.should_redraw = Some(Redraw::Line(self.render_row, line));
                            }
                        }
                    }
                }
                Action::Delete => {
                    let deleted_char = self.delete_char();
                    match deleted_char {
                        Some('\n') => {
                            self.should_redraw = Some(Redraw::All);
                        }
                        _ => {
                            let modified_line = self.content.get_line(self.render_row);
                            if let Some(line) = modified_line {
                                self.should_redraw = Some(Redraw::Line(self.render_row, line));
                            }
                        }
                    }
                }
                Action::ScrollBy(steps) => {
                    self.scroll_to(self.view.left as i32, self.view.top as i32 + steps);
                }
                // Action::ScrollTo(line_num) => {
                // self.scroll_to(self.view.left as i32, line_num as i32);
                // }
                Action::Resize(top, right, bottom, left) => {
                    let width = *right - *left;
                    let height = *bottom - *top;

                    // self.container.bottom = self.container.top + (*height) as u32 - 1;
                    // self.container.right = self.container.left + (*width) as u32 - 1;

                    self.view.bottom = self.view.top + height as u32 - 1;
                    self.view.right = self.view.left + width as u32 - 1;

                    self.should_redraw = Some(Redraw::All);
                }
                Action::SaveFile => {
                    self.save_file().unwrap();
                }
                Action::ChangeMode(mode) => {
                    self.should_redraw = Some(Redraw::Cursor);
                    self.mode = mode.clone();
                }
                _ => self.should_redraw = Some(Redraw::Cursor),
            };
        }

        if return_vec.is_empty() {
            None
        } else {
            Some(return_vec)
        }
    }

    fn get_offset(&self) -> u32 {
        if self.line_numbered {
            6
        } else {
            0
        }
    }

    fn move_cursor(&mut self, movement: Movement) {
        let line = self
            .content
            .get_line(self.render_row)
            .unwrap_or(String::from("\n"));
        let mut line_len = line.len() as u32;
        let mut wrap_left = false;
        self.should_redraw = Some(Redraw::Cursor);

        if self.render_row != self.row {
            self.row = self.render_row;
            self.col = self.render_col;
        }

        match movement {
            Movement::Up => {
                if self.render_row == self.view.top {
                    self.scroll_to(self.view.left as i32, self.view.top as i32 - 1);
                    self.should_redraw = Some(Redraw::All);
                }

                self.row = cmp::max(0, self.row as i32 - 1) as u32;
            }
            Movement::Down => {
                if self.render_row == self.view.bottom {
                    self.scroll_to(self.view.left as i32, self.view.top as i32 + 1);
                    self.should_redraw = Some(Redraw::All);
                }

                debug!("{:?}", self.view);
                self.row = cmp::min(self.view.bottom, self.row + 1) as u32;
            }
            Movement::Left => {
                if self.render_col == self.view.left {
                    self.scroll_to(self.view.left as i32 - 1, self.view.top as i32);
                    self.should_redraw = Some(Redraw::All);
                }

                if self.col == 0 && self.row != 0 {
                    self.row = cmp::max(0, self.row as i32 - 1) as u32;

                    wrap_left = true;

                    if self.render_row == self.view.top {
                        self.scroll_to(self.view.left as i32, self.view.top as i32 - 1);
                        self.should_redraw = Some(Redraw::All);
                    }
                } else {
                    self.col = cmp::max(0, cmp::min(self.render_col, self.col) as i32 - 1) as u32;
                }
            }
            Movement::Right => {
                if self.render_col == self.view.right - self.get_offset() {
                    self.scroll_to(self.view.left as i32 + 1, self.view.top as i32);
                    self.should_redraw = Some(Redraw::All);
                }

                self.col += 1;

                // TODO: skiping single-line file
                if self.col > line_len {
                    self.col = 0;
                    self.row += 1;

                    if self.render_row == self.view.bottom {
                        self.scroll_to(self.view.left as i32, self.view.top as i32 + 1);
                        self.should_redraw = Some(Redraw::All);
                    }
                }
            }
            Movement::LineEnd => {
                self.col = line_len;
            }
            Movement::LineStart => {
                self.col = 0;
            }
        }

        match self.content.get_line_len(self.row) {
            Some(n) => line_len = n,
            None => self.row -= 1,
        }

        if wrap_left {
            self.col = line_len;
        }

        self.render_col = cmp::min(line_len, self.col);

        self.goto_cursor();

        self.render_row = cmp::min(cmp::max(self.view.top, self.row), self.view.bottom);
        // info!("{}", self.render_row);
    }

    fn write_char(&mut self, c: char) {
        self.content.write_char(c, self.render_col, self.row);
    }

    fn delete_char(&mut self) -> Option<char> {
        self.content.delete_char(self.render_col, self.row)
    }

    fn scroll_to(&mut self, horizontal: i32, vertical: i32) {
        let horizontal_size = self.view.get_width();
        self.view.left = cmp::max(0, horizontal) as u32;
        self.view.right = self.view.left + horizontal_size;

        self.render_col = cmp::min(cmp::max(self.view.left, self.col), self.view.right);

        let vertical_size = self.view.get_height();
        self.view.top = cmp::max(0, vertical) as u32;
        self.view.bottom = self.view.top + vertical_size;

        self.render_row = cmp::min(cmp::max(self.view.top, self.row), self.view.bottom);

        match self.content.get_line_len(self.render_row) {
            Some(n) => self.render_col = cmp::min(self.col, n),
            None => (),
        }
    }

    fn goto_cursor(&mut self) {
        if self.render_col < self.view.left {
            self.scroll_to(self.render_col as i32, self.view.top as i32);
            self.should_redraw = Some(Redraw::All);
        } else if self.render_col + self.get_offset() > self.view.right {
            self.scroll_to(
                ((self.render_col + self.get_offset()) - self.view.get_width()) as i32,
                self.view.top as i32,
            );
            self.should_redraw = Some(Redraw::All);
        }
    }
}

impl<T: EditorContentTrait> EditorIO for Editor<T> {
    fn open_file(&mut self, path: &Path) -> Result<(), std::io::Error> {
        self.file_path = Some(path.to_path_buf());
        let mut file = File::open(path)?;
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf)?;
        self.content.load_data(buf);
        Ok(())
    }

    fn save_file(&self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.file_path {
            let mut file = File::create(path)?;
            let mut buf: Vec<u8> = Vec::new();
            self.content.read_data(&mut buf);
            file.write_all(&buf)?;
            return Ok(());
        }

        error!("there isn't any file opened!");
        Ok(())
    }

    fn write_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut file = File::create(path)?;
        let mut buf: Vec<u8> = Vec::new();
        self.content.read_data(&mut buf);
        file.write_all(&buf)?;
        Ok(())
    }
}

impl<T: EditorContentTrait> ModuleEvent for Editor<T> {
    fn on_event(&mut self, event: IncomingConsoleEvent) -> Option<Vec<OutcomingConsoleEvent>> {
        match event {
            IncomingConsoleEvent::Key(key_event) => {
                let actions = self.convert_key_to_actions(key_event);
                self.trigger_actions(&actions)
            }
            IncomingConsoleEvent::File(file_path) => {
                let _ = self.open_file(&file_path);
                None
            }
            _ => None,
        }
    }

    fn on_draw(&self) -> Option<Vec<DrawAction>> {
        if self.file_path.is_none() {
            return None;
        }

        let mut drawing_actions = vec![];

        match &self.should_redraw {
            Some(Redraw::All) => {
                debug!("Redraw: all");
                let mut line_num = self.view.top;

                while let Some(line) = self.content.get_line(line_num) {
                    let mut actual_string = String::new();
                    if line_num > self.view.bottom {
                        break;
                    }

                    if self.line_numbered {
                        actual_string.push_str(format!("{:>4}  ", line_num + 1).as_str());
                    }
                    actual_string.push_str(
                        line.truncate_at((self.view.left) as usize)
                            .unwrap_or(String::new())
                            .as_str(),
                    );
                    drawing_actions.push(DrawAction::AskRedraw(Redraw::Line(
                        line_num - self.view.top,
                        actual_string,
                    )));
                    line_num += 1;
                }

                // Redraw rest of the screen
                for i in line_num..self.view.bottom + 1 {
                    let mut actual_string = String::new();
                    if self.line_numbered {
                        actual_string
                            .push_str(format!("{:>4}  ", i + 1).dark_grey().to_string().as_str());
                    }
                    drawing_actions.push(DrawAction::AskRedraw(Redraw::Line(
                        i - self.view.top,
                        actual_string,
                    )));
                }
            }
            Some(Redraw::Line(y, str)) => {
                debug!("Redraw: line");
                let mut actual_string = String::new();
                let line_num = y;

                if self.line_numbered {
                    actual_string.push_str(format!("{:>4}  ", y + 1).as_str());
                }
                actual_string.push_str(
                    str.truncate_at((self.view.left) as usize)
                        .unwrap_or(String::new())
                        .as_str(),
                );

                drawing_actions.push(DrawAction::AskRedraw(Redraw::Line(
                    line_num - self.view.top,
                    actual_string,
                )));
            }
            Some(Redraw::Cursor) => {
                // debug!("Redraw: cursor");
            }
            Some(Redraw::Range(_, _)) => todo!(),
            None => {
                debug!("Redraw: none");
                return None;
            }
        }

        if self.line_numbered {
            drawing_actions.push(DrawAction::CursorTo(
                self.render_col + 6,
                self.render_row - self.view.top,
                get_cursor_style(self.mode),
            ));
        } else {
            drawing_actions.push(DrawAction::CursorTo(
                self.render_col,
                self.render_row - self.view.top,
                get_cursor_style(self.mode),
            ));
        }

        Some(drawing_actions)
    }
}

impl<T: EditorContentTrait> ModuleView for Editor<T> {
    fn get_container(&self) -> &Container {
        &self.view
    }
}

impl<T: EditorContentTrait> Module for Editor<T> {}

fn normal_mode_keybinding(key: KeyEvent) -> Vec<Action> {
    match key.code {
        KeyCode::Char('k') => vec![Action::Move(Movement::Up)],
        KeyCode::Char('j') => vec![Action::Move(Movement::Down)],
        KeyCode::Char('h') => vec![Action::Move(Movement::Left)],
        KeyCode::Char('l') => vec![Action::Move(Movement::Right)],
        KeyCode::Char('q') => vec![Action::Quit],
        KeyCode::Char('i') => vec![Action::ChangeMode(Mode::Insert)],
        KeyCode::Char('I') => vec![
            Action::Move(Movement::LineStart),
            Action::ChangeMode(Mode::Insert),
        ],
        KeyCode::Char('a') => vec![
            Action::Move(Movement::Right),
            Action::ChangeMode(Mode::Insert),
        ],
        KeyCode::Char('A') => vec![
            Action::Move(Movement::LineEnd),
            Action::ChangeMode(Mode::Insert),
        ],
        KeyCode::Char('s') => vec![Action::SaveFile],
        KeyCode::Char(':') => vec![Action::ChangeMode(Mode::Command)],
        KeyCode::PageDown => vec![Action::ScrollBy(1)],
        KeyCode::PageUp => vec![Action::ScrollBy(-1)],
        KeyCode::Backspace => vec![Action::Move(Movement::Left)],
        KeyCode::Enter => vec![Action::Move(Movement::Down)],
        KeyCode::Esc => vec![Action::Quit],
        KeyCode::Up => vec![Action::Move(Movement::Up)],
        KeyCode::Down => vec![Action::Move(Movement::Down)],
        KeyCode::Left => vec![Action::Move(Movement::Left)],
        KeyCode::Right => vec![Action::Move(Movement::Right)],
        _ => vec![Action::None],
    }
}

fn insert_mode_keybinding(key: KeyEvent) -> Vec<Action> {
    match key.code {
        KeyCode::Char(c) => vec![Action::InsertChar(c)],
        KeyCode::Backspace => vec![Action::Backspace],
        KeyCode::Delete => vec![Action::Delete],
        KeyCode::Up => vec![Action::Move(Movement::Up)],
        KeyCode::Down => vec![Action::Move(Movement::Down)],
        KeyCode::Left => vec![Action::Move(Movement::Left)],
        KeyCode::Right => vec![Action::Move(Movement::Right)],
        KeyCode::Esc => vec![Action::ChangeMode(Mode::Normal)],
        KeyCode::Enter => vec![Action::InsertChar('\n')],
        _ => vec![Action::None],
    }
}

fn get_cursor_style(mode: Mode) -> SetCursorStyle {
    return match mode {
        Mode::Normal => SetCursorStyle::SteadyBlock,
        Mode::Insert => SetCursorStyle::BlinkingBar,
        Mode::Visual => SetCursorStyle::SteadyUnderScore,
        Mode::Command => SetCursorStyle::BlinkingBar,
    };
}
