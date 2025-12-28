use crossterm::event::KeyEvent;

use crate::client::{Action, Container, DrawAction, console::{IncomingConsoleEvent, OutcomingConsoleEvent}};

pub mod command;
pub mod editor;

pub trait ModuleView {
    fn get_container(&self) -> &Container;
}

pub trait ModuleEvent {
    fn proxy_trigger_events(&mut self, _event: &IncomingConsoleEvent, idx: u32) -> Option<Vec<OutcomingConsoleEvent>> {
        None
    }
    fn on_load(&mut self) {}
    fn on_event(&mut self, _event: IncomingConsoleEvent) -> Option<Vec<OutcomingConsoleEvent>> { None }
    fn on_draw(&self) -> Option<Vec<DrawAction>> { None }
    fn on_resize(&mut self, _top: u32, _right: u32, _bottom: u32, _left: u32) {}
    fn on_destroy(&self) {}
}

pub trait Module: ModuleView + ModuleEvent {}