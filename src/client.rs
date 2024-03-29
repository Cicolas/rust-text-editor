use std::io::{stdout, Cursor, Stdout};

use crossterm::{
    cursor::{self, MoveTo, SetCursorStyle},
    event::{Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use log::info;

use crate::editor::{
    actions::Action, redraw::Redraw, EditorContentTrait, EditorEvent, Mode, Movement, VectorEditor,
};

pub struct ConsoleClient {
    stdout: Stdout,
    line_numbered: bool,
}

pub trait Client<T> {
    fn load(&mut self);
    fn update(&mut self, context: &mut T) -> Option<u8>;
    fn draw(&mut self, context: &T);
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
                .execute(Clear(ClearType::All)),
            Redraw::Line(line) => self
                .stdout
                .execute(MoveTo(0, line as u16))?
                .execute(Clear(ClearType::CurrentLine)),
            Redraw::Range(_, _) => todo!(),
        }
        // self.stdout.execute(cursor::Hide)?.execute(MoveTo(0, 0))
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

impl Client<VectorEditor> for ConsoleClient {
    fn load(&mut self) {
        enable_raw_mode().unwrap();

        execute!(self.stdout, terminal::EnterAlternateScreen).unwrap();
    }

    fn update(&mut self, context: &mut VectorEditor) -> Option<u8> {
        if let Event::Key(key) = crossterm::event::read().unwrap() {
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

        None
    }

    fn draw(&mut self, context: &VectorEditor) {
        if context.file_path.is_none() {
            println!("no file provided!");
            return;
        }

        if let Some(redraw) = context.should_redraw {
            self.pre_draw(redraw).unwrap();
        }

        let mut line_num = 0;

        match context.should_redraw {
            Some(Redraw::All) => {
                while let Some(line) = context.content.get_line(line_num) {
                    // for (line_num, line) in lines {
                    if self.line_numbered {
                        print!("{:>4}  ", line_num + 1);
                    }

                    println!("{}", line);
                    line_num += 1;
                }
            }
            Some(Redraw::Line(line)) => {
                self.stdout.execute(MoveTo(0, line as u16)).unwrap();
                if self.line_numbered {
                    print!("{:>4}  ", line + 1);
                }
                println!("{}", context.content.get_line(line).unwrap());
            }
            Some(Redraw::Range(_, _)) => todo!(),
            None => (),
        }

        let cursor_x = context.render_col + 6;
        let cursor_y = context.row;

        let carret = match context.mode {
            Mode::Normal => SetCursorStyle::SteadyBlock,
            Mode::Insert => SetCursorStyle::BlinkingBar,
            Mode::Visual => SetCursorStyle::SteadyUnderScore,
        };

        let _result = execute!(
            self.stdout,
            cursor::Show,
            carret,
            cursor::MoveTo(cursor_x as u16, cursor_y as u16)
        );
    }
}

impl Drop for ConsoleClient {
    fn drop(&mut self) {
        disable_raw_mode().unwrap()
    }
}
