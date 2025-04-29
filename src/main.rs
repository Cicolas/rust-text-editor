use std::env;

use client::{console::ConsoleClient, ClientEvent, ClientModular};
use module::{command::CommandModule, editor::{vector::CharVectorEditor, Editor}};

mod client;
mod logger;
mod utils;
mod module;

fn main() {
    logger::init().unwrap();

    // let mut bsp = ContainerLayout::new();
    // bsp.setup_bsp(BSP_EXPOENT, 16, 16);
    // bsp.push_module(0, vec![]).unwrap();
    // bsp.push_module(1, vec![]).unwrap();
    // bsp.push_module(2, vec![]).unwrap();
    // bsp.push_module(3, vec![]).unwrap();

    // bsp.remove_module(2);

    // let mut i = 0;
    // for LayoutNode {container, left, right, module_id, has_child } in bsp.layout_tree {
    //     println!("{} = {:?} ({:?}) -> {:?}, {:?}", i, container, module_id, left, right);
    //     i += 1;
    // }
    
    // return;
    let editor: CharVectorEditor = Editor::new();
    let command = CommandModule::new();
    let mut client = ConsoleClient::new();

    let mut args = env::args().skip(1);
    let path_arg = args.next();

    client.load();
    client.attach_module(Box::new(editor));
    // client.attach_module(Box::new(command));
    
    if let Some(path) = path_arg {
        client.handle_file(path);
    }

    loop {
        client.draw();

        if let Some(_) = client.update() {
            client.before_quit();
            break;
        }
    }
}
