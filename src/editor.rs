use std::{
    cmp,
    fs::File,
    io::{Read, Write},
    process::exit,
};

use log::{error, info};

use self::{actions::Action, redraw::Redraw};

pub mod actions;
pub mod redraw;

type VectorEditor<T> = Editor<EditorContent<Vec<T>>>;
pub type CharVectorEditor = VectorEditor<char>;

#[derive(Clone, Copy)]
pub enum Movement {
    Up,
    Down,
    Left,
    Right,
    LineEnd,
    LineStart,
}

#[derive(Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
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
    pub view_start: u32,
    pub view_end: u32,
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

impl<T: EditorContentTrait> Editor<T> {
    pub fn new(content: T) -> Self {
        Self {
            file_path: None,
            content,
            render_row: 0,
            row: 0,
            render_col: 0,
            col: 0,
            mode: Mode::Normal,
            should_redraw: Some(Redraw::All),
            view_start: 0,
            view_end: 0,
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

        match movement {
            Movement::Up => {
                if self.render_row == self.view_start {
                    self.scroll_to(self.view_start as i32 - 1);
                    self.should_redraw = Some(Redraw::All);
                }

                self.row = cmp::max(0, self.row as i32 - 1) as u32;
            }
            Movement::Down => {
                if self.render_row == self.view_end {
                    self.scroll_to(self.view_start as i32 + 1);
                    self.should_redraw = Some(Redraw::All);
                }

                self.row += 1;
            }
            Movement::Left => {
                if self.col == 0 && self.row != 0 {
                    self.row = cmp::max(0, self.row as i32 - 1) as u32;

                    wrap_left = true;
                } else {
                    self.col = cmp::max(0, cmp::min(self.render_col, self.col) as i32 - 1) as u32;
                }
            }
            Movement::Right => {
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
        self.render_row = cmp::min(cmp::max(self.view_start, self.row), self.view_end);
    }

    fn write_char(&mut self, c: char) {
        self.content.write_char(c, self.render_col, self.row);
    }

    fn delete_char(&mut self) -> Option<char> {
        self.content.delete_char(self.render_col, self.row)
    }

    fn scroll_to(&mut self, line_num: i32) {
        let view_size = self.view_end - self.view_start;

        self.view_start = cmp::max(0, line_num) as u32;
        self.view_end = self.view_start + view_size;

        self.render_row = cmp::min(cmp::max(self.view_start, self.row), self.view_end);

        match self.content.get_line_len(self.render_row) {
            Some(n) => self.render_col = cmp::min(self.col, n),
            None => (),
        }
    }
}

impl EditorIO for CharVectorEditor {
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

impl EditorEvent for CharVectorEditor {
    fn on_load_file(&mut self, path: String) {
        info!("loading file '{}'", path);

        self.open_file(path.as_str())
            .unwrap_or_else(|_err| error!("error while loading"));

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
                    self.scroll_to(self.view_start as i32 + steps);
                }
                Action::ScrollTo(line_num) => {
                    self.scroll_to(line_num as i32);
                }
                Action::Resize(line_count) => {
                    self.view_end = self.view_start + line_count as u32 - 1;

                    self.should_redraw = Some(Redraw::All);
                }
                Action::SaveFile => {
                    self.save_file().unwrap();
                }
                Action::AskRedraw(redraw) => {
                    self.should_redraw = Some(redraw);
                }
                Action::Quit => {
                    exit(0);
                }
                Action::None => {}
                _ => todo!(),
            };
        });
    }
}

fn is_crlf(c: char) -> bool {
    return c == '\n' || c == '\r';
}

pub struct EditorContent<T> {
    data: T,
    is_crlf: bool,
}

pub trait EditorContentTrait {
    fn load_data(&mut self, raw_data: Vec<u8>);
    fn read_data(&self, buffer: &mut Vec<u8>);
    fn get_line(&self, i: u32) -> Option<String>;
    fn get_line_len(&self, i: u32) -> Option<u32>;
    fn write_char(&mut self, c: char, col: u32, row: u32);
    fn delete_char(&mut self, col: u32, row: u32) -> Option<char>;
}

impl EditorContent<Vec<char>> {
    pub fn new() -> EditorContent<Vec<char>> {
        Self {
            data: Vec::<char>::new(),
            is_crlf: true,
        }
    }

    fn get_pos(&self, col: u32, row: u32) -> Option<usize> {
        let mut line_count = 0;
        let mut col_count = 0;
        let mut i = 0;

        while let Some(c_ref) = self.data.get(i) {
            if line_count == row && col_count == col {
                break;
            }

            i += 1;
            if *c_ref == '\n' {
                line_count += 1;
                col_count = 0;
            } else {
                col_count += 1;
            }
        }

        if i <= self.data.len() {
            Some(i as usize)
        } else {
            None
        }
    }
}

impl EditorContentTrait for EditorContent<Vec<char>> {
    fn load_data(&mut self, raw_data: Vec<u8>) {
        self.data = raw_data
            .iter()
            .map(|c| *c as char)
            .filter(|c| *c != '\r')
            .collect();
    }

    fn get_line(&self, i: u32) -> Option<String> {
        self.data
            .split(|c| is_crlf(*c))
            .map(|l| l.iter().collect())
            .nth(i as usize)
    }

    fn get_line_len(&self, i: u32) -> Option<u32> {
        Some(self.get_line(i)?.len() as u32)
    }

    fn write_char(&mut self, c: char, col: u32, row: u32) {
        if let Some(i) = self.get_pos(col, row) {
            self.data.insert(i, c);
        }
    }

    fn delete_char(&mut self, col: u32, row: u32) -> Option<char> {
        if let Some(i) = self.get_pos(col, row) {
            if i < self.data.len() {
                return Some(self.data.remove(i));
            }
        }

        None
    }

    fn read_data(&self, buffer: &mut Vec<u8>) {
        let data_bytes: Vec<u8> = self
            .data
            .iter()
            .map(|c| c.to_string().into_bytes())
            .map(|c| {
                if c[0] == 0x0A && self.is_crlf {
                    vec![0x0D, 0x0A]
                } else {
                    c
                }
            })
            .flatten()
            .collect();
        buffer.write(&data_bytes).unwrap();
    }
}
