use super::{Mode, Movement};

pub enum Action {
    Move(Movement),
    ChangeMode(Mode),
    InsertChar(char),
    Backspace,
    Delete,
    Quit,
    None,
}
