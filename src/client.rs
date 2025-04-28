use core::panic;
use std::{collections::VecDeque, fmt::Error, os::unix::process::parent_id};

use crate::module::Module;

pub mod console;

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
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
    Resize(u16, u16, u16, u16),

    FocusMe,
    UnfocusMe,

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

#[derive(Debug, Clone, Copy)]
pub struct Container {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

pub struct LayoutNode {
    pub container: Container,
    pub module_id: Option<usize>,
    pub has_child: bool,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

pub struct ContainerLayout {
    pub layout_tree: Vec<LayoutNode>,
}
// pub type ContainerLayout = Vec<(Container, Option<usize>, Option<usize>)>;

#[derive(Debug, Clone, Copy)]
pub enum Fixture {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Min(Option<u32>, Option<u32>),
    Max(Option<u32>, Option<u32>),

    Strech(bool, bool),
    Shrink(bool, bool),

    FixOn(Fixture),
}

impl Default for Container {
    fn default() -> Self {
        Self {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        }
    }
}

impl Container {
    pub fn get_width(&self) -> u32 {
        self.right - self.left
    }

    pub fn get_height(&self) -> u32 {
        self.bottom - self.top
    }
}

impl ContainerLayout {
    pub fn new() -> Self {
        ContainerLayout {
            layout_tree: Vec::new(),
        }
    }
}

pub trait ContainerAutoFlow {
    fn setup_bsp(&mut self, expoent: u16, screen_w: u32, screen_h: u32);
    fn push_module(&mut self, module_id: usize, constraints: Vec<Constraint>) -> Result<(), Error>;
}

impl ContainerAutoFlow for ContainerLayout {
    fn setup_bsp(&mut self, expoent: u16, screen_w: u32, screen_h: u32) {
        self.layout_tree.push(LayoutNode {
            container: Container {
                top: 0,
                right: screen_w,
                bottom: screen_h,
                left: 0,
            },
            has_child: false,
            module_id: None,
            left: None,
            right: None,
        });

        let mut container_queue = VecDeque::<(usize, bool)>::new();
        container_queue.push_back((0, false));

        while let Some((container_idx, vertical)) = container_queue.pop_front() {
            let parent = self.layout_tree[container_idx].container;
            let edge_w = parent.left + (parent.get_width() / 2);
            let edge_h = parent.top + (parent.get_height() / 2);

            let left_idx = self.layout_tree.len();
            let left_container = Container {
                top: parent.top,
                right: if vertical { edge_w } else { parent.right },
                bottom: if !vertical { edge_h } else { parent.bottom },
                left: parent.left,
            };

            let right_idx = left_idx + 1;
            let right_container = Container {
                top: if !vertical { edge_h } else { parent.top },
                right: parent.right,
                bottom: parent.bottom,
                left: if vertical { edge_w } else { parent.left },
            };

            self.layout_tree.push(LayoutNode {
                container: left_container,
                module_id: None,
                has_child: false,
                left: None,
                right: None,
            });
            self.layout_tree.push(LayoutNode {
                container: right_container,
                module_id: None,
                has_child: false,
                left: None,
                right: None,
            });
            self.layout_tree[container_idx].left = Some(left_idx);
            self.layout_tree[container_idx].right = Some(right_idx);

            if self.layout_tree.len() < (1 << expoent) {
                container_queue.push_back((left_idx, !vertical));
                container_queue.push_back((right_idx, !vertical));
            }
        }
    }

    fn push_module(&mut self, module_id: usize, _constraints: Vec<Constraint>) -> Result<(), Error> {
        let mut container_queue = VecDeque::<usize>::new();
        container_queue.push_back(0);
        
        while let Some(layout_idx) = container_queue.pop_front() {
            if !self.layout_tree[layout_idx].has_child {
                self.layout_tree[layout_idx].module_id = Some(module_id);
                self.layout_tree[layout_idx].has_child = true;
                break;
            }
            
            if let Some(module) = self.layout_tree[layout_idx].module_id {
                let right_idx = self.layout_tree[layout_idx].right;
                let left_idx = self.layout_tree[layout_idx].left;

                if let Some(r_idx) = right_idx {
                    self.layout_tree[r_idx].module_id = Some(module_id);
                    self.layout_tree[r_idx].has_child = true;
                } else {
                    return Err(Error);
                }

                if let Some(l_idx) = left_idx {
                    self.layout_tree[l_idx].module_id = Some(module);
                    self.layout_tree[l_idx].has_child = true;
                } else {
                    return Err(Error);
                }

                self.layout_tree[layout_idx].module_id = None;
                break;
            }

            if let Some(left) = self.layout_tree[layout_idx].left {
                container_queue.push_back(left);
            }

            if let Some(right) = self.layout_tree[layout_idx].right {
                container_queue.push_back(right);
            }
        }

        Ok(())
    }
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
    fn change_focus(&mut self, idx: u32);
}
