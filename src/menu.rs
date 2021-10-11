use crate::GameOfLife;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute, queue};

use std::fs;
use std::io::{self, Stdout};


pub fn game_selection(so: &mut Stdout) -> Result<GameOfLife, io::Error> {
    queue!(so, Clear(ClearType::All), cursor::MoveTo(0, 0));
    //Show options menu
    queue!(
        so,
        Print("Welcome to the Conway's Game of Life!"),
        cursor::MoveToNextLine(1)
    );
    queue!(
        so,
        Print("Where do you want to start?"),
        cursor::MoveToNextLine(1)
    );
    queue!(so, Print("'q' to quit"), cursor::MoveToNextLine(1));
    queue!(so, Print("1 : Load an example"), cursor::MoveToNextLine(1));
    queue!(so, Print("2 : Load your file"), cursor::MoveToNextLine(1));
    execute!(
        so,
        Print("3 : Generate from word"),
        cursor::MoveToNextLine(1)
    );

    match event::read().unwrap() {
        Event::Key(KeyEvent {
            code: KeyCode::Char('1'),
            modifiers: KeyModifiers::NONE,
        }) => {
            let gol_file = file_selection(so);
            return Ok(GameOfLife::init(&gol_file)?);
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('2'),
            modifiers: KeyModifiers::NONE,
        }) => {
            let path = get_input(so, "Enter file path");
            return Ok(GameOfLife::init(&path)?);
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('3'),
            modifiers: KeyModifiers::NONE,
        }) => {
            let word = get_input(so, "Enter a text to use");
            return Ok(GameOfLife::from_word(&word)?);
        }
        _ => (),
    }

    //create game
    Err(io::Error::new(io::ErrorKind::Other, "No Option Selected"))
}

fn get_input(so: &mut Stdout, instruction: &str) -> String {
    let mut res = String::new();
    queue!(so, Clear(ClearType::All), cursor::MoveTo(0, 0));
    queue!(so, Print(instruction));
    execute!(so, cursor::MoveToNextLine(1));

    loop {
        match event::read().unwrap() {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => {
                res = res[0..res.len() - 1].to_string();
                execute!(
                    so,
                    Clear(ClearType::CurrentLine),
                    cursor::MoveToColumn(0),
                    Print(&res)
                );
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
            }) => {
                res += std::str::from_utf8(&[c as u8]).unwrap();
                execute!(so, Print(c));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
            }) => {
                res += std::str::from_utf8(&[c as u8]).unwrap();
                execute!(so, Print(c));
            }
            _ => (),
        }
    }

    res
}

//Display the file selection menu when
#[allow(unused_must_use)]
fn file_selection(so: &mut Stdout) -> String {
    let base_dir = "./selection_files/";
    let mut paths = fs::read_dir(base_dir).unwrap();

    //display menu
    let mut i = 1;
    queue!(so, Clear(ClearType::All), cursor::MoveTo(0, 0));
    queue!(so, Print("Choose a file to load"));
    let mut files: Vec<String> = vec![];
    for p in &mut paths {
        let mut file_name = p.unwrap().path().display().to_string();
        file_name = file_name.replace(base_dir, "");
        let s = format!("{} : {}", i, &file_name);
        files.push(file_name);
        queue!(so, cursor::MoveToNextLine(1), Print(s));

        if i >= 9 {
            break;
        }
        i += 1;
    }
    execute!(so);

    let max = i;
    let file_keys: Vec<usize> = (1..=max).collect();

    let mut res = String::new();
    match event::read().unwrap() {
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
        }) => {
            for k in file_keys.iter() {
                if c == std::char::from_digit((*k) as u32, 10).unwrap() {
                    res = base_dir.to_string() + &files[*k - 1];
                }
            }
        }
        _ => (),
    }

    res
}

//show infos about inputs and the game
#[allow(unused_must_use)]
pub fn hud(so: &mut Stdout, c: usize, s: f32) {
    queue!(so, Print("'q' to quit; 'x' to speed up; 'c' to slow down; arrows to move; 'z' to zoom; 'u' to unzoom"), cursor::MoveToNextLine(1));
    queue!(so, Print(format!("Generation: {}; Speed: {}/s", c, s)));
}
