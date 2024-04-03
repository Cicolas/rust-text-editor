use std::io::Write;

use crate::utils::is_crlf;

use super::{Editor, EditorContent, EditorContentTrait};

pub type VectorEditor<T> = Editor<EditorContent<Vec<T>>>;
pub type CharVectorEditor = VectorEditor<char>;

impl EditorContent<Vec<char>> {
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
    fn new() -> EditorContent<Vec<char>> {
        Self {
            data: Vec::<char>::new(),
            is_crlf: true,
        }
    }

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
