# Rust Text Editor (RTE)

RTE aims build a easy and intuitive interface for experimenting and exploring the crafting of a text editor

Firstly it was build aiming to be a Vim-like editor, using the console as interface. RTE has 3 modes (like vim): NORMAL, INSERT, VISUAL (_not implemented_).

## How to Install

Prerequisites:
- rust 1.77 or above
- cargo 1.77 or above
- git (optional)

_it probably works on older cargo versions, but I haven't tested_

After installing rust and cargo (more on: [rust-lang](https://www.rust-lang.org/pt-BR/learn/get-started)), you need to clone this repository (you can also download via github):

```
git clone https://github.com/Cicolas/rust-text-editor --depth=0
```

After downloading the source you need to open the `rust-text-editor/` folder:

```
cd rust-text-editor/
```

Now you have to compile and run the editor, for that cargo has a very useful command (`cargo run`):

When running the editor you need to provide an input file path, _all paths are relative_, via command line argument, as follow:

```
cargo run teste.txt
```

## How to Use

Before using the editor you need to understand how does Vim-like editors works, firstly you need to understand how the 3 modes works:
- NORMAL (represented by a steady-block cursor)
- INSERT (represented by a blinking-line cursor)
- VISUAL (represented by a steady-underscore cursor)

### Normal mode

Normal mode is the default mode, you cannot type in this mode, here all your keys will be interpreted as commands, as follow:

| key                  | command                                 |
|----------------------|-----------------------------------------|
| h / left / Backspace | Move cursor left                        |
| j / down / Enter     | Move cursor down                        |
| k / up               | Move cursor up                          |
| l / right            | Move cursor right                       |
| q / Esc              | Quit                                    |
| i                    | Enter insert mode  at cursor position   |
| I                    | Enter insert mode at line start         |
| a                    | Enter insert mode after cursor position |
| A                    | Enter insert mode at line end           |
| s                    | Save current file                       |
| PageDown             | Move view down                          |
| PageUp               | Move view up                            |

### Insert mode

Insert mode is used for editing the file, here all keys would be used to write in the file, the only exceptions are:

| key       | command                   |
|-----------|---------------------------|
| left      | Move cursor left          |
| down      | Move cursor down          |
| up        | Move cursor up            |
| right     | Move cursor right         |
| Esc       | Change to Normal mode     |
| Enter     | Insert Line Break         |
| Backspace | Delete the left character |
| Delete    | Delete current char       |

### Visual mode

_Not implemented_

## Disclaimer

This project was made only for studying purpouses, it is far from optimized or even good structured.

The development of RTE is paused, I know that is far from perfect but I think it has already achieved its objective.
