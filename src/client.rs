use std::{io::{stdout, BufRead, Stdout}, process::exit};

use crossterm::{cursor, execute, terminal::{Clear, ClearType}, ExecutableCommand};
use log::error;

use crate::editor::EditorV8;

pub struct ConsoleClient {
    stdout: Stdout,
    line_numbered: bool,
}

pub trait Client<T> {
    fn draw(&mut self, context: &T);
}

impl ConsoleClient {
    pub fn new(line_numbered: bool) -> Self {
        Self { 
            stdout: stdout(),
            line_numbered 
        }
    }
}

impl Client<EditorV8> for ConsoleClient {
    fn draw(&mut self, context: &EditorV8) {
        if context.file_path.is_none() {
            println!("no file provided!");
            return;
        }
        
        execute!(
            self.stdout, 
            Clear(ClearType::Purge),
            cursor::MoveTo(0, 0)
        ).unwrap_or_else(|_| {
            error!("unable to setup console");
            exit(1);
        });
        
        let lines = context.content.lines().enumerate();

        for (line_num, line) in lines {
            let display_str = line.unwrap_or("<ERROR>".to_string());
            
            if self.line_numbered {
                print!("{:>4}  ", line_num);
            }
                
            println!("{}", display_str);
        }
    }
}
