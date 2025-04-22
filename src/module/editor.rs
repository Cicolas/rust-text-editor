use std::{
    cmp,
    fs::File,
    io::{Read, Write},
};

use log::{debug, error, info};

use crate::client::{Action, DrawAction, Mode, Movement, Redraw};
use crate::utils::TruncAt;

use super::{Module, ModuleEvent, ModuleView};

pub mod vector;

#[derive(Debug, Clone)]
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

pub trait EditorIO {
    fn open_file(&mut self, path: &str) -> Result<(), std::io::Error>;
    fn save_file(&self) -> Result<(), std::io::Error>;
    fn write_file(&self, path: &str) -> Result<(), std::io::Error>;
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
    pub file_path: Option<String>,
    pub content: T,
    pub render_row: u32,
    pub row: u32,
    pub render_col: u32,
    pub col: u32,
    pub mode: Mode,
    pub should_redraw: Option<Redraw>,
    pub view: Container,
    pub line_numbered: bool,
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
            mode: Mode::Normal,
            should_redraw: None,
            view: Container::default(),
            line_numbered: true,
            // view_start: 0,
            // view_end: 0,
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
                if self.render_col == self.view.right - self.get_offset() {
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

    // fn draw_line(&self, line_num: u32, content: String, len: u32) {
    //     if self.line_numbered {
    //         print!("{:>4}  ", line_num + 1);
    //     }

    //     let striped_content = content.with_exact_width(len as usize);

    //     if cfg!(target_os = "windows") {
    //         println!("{}", striped_content);
    //     } else {
    //         println!("{}\r", striped_content);
    //     }
    // }
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

impl<T: EditorContentTrait> ModuleEvent for Editor<T> {
    fn on_action(&mut self, actions: &Vec<Action>) -> Option<Vec<Action>> {
        self.should_redraw = None;

        let mut return_vec = Vec::<Action>::new();

        actions.iter().for_each(|action| {
            match action {
                Action::OpenFile(path) => {
                    info!("loading file '{}'\r", path);

                    if let Err(error) = self.open_file(path.as_str()) {
                        error!("error while loading '{}': {}", path, error);
                    } else {
                        self.file_path = Some(path.clone());
                        info!("'{}' loaded!\r", path);
                        self.should_redraw = Some(Redraw::All);
                    }
                }
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
                    self.move_cursor(Movement::Left);
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
                Action::Resize(width, height) => {
                    self.view.bottom = self.view.top + (*height) as u32 - 1;
                    self.view.right = self.view.left + (*width) as u32 - 1;

                    self.should_redraw = Some(Redraw::All);
                }
                Action::SaveFile => {
                    self.save_file().unwrap();
                }
                Action::ChangeMode(mode) => {
                    self.should_redraw = Some(Redraw::Cursor);
                    return_vec.push(Action::ChangeMode(mode.clone()));
                }
                Action::None => {}
                a => {
                    let post_action = a.clone();
                    return_vec.push(post_action);
                }
            };
        });

        if return_vec.is_empty() {
            None
        } else {
            Some(return_vec)
        }
    }

    fn on_draw(&self) -> Option<Vec<DrawAction>> {
        if self.file_path.is_none() {
            return None;
        }

        let mut drawing_actions = vec![];

        match &self.should_redraw {
            Some(Redraw::All) => {
                debug!("all");
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
            }
            Some(Redraw::Line(y, str)) => {
                debug!("line");
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
                debug!("cursor");
            }
            Some(Redraw::Range(_, _)) => todo!(),
            None => {
                debug!("none");
                return None;
            }
        }

        if self.line_numbered {
            drawing_actions.push(DrawAction::CursorTo(self.render_col + 6, self.render_row));
        } else {
            drawing_actions.push(DrawAction::CursorTo(self.render_col, self.render_row));
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
