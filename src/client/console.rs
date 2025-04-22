use std::{
    io::{stdout, Stdout}, vec
};

use crossterm::{
    cursor::{self, MoveTo, SetCursorStyle},
    event::{Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand,
};
use pad::PadStr;

use crate::module::{editor::Container, Module};

use super::{Action, ClientEvent, ClientModular, DrawAction, Mode, Movement, Redraw};

pub struct ConsoleClient {
    stdout: Stdout,
    console_mode: Mode,
    modules: Vec<Box<dyn Module>>,
}

impl ConsoleClient {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            console_mode: Mode::Normal,
            modules: Vec::new(),
        }
    }

    fn draw_line(&self, content: String, len: u32) {
        let striped_content = content.with_exact_width(len as usize);

        if cfg!(target_os = "windows") {
            println!("{}", striped_content);
        } else {
            println!("{}\r", striped_content);
        }
    }

    fn erase_line(&self, len: u32) {
        let striped_content = "".with_exact_width(len as usize);

        if cfg!(target_os = "windows") {
            println!("{}", striped_content);
        } else {
            println!("{}\r", striped_content);
        }
    }

    fn get_cursor_style(&self, mode: Mode) -> SetCursorStyle {
        return match mode {
            Mode::Normal => SetCursorStyle::SteadyBlock,
            Mode::Insert => SetCursorStyle::BlinkingBar,
            Mode::Visual => SetCursorStyle::SteadyUnderScore,
            Mode::Command => SetCursorStyle::BlinkingBar,
        }
    }

    fn draw_cursor(&mut self, col: u32, row: u32, mode: Mode, view: &Container) {
        let render_col = col - view.left;
        let render_row = row - view.top;

        let carret = self.get_cursor_style(mode);

        execute!(
            self.stdout,
            cursor::Show,
            carret,
            cursor::MoveTo(
                render_col as u16,
                render_row as u16
            )
        )
        .unwrap();
    }

    fn normal_mode_keybinding(&self, key: KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Char('k') => vec![Action::Move(Movement::Up)],
            KeyCode::Char('j') => vec![Action::Move(Movement::Down)],
            KeyCode::Char('h') => vec![Action::Move(Movement::Left)],
            KeyCode::Char('l') => vec![Action::Move(Movement::Right)],
            KeyCode::Char('q') => vec![Action::Quit],
            KeyCode::Char('i') => vec![Action::ChangeMode(Mode::Insert)],
            KeyCode::Char('I') => vec![
                Action::Move(Movement::LineStart),
                Action::ChangeMode(Mode::Insert),
            ],
            KeyCode::Char('a') => vec![
                Action::Move(Movement::Right),
                Action::ChangeMode(Mode::Insert),
            ],
            KeyCode::Char('A') => vec![
                Action::Move(Movement::LineEnd),
                Action::ChangeMode(Mode::Insert),
            ],
            KeyCode::Char('s') => vec![Action::SaveFile],
            KeyCode::Char(':') => vec![Action::ChangeMode(Mode::Command)],
            KeyCode::PageDown => vec![Action::ScrollBy(1)],
            KeyCode::PageUp => vec![Action::ScrollBy(-1)],
            KeyCode::Backspace => vec![Action::Move(Movement::Left)],
            KeyCode::Enter => vec![Action::Move(Movement::Down)],
            KeyCode::Esc => vec![Action::Quit],
            KeyCode::Up => vec![Action::Move(Movement::Up)],
            KeyCode::Down => vec![Action::Move(Movement::Down)],
            KeyCode::Left => vec![Action::Move(Movement::Left)],
            KeyCode::Right => vec![Action::Move(Movement::Right)],
            _ => vec![Action::None],
        }
    }

    fn insert_mode_keybinding(&self, key: KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Char(c) => vec![Action::InsertChar(c)],
            KeyCode::Backspace => vec![Action::Backspace],
            KeyCode::Delete => vec![Action::Delete],
            KeyCode::Up => vec![Action::Move(Movement::Up)],
            KeyCode::Down => vec![Action::Move(Movement::Down)],
            KeyCode::Left => vec![Action::Move(Movement::Left)],
            KeyCode::Right => vec![Action::Move(Movement::Right)],
            KeyCode::Esc => vec![Action::ChangeMode(Mode::Normal)],
            KeyCode::Enter => vec![Action::InsertChar('\n')],
            _ => vec![Action::None],
        }
    }

    fn command_mode_keybinding(&self, key: KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Char(c) => vec![Action::InsertChar(c)],
            KeyCode::Esc => vec![Action::ChangeMode(Mode::Normal)],
            _ => vec![Action::None]
        }
    }

    fn trigger_actions(&mut self, actions: Vec<Action>) {
        let mut all_post_actions = Vec::new();

        for module in self.modules.iter_mut() {
            let m = module.as_mut();
            
            if let Some(post_actions) = m.on_action(&actions) {
                for action in post_actions {
                    all_post_actions.push(action);
                }
            }
        }

        for action in all_post_actions {
            match action {
                Action::ChangeMode(mode) => {
                    let carret = self.get_cursor_style(mode);

                    execute!(
                        self.stdout,
                        // cursor::Show,
                        carret,
                    ).unwrap();
                    
                    self.console_mode = mode;
                },
                _ => todo!()
            }
        }
    }

    fn trigger_drawing(&mut self) {
        let mut all_draw_actions = Vec::new();
        
        for module in self.modules.iter_mut() {
            let m = module.as_mut();
            let container = m.get_container();
            
            if let Some(draw_actions) = m.on_draw() {
                for action in draw_actions {
                    all_draw_actions.push((action, container.clone()));
                }
            }
        }
        
        for (action, container) in all_draw_actions {
            match action {
                DrawAction::CursorTo(x, y) => {
                    self.draw_cursor(x, y, self.console_mode, &container);
                },
                DrawAction::AskRedraw(redraw) => {
                    match redraw {
                        Redraw::All => {
                            self.stdout
                                .execute(MoveTo(0, 0))
                                .unwrap()
                                .execute(cursor::Hide)
                                .unwrap();

                            for line_num in 0..container.get_height() {
                                self.stdout.execute(MoveTo(0, line_num as u16)).unwrap();
                                self.erase_line(container.get_width());
                            }
                        }
                        Redraw::Line(y, line) => {
                            self.stdout.execute(MoveTo(0, y as u16)).unwrap();
            
                            self.draw_line(
                                line,
                                container.get_width(),
                            );
                        }
                        Redraw::Range(_, _) => todo!(),
                        Redraw::Cursor => { todo!() }
                    }
                },
            }
        }
    }
}

impl ClientEvent for ConsoleClient {
    fn load(&mut self) {
        enable_raw_mode().unwrap();

        let (w, h) = terminal::size().unwrap();
        self.trigger_actions(vec![Action::Resize(w, h - 2)]);

        execute!(self.stdout, EnterAlternateScreen, Clear(ClearType::All)).unwrap();
    }

    fn update(&mut self) -> Option<u8> {
        let event = crossterm::event::read();

        match event {
            Ok(Event::Key(key)) => {
                if key.kind == KeyEventKind::Release {
                    return None;
                }

                let actions = match self.console_mode {
                    Mode::Normal => self.normal_mode_keybinding(key),
                    Mode::Insert => self.insert_mode_keybinding(key),
                    Mode::Visual => todo!(),
                    Mode::Command => self.command_mode_keybinding(key)
                };

                match actions.first() {
                    Some(Action::Quit) => {
                        execute!(self.stdout, MoveTo(0, 0), Clear(ClearType::All)).unwrap();
                        return Some(0);
                    }
                    Some(_) => {
                        self.trigger_actions(actions);
                    }
                    None => {
                        panic!("Invalid Action");
                    }
                }

                // if self.console_mode == Mode::Command {
                //     actions.iter().for_each(|action| {
                //         match *action {
                //             Action::InsertChar(c) => self.command_str.push(c),
                //             Action::ChangeMode(mode) => self.console_mode = mode, 
                //             _ => { todo!() }
                //         }
                //     });
                // }
        
            }
            Ok(Event::Resize(w, h)) => {
                self.trigger_actions(vec![Action::Resize(
                    w,
                    h - 2,
                )]);
                // self.update_modules(vec![Action::Resize(
                //     if self.line_numbered { w - 6 } else { w },
                //     h - 1,
                // )]);
            }
            _ => (),
        } 

        None
    }

    fn draw(&mut self) {
        self.trigger_drawing();
    }

    fn before_quit(&mut self) {
        execute!(self.stdout, LeaveAlternateScreen).unwrap();
    }
    
    fn handle_file(&mut self, path: String) {
        self.trigger_actions(vec![Action::OpenFile(path)]);
    }
}

impl ClientModular for ConsoleClient {
    fn attach_module(&mut self, mut module: Box<dyn Module>) {
        module.on_load();

        self.modules.push(module);
    }
}

impl Drop for ConsoleClient {
    fn drop(&mut self) {
        disable_raw_mode().unwrap()
    }
}
