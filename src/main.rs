use std::env;

use client::ConsoleClient;
use editor::{Editor, EditorEvent};

use crate::client::Client;

mod client;
mod editor;
mod logger;

fn main() {
    logger::init().unwrap();

    let mut editor = Editor::new();
    let mut client = ConsoleClient::new(true);

    let mut args = env::args().skip(1);
    let path_arg = args.next();

    client.load();

    if let Some(path) = path_arg {
        editor.on_load_file(path);
    }

    loop {
        if let Some(_) = client.update(&mut editor) {
            break;
        }

        client.draw(&editor);
    }
}
