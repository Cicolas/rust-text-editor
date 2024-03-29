#[derive(Clone, Copy)]
pub enum Redraw {
    All,
    Line(u32),
    Range(u32, u32),
}
