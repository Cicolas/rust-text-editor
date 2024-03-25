use std::{any::Any, fs::File, io::{stdin, stdout, Read, Stdin, Stdout, Write}, process::exit};

use crossterm::{cursor, event::{read, Event, KeyCode, KeyEventKind, KeyEventState}, execute, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType}, ExecutableCommand};
use log::{error, info};

pub type EditorV8 = Editor<Vec<u8>>;

#[derive(Debug)]
pub struct Editor<T> {
    pub stdout: Stdout,
    pub file_path: Option<String>,
    pub content: T,
    pub cursor: u64,
    pub row: u32,
    pub col: u32
}

pub trait EditorIO {
    fn open_file(&mut self, path: &str) -> Result<(), std::io::Error>;
    fn write_file(path: &str, data: Vec<u8>);
}

pub trait EditorEvent {
    fn on_load(&mut self);
    fn on_load_file(&mut self, path: String);
    fn on_update(&mut self) -> Option<u8>;
    fn on_write(&mut self, keycode: u32);
    fn on_exit(&mut self);
}

impl EditorV8 {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            file_path: None,            
            content: Vec::new(),
            cursor: 0,
            row: 0,
            col: 0,
        }
    }
}

impl EditorIO for EditorV8 {
    fn open_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;
        file.read_to_end(&mut self.content)?;
        Ok(())
    }

    fn write_file(path: &str, data: Vec<u8>) {
        todo!()
    }
}

impl EditorEvent for EditorV8 {
    fn on_load(&mut self) {
        enable_raw_mode().unwrap_or_else(|_|
            error!("unable to enable raw mode!")
        );

        execute!(self.stdout, Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap_or_else(|_| {
            error!("unable to setup console");
            exit(1);
        });
    }

    fn on_load_file(&mut self, path: String) {
        info!("loading file '{}'", path);
        
        self.open_file(path.as_str()).unwrap_or_else(|_err|
            error!("error while loading") 
        );
        
        self.file_path = Some(path);
    }

    fn on_update(&mut self) -> Option<u8> {
        match read().unwrap() {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    return None;
                }
                
                match key.code {
                    KeyCode::Char(c) => {
                        self.on_write(c as u32);
                    },
                    KeyCode::Backspace => {
                        self.on_write(0x08);  
                    },
                    KeyCode::Enter => {
                        self.on_write(0x0D);
                    },
                    KeyCode::Esc => {
                        return Some(1);
                    },
                    _ => {}
                }              
            },
            _ => ()
        }
        
        None
    }
    
    fn on_write(&mut self, keycode: u32) {
        // info!("{:4x}", keycode);
    }

    fn on_exit(&mut self) {
        disable_raw_mode().unwrap_or_else(|_|
            error!("unable to disable raw mode!")
        )
    }
}