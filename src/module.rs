use editor::Container;

use crate::client::{Action, DrawAction};

pub mod command;
pub mod editor;

pub trait ModuleView {
    fn get_container(&self) -> &Container;
}

pub trait ModuleEvent {
    fn on_load(&mut self) {}
    fn on_action(&mut self, _actions: &Vec<Action>) -> Option<Vec<Action>> { None }
    fn on_draw(&self) -> Option<Vec<DrawAction>> { None }
    fn on_destroy(&self) {}
}

pub trait Module: ModuleView + ModuleEvent {}