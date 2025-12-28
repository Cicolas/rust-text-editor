use std::cmp;

use crossterm::{cursor::SetCursorStyle, event::{KeyCode, KeyEvent}};
use log::debug;

use crate::client::{Action, Container, DrawAction, Mode, Movement, Redraw, console::{IncomingConsoleEvent, OutcomingConsoleEvent}};

use super::{Module, ModuleEvent, ModuleView};

pub struct CommandModule {
    pub width: u32,
    pub render_col: u32,
    pub command_str: String,
}

impl CommandModule {
    pub fn new() -> Self {
        CommandModule {
            width: 0,
            render_col: 0,
            command_str: String::new(),
        }
    }

    fn convert_key_to_actions(&mut self, key: KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Char(c) => vec![Action::InsertChar(c)],
            KeyCode::Esc => vec![Action::ChangeMode(Mode::Normal)],
            KeyCode::Backspace => vec![Action::Backspace],
            KeyCode::Delete => vec![Action::Delete],
            KeyCode::Left => vec![Action::Move(Movement::Left)],
            KeyCode::Right => vec![Action::Move(Movement::Right)],
            KeyCode::Enter => vec![Action::InsertChar('\n')],
            _ => vec![Action::None],
        }
    }

    fn trigger_actions(&mut self, actions: &Vec<Action>) -> Option<Vec<OutcomingConsoleEvent>> {
        for action in actions.into_iter() {
            match action {
                Action::Move(movement) => {
                    match movement {
                        Movement::Left => {
                            self.render_col = cmp::max(0, (self.render_col as i32) - 1) as u32;
                        },
                        Movement::Right => {
                            self.render_col = cmp::min(self.command_str.len() as u32, self.render_col + 1) as u32;
                        },
                        Movement::LineEnd => self.render_col = self.command_str.len() as u32,
                        Movement::LineStart => self.render_col = 0,
                        _ => {}
                    }
                },
                Action::InsertChar('\n') => {
                    return Some(self.process_command());
                },
                Action::InsertChar(c) => {
                    self.render_col += 1;
                    self.command_str.push(*c)
                },
                Action::Backspace => {
                    if self.command_str.len() > 0 {
                        let remove_col = self.render_col as i32 - 1;
                        if remove_col >= 0 {
                            self.command_str.remove(remove_col as usize);
                        }
                        self.render_col = cmp::max(0, remove_col) as u32;
                    }
                },
                Action::Delete => {
                    if self.command_str.len() > 0 {
                        if self.render_col < self.command_str.len() as u32 {
                            self.command_str.remove(self.render_col as usize);
                        }
                        self.render_col = cmp::min(self.command_str.len() as u32, self.render_col) as u32;
                    }
                },
                Action::Resize(_, right, _, left) => {
                    self.width = (*right - *left) as u32;
                }
                _ => {}
            }
        }

        None
    }

    fn process_command(&mut self) -> Vec<OutcomingConsoleEvent> {
        debug!("Processing command: {}", self.command_str);

        let command = self.command_str.clone();
        self.command_str.clear();
        self.render_col = 0;

        match command.as_str() {
            "q" | "quit" => {
                vec![OutcomingConsoleEvent::Quit]
            }
            _ => panic!("Command unknown")
        }
    }
}

impl ModuleEvent for CommandModule {
    fn on_event(&mut self, event: IncomingConsoleEvent) -> Option<Vec<OutcomingConsoleEvent>> {
        match event {
            IncomingConsoleEvent::Key(key) => {
                let actions = self.convert_key_to_actions(key);
                self.trigger_actions(&actions)
            },
            _ => None,
        }
    }

    fn on_draw(&self) -> Option<Vec<crate::client::DrawAction>> {
        let mut str = String::from(":");
        str.push_str(&self.command_str);
        
        Some(vec![
            DrawAction::AskRedraw(
                Redraw::Line(0, str)
            ),
            DrawAction::CursorTo(self.render_col + 1 as u32, 0, SetCursorStyle::BlinkingBlock),
        ])
    }
}

impl ModuleView for CommandModule {
    fn get_container(&self) -> &Container {
        &Container { top: 0, left: 0, bottom: 1, right: 20 }
    }
}

impl Module for CommandModule {}
