use std::{
    cell::{Ref, RefCell},
    io::{stdout, Stdout},
    ops::Index,
    path::PathBuf,
    usize, vec,
};

use crossterm::{
    cursor::{self, MoveTo, SetCursorStyle},
    event::{Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use log::warn;
use pad::PadStr;

use crate::module::Module;

use super::{
    Action, ClientEvent, ClientModular, Container, ContainerAutoFlow, ContainerLayout, DrawAction,
    Mode, Movement, Redraw,
};

const BSP_EXPOENT: u16 = 3;

#[derive(Clone)]
pub enum IncomingConsoleEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    File(PathBuf),
}

pub enum OutcomingConsoleEvent {
    FocusMe(u32),
    UnfocusMe,
    Quit,
    Message(String, String), // (module, message)
    DisableProxy,
    EnableProxy,
    Interrupt,
    None,
}

pub struct ConsoleClient {
    stdout: Stdout,
    modules: Vec<Box<dyn Module>>,
    focus_stack: Vec<u32>,
    containers: ContainerLayout,
    proxy_enabled: bool,
}

impl ConsoleClient {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            modules: Vec::new(),
            focus_stack: Vec::new(),
            containers: ContainerLayout::new(),
            proxy_enabled: true,
        }
    }

    fn draw_line(&self, content: String, len: u32) {
        let striped_content = content.with_exact_width(len as usize);

        if cfg!(target_os = "windows") {
            println!("{}", striped_content);
        } else {
            println!("{}\r", striped_content);
        }
    }

    fn erase_line(&self, len: u32) {
        let striped_content = "".with_exact_width(len as usize);

        if cfg!(target_os = "windows") {
            println!("{}", striped_content);
        } else {
            println!("{}\r", striped_content);
        }
    }

    fn draw_cursor(&mut self, col: u32, row: u32, cursor_style: SetCursorStyle, view: &Container) {
        let render_col = col + view.left;
        let render_row = row + view.top;

        let carret = cursor_style;

        execute!(
            self.stdout,
            cursor::Show,
            carret,
            cursor::MoveTo(render_col as u16, render_row as u16)
        )
        .unwrap();
    }

    fn trigger_proxy(&mut self, event: IncomingConsoleEvent) -> Vec<OutcomingConsoleEvent> {
        let mut all_post_actions = Vec::new();

        for idx in 0..self.modules.len() {
            let mut module = self.modules.get_mut(idx);

            if let Some(m) = module.as_mut() {
                if let Some(post_actions) = m.proxy_trigger_events(&event, idx as u32) {
                    for action in post_actions {
                        all_post_actions.push(action);
                    }
                }
            }
        }

        all_post_actions
    }

    fn trigger_events(&mut self, event: IncomingConsoleEvent) -> Vec<OutcomingConsoleEvent> {
        if self.focus_stack.last().is_none() {
            warn!("Any module on focus");
            return vec![OutcomingConsoleEvent::None];
        }

        let idx = *self.focus_stack.last().unwrap();
        let mut module = self.modules.get_mut(idx as usize);
        let mut all_post_actions = Vec::new();

        if let Some(m) = module.as_mut() {
            if let Some(post_actions) = m.on_event(event) {
                for action in post_actions {
                    all_post_actions.push(action);
                }
            }
        }

        all_post_actions
    }

    fn trigger_drawing(&mut self) {
        let mut all_draw_actions = Vec::new();

        for idx in 0..self.modules.len() {
            let m = self.modules[idx].as_mut();
            let container = self.containers.get_module(idx).unwrap();

            if let Some(draw_actions) = m.on_draw() {
                for action in draw_actions {
                    all_draw_actions.push((action, container.clone()));
                }
            }
        }

        for (action, container) in all_draw_actions {
            match action {
                DrawAction::CursorTo(x, y, cursor_style) => {
                    self.draw_cursor(x, y, cursor_style, &container);
                }
                DrawAction::AskRedraw(redraw) => match redraw {
                    Redraw::All => {
                        self.stdout
                            .execute(MoveTo(0, 0))
                            .unwrap()
                            .execute(cursor::Hide)
                            .unwrap();

                        for line_num in 0..container.get_height() {
                            self.stdout
                                .execute(MoveTo(
                                    container.left as u16,
                                    (line_num + container.top) as u16,
                                ))
                                .unwrap();
                            self.erase_line(container.get_width());
                        }
                    }
                    Redraw::Line(y, line) => {
                        self.stdout
                            .execute(MoveTo(container.left as u16, (y + container.top) as u16))
                            .unwrap();

                        self.draw_line(line, container.get_width());
                    }
                    Redraw::Range(_, _) => todo!(),
                    Redraw::Cursor => {
                        todo!()
                    }
                },
            }
        }
    }

    fn trigger_resize(&mut self, width: u16, height: u16) {
        execute!(self.stdout, Clear(ClearType::All)).unwrap();

        for idx in 0..self.modules.len() {
            let container = self.containers.get_module(idx).unwrap();

            let m = self.modules[idx].as_mut();
            m.on_resize(
                container.top,
                container.right,
                container.bottom,
                container.left,
            );
        }
    }

    fn handle_outcoming_events(&mut self, events: Vec<OutcomingConsoleEvent>) -> bool {
        for event in events {
            match event {
                OutcomingConsoleEvent::Quit => {
                    execute!(self.stdout, MoveTo(0, 0), Clear(ClearType::All)).unwrap();
                    self.before_quit();
                    std::process::exit(0);
                }
                OutcomingConsoleEvent::EnableProxy => {
                    self.proxy_enabled = true;
                }
                OutcomingConsoleEvent::DisableProxy => {
                    self.proxy_enabled = false;
                }
                OutcomingConsoleEvent::FocusMe(idx) => {
                    self.change_focus(idx);
                }
                OutcomingConsoleEvent::UnfocusMe => {
                    self.focus_stack.pop();
                }
                OutcomingConsoleEvent::Interrupt => {
                    return false;
                }
                _ => (),
            }
        }

        true
    }
}

impl ClientEvent for ConsoleClient {
    fn load(&mut self) {
        enable_raw_mode().unwrap();

        let (w, h) = terminal::size().unwrap();
        self.containers
            .setup_bsp(BSP_EXPOENT, w as u32, (h - 2) as u32);

        execute!(self.stdout, EnterAlternateScreen, Clear(ClearType::All)).unwrap();
    }

    fn update(&mut self) {
        let event = crossterm::event::read();

        match event {
            Ok(Event::Key(key)) => {
                if key.kind == KeyEventKind::Release {
                    return;
                }

                if self.proxy_enabled {
                    let proxy_outcoming_envents =
                        self.trigger_proxy(IncomingConsoleEvent::Key(key));
                    if !self.handle_outcoming_events(proxy_outcoming_envents) {
                        return;
                    }
                }

                // TODO: handle outcoming events
                let outcoming_events = self.trigger_events(IncomingConsoleEvent::Key(key));
                self.handle_outcoming_events(outcoming_events);
            }
            Ok(Event::Resize(w, h)) => {
                self.trigger_resize(w, h);
            }
            _ => (),
        }
    }

    fn draw(&mut self) {
        self.trigger_drawing();
    }

    fn before_quit(&mut self) {
        disable_raw_mode().unwrap();
        execute!(self.stdout, LeaveAlternateScreen).unwrap();
    }

    fn handle_file(&mut self, path: String) {
        self.trigger_events(IncomingConsoleEvent::File(PathBuf::from(path)));
    }
}

impl ClientModular for ConsoleClient {
    fn attach_module(&mut self, mut module: Box<dyn Module>) {
        let module_idx = self.modules.len();
        self.containers.push_module(module_idx, vec![]).unwrap();

        module.on_load();
        self.modules.push(module);
        self.focus_stack.push(module_idx as u32);

        // TODO: resize all modules
        let (w, h) = terminal::size().unwrap();
        self.trigger_resize(w, h);
    }

    fn change_focus(&mut self, idx: u32) {
        self.focus_stack.push(idx);
    }
}

impl Drop for ConsoleClient {
    fn drop(&mut self) {
        disable_raw_mode().unwrap()
    }
}
