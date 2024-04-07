use std::env;

use client::{console::ConsoleClient, ClientEvent};
use editor::{vector::CharVectorEditor, Editor, EditorEvent};

mod client;
mod editor;
mod logger;
mod utils;

fn main() {
    logger::init().unwrap();

    let mut editor: CharVectorEditor = Editor::new();
    let mut client: ConsoleClient = ConsoleClient::new(false);

    let mut args = env::args().skip(1);
    let path_arg = args.next();

    client.load(&mut editor);

    if let Some(path) = path_arg {
        editor.on_load_file(path);
    }

    loop {
        client.draw(&editor);

        if let Some(_) = client.update(&mut editor) {
            break;
        }
    }
}
