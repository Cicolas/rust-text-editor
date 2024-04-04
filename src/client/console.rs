use std::{
    cmp,
    io::{stdout, Stdout},
};

use crossterm::{
    cursor::{self, MoveTo, SetCursorStyle},
    event::{Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};

use crate::editor::{
    vector::CharVectorEditor, Action, EditorContentTrait, EditorEvent, Mode, Movement, Redraw,
};

use super::ClientEvent;

pub struct ConsoleClient {
    stdout: Stdout,
    line_numbered: bool,
}

impl ConsoleClient {
    pub fn new(line_numbered: bool) -> Self {
        Self {
            stdout: stdout(),
            line_numbered,
        }
    }

    fn pre_draw(&mut self, should_redraw: Redraw) -> Result<&mut Stdout, std::io::Error> {
        match should_redraw {
            Redraw::All => self
                .stdout
                .execute(MoveTo(0, 0))?
                .execute(Clear(ClearType::All))?
                .execute(Clear(ClearType::Purge)),
            Redraw::Line(line) => self
                .stdout
                .execute(MoveTo(0, line as u16))?
                .execute(Clear(ClearType::CurrentLine)),
            Redraw::Range(_, _) => todo!(),
        }
        // self.stdout.execute(cursor::Hide)?.execute(MoveTo(0, 0))
    }

    fn draw_line(&self, line_num: u32, content: String) {
        if self.line_numbered {
            print!("{:>4}  ", line_num + 1);
        }

        if cfg!(target_os = "windows") {
            println!("{}", content);
        } else {
            println!("{}\r", content);
        }
    }

    fn draw_cursor(&mut self, col: u32, row: u32, mode: Mode, view_start: u32) {
        let render_col = col;
        let render_row = row - view_start;

        let carret = match mode {
            Mode::Normal => SetCursorStyle::SteadyBlock,
            Mode::Insert => SetCursorStyle::BlinkingBar,
            Mode::Visual => SetCursorStyle::SteadyUnderScore,
        };

        execute!(
            self.stdout,
            cursor::Show,
            carret,
            cursor::MoveTo(
                (render_col + if self.line_numbered { 6 } else { 0 }) as u16,
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
            KeyCode::PageDown => vec![Action::ScrollBy(1), Action::AskRedraw(Redraw::All)],
            KeyCode::PageUp => vec![Action::ScrollBy(-1), Action::AskRedraw(Redraw::All)],
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
}

impl ClientEvent<CharVectorEditor> for ConsoleClient {
    fn load(&mut self, context: &mut CharVectorEditor) {
        enable_raw_mode().unwrap();

        let (_, h) = terminal::size().unwrap();
        context.on_action(vec![Action::Resize(h - 1)]);

        execute!(self.stdout, Clear(ClearType::All)).unwrap();
    }

    fn update(&mut self, context: &mut CharVectorEditor) -> Option<u8> {
        let event = crossterm::event::read();

        match event {
            Ok(Event::Key(key)) => {
                if key.kind == KeyEventKind::Release {
                    return None;
                }

                let actions = match context.mode {
                    Mode::Normal => self.normal_mode_keybinding(key),
                    Mode::Insert => self.insert_mode_keybinding(key),
                    Mode::Visual => todo!(),
                };

                context.on_action(actions);
            }
            Ok(Event::Resize(_, h)) => context.on_action(vec![Action::Resize(h - 1)]),
            _ => (),
        }

        None
    }

    fn draw(&mut self, context: &CharVectorEditor) {
        if context.file_path.is_none() {
            println!("no file provided!");
            return;
        }

        if let Some(redraw) = context.should_redraw {
            self.pre_draw(redraw).unwrap();
        }

        let mut line_num = context.view_start;

        match context.should_redraw {
            Some(Redraw::All) => {
                while let Some(line) = context.content.get_line(line_num) {
                    if line_num > context.view_end {
                        break;
                    }
                    self.draw_line(line_num, line);
                    line_num += 1;
                }
            }
            Some(Redraw::Line(line_num)) => {
                if let Some(line) = context.content.get_line(line_num) {
                    let clear_row = cmp::max(0, line_num as i32 - context.view_start as i32);
                    self.stdout.execute(MoveTo(0, clear_row as u16)).unwrap();
                    self.draw_line(line_num, line);
                }
            }
            Some(Redraw::Range(_, _)) => todo!(),
            None => (),
        }

        self.draw_cursor(
            context.render_col,
            context.render_row,
            context.mode,
            context.view_start,
        );
    }
}

impl Drop for ConsoleClient {
    fn drop(&mut self) {
        disable_raw_mode().unwrap()
    }
}
