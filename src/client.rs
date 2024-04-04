use crate::editor::{EditorEvent, EditorIO};

pub mod console;

pub trait ClientEvent<T>
where
    T: EditorEvent + EditorIO,
{
    fn load(&mut self, context: &mut T);
    fn update(&mut self, context: &mut T) -> Option<u8>;
    fn draw(&mut self, context: &T);
}
