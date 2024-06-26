use std::{
    cmp,
    fs::File,
    io::{Read, Write},
};

use log::{error, info};

pub mod vector;

pub struct Container {
    pub top: u32,
    pub left: u32,
    pub bottom: u32,
    pub right: u32,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        }
    }
}

impl Container {
    pub fn get_width(&self) -> u32 {
        self.right - self.left
    }

    pub fn get_height(&self) -> u32 {
        self.bottom - self.top
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    Up,
    Down,
    Left,
    Right,
    LineEnd,
    LineStart,
}

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

#[allow(unused)]
#[derive(PartialEq, Eq)]
pub enum Action {
    Move(Movement),
    ChangeMode(Mode),
    InsertChar(char),
    Backspace,
    Delete,
    Quit,
    None,

    ScrollBy(i32),
    // ScrollTo(u32),
    Resize(u16, u16),

    OpenFile(String),
    WriteFile(String),
    SaveFile,

    AskRedraw(Redraw),
}

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Redraw {
    All,
    Line(u32),
    Range(u32, u32),
}

pub trait EditorIO {
    fn open_file(&mut self, path: &str) -> Result<(), std::io::Error>;
    fn save_file(&self) -> Result<(), std::io::Error>;
    fn write_file(&self, path: &str) -> Result<(), std::io::Error>;
}

pub trait EditorEvent {
    fn on_load_file(&mut self, path: String);
    fn on_action(&mut self, action: Vec<Action>);
}

pub struct EditorContent<T> {
    data: T,
    is_crlf: bool,
}

pub struct Editor<T: EditorContentTrait> {
    pub file_path: Option<String>,
    pub content: T,
    pub render_row: u32,
    pub row: u32,
    pub render_col: u32,
    pub col: u32,
    pub mode: Mode,
    pub should_redraw: Option<Redraw>,
    pub view: Container,
    // pub view_start: u32,
    // pub view_end: u32,
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
            mode: Mode::Normal,
            should_redraw: None,
            view: Container::default(),
            // view_start: 0,
            // view_end: 0,
        }
    }

    fn move_cursor(&mut self, movement: Movement) {
        let line = self
            .content
            .get_line(self.render_row)
            .unwrap_or(String::from("\n"));
        let mut line_len = line.len() as u32;
        let mut wrap_left = false;

        if self.render_row != self.row {
            self.row = self.render_row;
            self.col = self.render_col;
        }

        // if self.render_col != self.col {
        //     self.row = self.render_row;
        //     self.col = self.render_col;
        // }

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

                self.row += 1;
            }
            Movement::Left => {
                if self.render_col == self.view.left {
                    self.scroll_to(self.view.left as i32 - 1, self.view.top as i32);
                    self.should_redraw = Some(Redraw::All);
                }

                if self.col == 0 && self.row != 0 {
                    self.row = cmp::max(0, self.row as i32 - 1) as u32;

                    wrap_left = true;
                } else {
                    self.col = cmp::max(0, cmp::min(self.render_col, self.col) as i32 - 1) as u32;
                }
            }
            Movement::Right => {
                if self.render_col == self.view.right {
                    self.scroll_to(self.view.left as i32 + 1, self.view.top as i32);
                    self.should_redraw = Some(Redraw::All);
                }

                self.col += 1;

                // TODO: skiping single-line file
                if self.col > line_len {
                    self.col = 0;
                    self.row += 1;
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
        } else if self.render_col > self.view.right {
            self.scroll_to(
                (self.render_col - self.view.get_width()) as i32,
                self.view.top as i32,
            );
        }

        self.should_redraw = Some(Redraw::All);
    }
}

impl<T: EditorContentTrait> EditorIO for Editor<T> {
    fn open_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        self.file_path = Some(path.to_string());
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

    fn write_file(&self, path: &str) -> Result<(), std::io::Error> {
        let mut file = File::create(path)?;
        let mut buf: Vec<u8> = Vec::new();
        self.content.read_data(&mut buf);
        file.write_all(&buf)?;
        Ok(())
    }
}

impl<T: EditorContentTrait> EditorEvent for Editor<T> {
    fn on_load_file(&mut self, path: String) {
        info!("loading file '{}'", path);

        self.open_file(path.as_str()).unwrap();

        self.file_path = Some(path);
    }

    fn on_action(&mut self, actions: Vec<Action>) {
        self.should_redraw = None;

        actions.iter().for_each(|action| {
            match *action {
                Action::Move(mov) => {
                    self.move_cursor(mov);
                }
                Action::ChangeMode(mode) => {
                    self.mode = mode;
                }
                Action::InsertChar(c) => {
                    if c == '\n' {
                        self.should_redraw = Some(Redraw::All);
                    } else {
                        self.should_redraw = Some(Redraw::Line(self.row));
                    }

                    self.write_char(c);
                    self.move_cursor(Movement::Right);
                }
                Action::Backspace => {
                    if self.render_col == 0 {
                        self.should_redraw = Some(Redraw::All);
                    } else {
                        self.should_redraw = Some(Redraw::Line(self.row));
                    }

                    self.move_cursor(Movement::Left);
                    self.delete_char();
                }
                Action::Delete => {
                    let deleted_char = self.delete_char();
                    match deleted_char {
                        Some('\n') => {
                            self.should_redraw = Some(Redraw::All);
                        }
                        _ => {
                            self.should_redraw = Some(Redraw::Line(self.row));
                        }
                    }
                }
                Action::ScrollBy(steps) => {
                    self.scroll_to(self.view.left as i32, self.view.top as i32 + steps);
                }
                // Action::ScrollTo(line_num) => {
                // self.scroll_to(self.view.left as i32, line_num as i32);
                // }
                Action::Resize(width, height) => {
                    self.view.bottom = self.view.top + height as u32 - 1;
                    self.view.right = self.view.left + width as u32 - 1;

                    self.should_redraw = Some(Redraw::All);
                }
                Action::SaveFile => {
                    self.save_file().unwrap();
                }
                Action::AskRedraw(redraw) => {
                    self.should_redraw = Some(redraw);
                }
                Action::Quit => {
                    panic!("program exited!");
                }
                Action::None => {}
                _ => todo!(),
            };
        });
    }
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
