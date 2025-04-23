use std::cmp;

use log::debug;

use crate::client::{Action, DrawAction, Mode, Movement, Redraw};

use super::{editor::Container, Module, ModuleEvent, ModuleView};

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
}

impl ModuleEvent for CommandModule {
    fn on_action(&mut self, actions: &Vec<Action>) -> Option<Vec<Action>> {
        let mut return_vec = Vec::<Action>::new();

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
                Action::ChangeMode(mode) => {
                    match mode {
                        Mode::Normal => {
                            self.command_str = String::new();
                            return_vec.push(Action::ChangeMode(Mode::Normal));
                        },
                        _ => {}
                    }
                },
                Action::InsertChar('\n') => {
                    debug!("{}", self.command_str);
                    self.command_str = String::new();
                    self.render_col = 0;
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
                Action::Resize(w, _) => {
                    self.width = *w as u32;
                }
                _ => {}
            }
        }

        Some(return_vec)
    }

    fn on_draw(&self) -> Option<Vec<crate::client::DrawAction>> {
        let mut str = String::from(":");
        str.push_str(&self.command_str);
        
        Some(vec![
            DrawAction::AskRedraw(
                Redraw::Line(0, str)
            ),
            DrawAction::CursorTo(self.render_col + 1 as u32, 0),
        ])
    }
}

impl ModuleView for CommandModule {
    fn get_container(&self) -> &Container {
        &Container { top: 0, left: 0, bottom: 1, right: 20 }
    }
}

impl Module for CommandModule {}
