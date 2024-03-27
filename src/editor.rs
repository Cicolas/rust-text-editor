use std::{cmp, fs::File, io::Read, ops::Add, process::exit};

use log::{error, info};

pub type VectorEditor = Editor<Vec<char>>;

enum Movement {
    Up,
    Down,
    Left,
    Right,
    LineEnd,
    LineStart,
}

pub enum Mode {
    Normal,
    Insert,
    Visual,
}

pub struct Editor<T> {
    pub file_path: Option<String>,
    pub content: EditorContent<T>,
    pub row: u32,
    pub render_col: u32,
    pub col: u32,
    pub mode: Mode,
}

pub trait EditorIO {
    fn open_file(&mut self, path: &str) -> Result<(), std::io::Error>;
    fn write_file(path: &str, data: Vec<u8>);
}

pub trait EditorEvent {
    fn on_load_file(&mut self, path: String);
    fn on_write(&mut self, keycode: u32);
}

impl VectorEditor {
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: EditorContent::<Vec<char>>::new(),
            row: 0,
            render_col: 0,
            col: 0,
            mode: Mode::Normal,
        }
    }

    fn move_cursor(&mut self, movement: Movement) {
        let line = self
            .content
            .get_line(self.row)
            .unwrap_or(String::from("\n"));
        let mut line_len = line.len() as u32;
        let mut wrap_left = false;

        match movement {
            Movement::Up => {
                self.row = cmp::max(0, self.row as i32 - 1) as u32;
            }
            Movement::Down => {
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
    }
}

impl EditorIO for VectorEditor {
    fn open_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf)?;
        self.content.load_data(buf);
        Ok(())
    }

    fn write_file(path: &str, data: Vec<u8>) {
        todo!()
    }
}

impl EditorEvent for VectorEditor {
    fn on_load_file(&mut self, path: String) {
        info!("loading file '{}'", path);

        self.open_file(path.as_str())
            .unwrap_or_else(|_err| error!("error while loading"));

        self.file_path = Some(path);
    }

    fn on_write(&mut self, keycode: u32) {
        match char::from_u32(keycode) {
            // UP
            _ if keycode == 0x26 => {
                self.move_cursor(Movement::Up);
            }
            // DOWN
            _ if keycode == 0x28 => {
                self.move_cursor(Movement::Down);
            }
            // LEFT
            _ if keycode == 0x25 => {
                self.move_cursor(Movement::Left);
            }
            // RIGHT
            _ if keycode == 0x27 => {
                self.move_cursor(Movement::Right);
            }
            _ => (),
        }

        match self.mode {
            Mode::Normal => match char::from_u32(keycode) {
                Some('h') => {
                    self.move_cursor(Movement::Left);
                }
                Some('j') => {
                    self.move_cursor(Movement::Down);
                }
                Some('k') => {
                    self.move_cursor(Movement::Up);
                }
                Some('l') => {
                    self.move_cursor(Movement::Right);
                }
                Some('i') => self.mode = Mode::Insert,
                Some('a') => {
                    self.move_cursor(Movement::Right);
                    self.mode = Mode::Insert;
                }
                Some('I') => {
                    self.move_cursor(Movement::LineStart);
                    self.mode = Mode::Insert;
                }
                Some('A') => {
                    self.move_cursor(Movement::LineEnd);
                    self.mode = Mode::Insert;
                }
                Some('q') => {
                    exit(0);
                }
                _ => {
                    if keycode == 0x1B {
                        exit(0);
                    }
                }
            },
            Mode::Insert => {
                // ESC
                if keycode == 0x1B {
                    self.mode = Mode::Normal;
                    return;
                }
                if keycode == 0x08 {
                    if !(self.col == 0 && self.row == 0) {
                        self.move_cursor(Movement::Left);                    
                        self.content.delete_char(self.render_col, self.row);
                    }
                    return;
                }

                match char::from_u32(keycode) {
                    Some(c) => {
                        self.content.write_char(c, self.render_col, self.row);
                        self.move_cursor(Movement::Right);
                    }
                    _ => (),
                }
            }
            Mode::Visual => todo!(),
        }
    }
}

fn is_crlf(c: char) -> bool {
    return c == '\n' || c == '\r';
}

pub struct EditorContent<T> {
    data: T,
}

pub trait EditorContentTrait {
    fn load_data(&mut self, raw_data: Vec<u8>);
    fn get_line(&self, i: u32) -> Option<String>;
    fn get_line_len(&self, i: u32) -> Option<u32>;
    fn write_char(&mut self, c: char, col: u32, row: u32);
    fn delete_char(&mut self, col: u32, row: u32);
}

impl EditorContent<Vec<char>> {
    fn new() -> EditorContent<Vec<char>> {
        Self {
            data: Vec::<char>::new(),
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

        if i < self.data.len() {
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

    fn delete_char(&mut self, col: u32, row: u32) {
        if let Some(i) = self.get_pos(col, row) {
            self.data.remove(i);
        }
    }
}
