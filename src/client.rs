use std::io::{stdout, Stdout};

use crossterm::{
    cursor::{self, MoveTo, SetCursorStyle},
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};

use crate::editor::{
    actions::Action, EditorContentTrait, EditorEvent, Mode, Movement, VectorEditor,
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

    fn pre_draw(&mut self) -> Result<&mut Stdout, std::io::Error> {
        self.stdout
            .execute(Clear(ClearType::All))?
            .execute(MoveTo(0, 0))
    }

    fn normal_mode_keybinding(&self, key: KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Char(c) => match c {
                'k' => vec![Action::Move(Movement::Up)],
                'j' => vec![Action::Move(Movement::Down)],
                'h' => vec![Action::Move(Movement::Left)],
                'l' => vec![Action::Move(Movement::Right)],
                'q' => vec![Action::Quit],
                'i' => vec![Action::ChangeMode(Mode::Insert)],
                'I' => vec![
                    Action::Move(Movement::LineStart),
                    Action::ChangeMode(Mode::Insert),
                ],
                'a' => vec![
                    Action::Move(Movement::Right),
                    Action::ChangeMode(Mode::Insert),
                ],
                'A' => vec![
                    Action::Move(Movement::LineEnd),
                    Action::ChangeMode(Mode::Insert),
                ],
                _ => vec![Action::None],
            },
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

        execute!(self.stdout, Clear(ClearType::All)).unwrap();
    }

    fn update(&mut self, context: &mut VectorEditor) -> Option<u8> {
        match read().unwrap() {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    return None;
                }

                let actions = match context.mode {
                    Mode::Normal => self.normal_mode_keybinding(key),
                    Mode::Insert => self.insert_mode_keybinding(key),
                    Mode::Visual => todo!(),
                };

                context.on_actions(actions);
            }
            _ => (),
        }

        None
    }

    fn draw(&mut self, context: &VectorEditor) {
        if context.file_path.is_none() {
            println!("no file provided!");
            return;
        }

        self.pre_draw().unwrap();

        let mut line_num = 0;

        while let Some(line) = context.content.get_line(line_num) {
            // for (line_num, line) in lines {
            if self.line_numbered {
                print!("{:>4}  ", line_num + 1);
            }

            println!("{}", line);
            line_num += 1;
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
