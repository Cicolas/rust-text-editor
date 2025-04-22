use std::env;

use client::{console::ConsoleClient, ClientEvent, ClientModular};
use log::info;
use module::editor::{vector::CharVectorEditor, Editor};

mod client;
mod logger;
mod utils;
mod module;

fn main() {
    logger::init().unwrap();

    let editor: CharVectorEditor = Editor::new();
    let mut client = ConsoleClient::new();

    let mut args = env::args().skip(1);
    let path_arg = args.next();

    client.attach_module(Box::new(editor));
    client.load();
    
    if let Some(path) = path_arg {
        client.handle_file(path);
    }

    let mut i = 0;

    loop {
        client.draw();

        if let Some(_) = client.update() {
            client.before_quit();
            break;
        }
        info!("{}", {i += 1; i});
    }
}
