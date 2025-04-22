use crate::module::Module;

pub mod console;

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    Up,
    Down,
    Left,
    Right,
    LineEnd,
    LineStart,
}

#[allow(unused)]
#[derive(PartialEq, Eq, Clone)]
pub enum Action {
    Move(Movement),
    ChangeMode(Mode),
    InsertChar(char),
    Backspace,
    Delete,
    Quit,
    None,

    ScrollBy(i32),
    // ScrollTo(u32),
    Resize(u16, u16),

    OpenFile(String),
    WriteFile(String),
    SaveFile,
}

pub enum DrawAction {
    CursorTo(u32, u32),
    AskRedraw(Redraw),
}

#[allow(unused)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Redraw {
    Cursor,
    All,
    Line(u32, String),
    Range(u32, u32),
}

pub trait ClientEvent {
    fn load(&mut self);
    fn update(&mut self) -> Option<u8>;
    fn draw(&mut self);
    fn before_quit(&mut self);

    fn handle_file(&mut self, path: String);
}

pub trait ClientModular {
    fn attach_module(&mut self, module: Box<dyn Module>);
}
