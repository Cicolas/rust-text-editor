use super::{redraw::Redraw, Mode, Movement};

pub enum Action {
    Move(Movement),
    ChangeMode(Mode),
    InsertChar(char),
    Backspace,
    Delete,
    Quit,
    None,

    ScrollBy(i32),
    ScrollTo(u32),
    Resize(u16),

    OpenFile(String),
    WriteFile(String),
    SaveFile,

    AskRedraw(Redraw),
}
