use std::{cmp, fs::File, io::Read, process::exit};

use log::{error, info};

pub type VectorEditor = Editor<Vec<char>>;

enum Movement {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub enum Mode {
    NORMAL,
    INSERT,
    VISUAL,
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
            mode: Mode::NORMAL,
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
            Movement::UP => {
                self.row = cmp::max(0, self.row as i32 - 1) as u32;
            }
            Movement::DOWN => {
                self.row += 1;
            }
            Movement::LEFT => {
                if self.col == 0 && self.row != 0 {
                    self.row = cmp::max(0, self.row as i32 - 1) as u32;

                    wrap_left = true;
                } else {
                    self.col = cmp::max(0, cmp::min(self.render_col, self.col) as i32 - 1) as u32;
                }
            }
            Movement::RIGHT => {
                self.col += 1;

                if self.col > line_len {
                    self.col = 0;
                    self.row += 1;
                }
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
                self.move_cursor(Movement::UP);
            }
            // DOWN
            _ if keycode == 0x28 => {
                self.move_cursor(Movement::DOWN);
            }
            // LEFT
            _ if keycode == 0x25 => {
                self.move_cursor(Movement::LEFT);
            }
            // RIGHT
            _ if keycode == 0x27 => {
                self.move_cursor(Movement::RIGHT);
            }
            _ => (),
        }

        match self.mode {
            Mode::NORMAL => match char::from_u32(keycode) {
                Some('h') => {
                    self.move_cursor(Movement::LEFT);
                }
                Some('j') => {
                    self.move_cursor(Movement::DOWN);
                }
                Some('k') => {
                    self.move_cursor(Movement::UP);
                }
                Some('l') => {
                    self.move_cursor(Movement::RIGHT);
                }
                Some('i') => self.mode = Mode::INSERT,
                Some('a') => {
                    self.move_cursor(Movement::RIGHT);
                    self.mode = Mode::INSERT;
                }
                Some('q') => {
                    exit(0);
                }
                _ => {
                    if keycode == 0x1B {
                        exit(0);
                    }
                },
            },
            Mode::INSERT => {
                match char::from_u32(keycode) {
                    // ESC
                    _ if keycode == 0x1B => {
                        self.mode = Mode::NORMAL;
                    }
                    _ => (),
                }
            }
            Mode::VISUAL => todo!(),
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
}

impl EditorContent<Vec<char>> {
    fn new() -> EditorContent<Vec<char>> {
        Self {
            data: Vec::<char>::new(),
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
}
