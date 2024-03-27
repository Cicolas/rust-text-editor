use std::io::{stdout, Stdout};

use crossterm::{
    cursor::{self, MoveTo, SetCursorStyle},
    event::{read, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};

use crate::editor::{EditorContentTrait, EditorEvent, Mode, VectorEditor};

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
            .execute(Clear(ClearType::Purge))?
            .execute(MoveTo(0, 0))
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

                let code = match key.code {
                    KeyCode::Char(c)   => c as u32,
                    KeyCode::Backspace => 0x08,
                    KeyCode::Enter     => 0x0D,
                    KeyCode::Esc       => 0x1B,
                    KeyCode::Up        => 0x26,
                    KeyCode::Down      => 0x28,
                    KeyCode::Left      => 0x25,
                    KeyCode::Right     => 0x27,
                    _                  => 0
                };

                context.on_write(code);
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
                print!("{:>4}  ", line_num);
            }

            println!("{}", line);
            line_num += 1;
        }

        let cursor_x = context.render_col + 6;
        let cursor_y = context.row;

        let carret = match context.mode {
            Mode::NORMAL => SetCursorStyle::SteadyBlock,
            Mode::INSERT => SetCursorStyle::BlinkingBar,
            Mode::VISUAL => SetCursorStyle::SteadyUnderScore,
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
