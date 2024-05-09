use std::{
    error::Error,
    io::{stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo},
    event::{poll, read, Event, KeyCode},
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{self, size, Clear, ClearType},
    QueueableCommand,
};
use icon::{preview, query};

use crate::cli::Cli;

pub fn browse(args: &Cli) -> Result<(), Box<dyn Error>> {
    let (cols, rows) = size()?;

    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All)).unwrap();

    let mut previously_selected_index: Option<u16> = None;
    let mut selected_index: u16 = 0;

    let mut query_results = query(&args.query, &args.prefix, false)?;

    query_results.truncate(100);

    if !query_results.is_empty() {
        let mut row = 1;
        let mut col = 2;

        for r in query_results.iter() {
            stdout.queue(MoveTo(col as u16, row))?;
            preview(&r)?;

            if col + 18 > cols {
                col = 2;
                row += 4;
            } else {
                col += 8;
            };
        }
    }

    let mut quit = false;

    let mut selected: Option<String> = None;

    while !quit {
        if poll(Duration::from_millis(500)).unwrap() {
            match read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Enter => {
                        selected = Some(query_results[selected_index as usize].clone());
                        quit = true;
                    }
                    KeyCode::Up => {
                        if selected_index + 1 > cols / 8 {
                            previously_selected_index = Some(selected_index);
                            selected_index -= cols / 8;
                        }
                    }
                    KeyCode::Down => {
                        if selected_index < query_results.len() as u16 - cols / 8 {
                            previously_selected_index = Some(selected_index);
                            selected_index += cols / 8;
                        }
                    }
                    KeyCode::Left => {
                        if selected_index > 0 {
                            previously_selected_index = Some(selected_index);
                            selected_index -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if selected_index < query_results.len() as u16 - 1 {
                            previously_selected_index = Some(selected_index);
                            selected_index += 1;
                        }
                    }
                    KeyCode::Char(c) => {
                        if c == 'q' {
                            quit = true;
                        }
                        if c == 'j' {
                            // Move down
                            if selected_index < query_results.len() as u16 - cols / 8 {
                                previously_selected_index = Some(selected_index);
                                selected_index += cols / 8;
                            }
                        }
                        if c == 'k' {
                            // Move up
                            if selected_index + 1 > cols / 8 {
                                previously_selected_index = Some(selected_index);
                                selected_index -= cols / 8;
                            }
                        }
                        if c == 'h' {
                            // Move left
                            if selected_index > 0 {
                                previously_selected_index = Some(selected_index);
                                selected_index -= 1;
                            }
                        }
                        if c == 'l' {
                            // Move right
                            if selected_index < query_results.len() as u16 - 1 {
                                previously_selected_index = Some(selected_index);
                                selected_index += 1;
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let mut col = 2;
        let mut row = 1;

        let num_results = query_results.iter().len() as u16;

        for i in 0..num_results {
            let i = i as u16;

            if let Some(psi) = previously_selected_index {
                if i == psi {
                    stdout.queue(MoveTo(col as u16, row))?;
                    stdout.queue(Clear(ClearType::UntilNewLine))?;
                    stdout.queue(MoveTo(col as u16, row + 1))?;
                    stdout.queue(Clear(ClearType::UntilNewLine))?;
                    stdout.queue(MoveTo(col as u16, row + 2))?;
                    stdout.queue(Clear(ClearType::UntilNewLine))?;
                    stdout.flush().unwrap();
                }
            }

            if i == selected_index {
                stdout.queue(MoveTo(col as u16, row))?;
                stdout.queue(SetForegroundColor(Color::White))?;
                stdout.queue(SetBackgroundColor(Color::Blue))?;
                stdout.queue(Print(format!("      ")))?;
                stdout.queue(MoveTo(col as u16, row + 1))?;
                stdout.queue(Print(format!("      ")))?;
                stdout.queue(MoveTo(col as u16, row + 2))?;
                stdout.queue(Print(format!("      ")))?;
                stdout.queue(SetForegroundColor(Color::White))?;
                stdout.queue(SetBackgroundColor(Color::Black))?;
            }

            if col + 18 > cols {
                col = 2;
                row += 4;
                (col, row)
            } else {
                col += 8;
                (col, row)
            };
        }

        stdout.queue(MoveTo(1, rows - 1))?;
        stdout.queue(Print(query_results[selected_index as usize].clone()))?;

        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(33));
    }

    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(Clear(ClearType::All))?;

    terminal::disable_raw_mode()?;

    if let Some(selected) = selected {
        println!("{}", selected);
    }

    Ok(())
}
