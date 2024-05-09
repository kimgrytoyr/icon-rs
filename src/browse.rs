use std::{
    collections::HashMap,
    error::Error,
    io::{stdout, Write},
    time::Duration,
};

use crate::files::{preview, query};
use crossterm::{
    cursor::{Hide, MoveTo},
    event::{poll, read, Event, KeyCode},
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{self, size, Clear, ClearType},
    QueueableCommand,
};

use crate::{cli::Cli, enums::IconCollection};

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn do_move<'a>(
    direction: Direction,
    selected_index: &'a mut u16,
    previously_selected_index: &mut Option<u16>,
    items: u16,
    cols: &u16,
) -> () {
    let items_per_row = (cols - 2) / 8;

    match direction {
        Direction::Up => {
            if *selected_index + 1 > items_per_row {
                *previously_selected_index = Some(*selected_index);
                *selected_index -= items_per_row;
            }
        }
        Direction::Down => {
            if *selected_index < items - items_per_row {
                *previously_selected_index = Some(*selected_index);
                *selected_index += items_per_row;
            }
        }
        Direction::Left => {
            if *selected_index > 0 {
                *previously_selected_index = Some(*selected_index);
                *selected_index -= 1;
            }
        }
        Direction::Right => {
            if *selected_index < items - 1 {
                *previously_selected_index = Some(*selected_index);
                *selected_index += 1;
            }
        }
    }
}

pub fn browse(
    args: &Cli,
    collections_cache: &mut HashMap<String, IconCollection>,
) -> Result<(), Box<dyn Error>> {
    let (cols, rows) = size()?;

    terminal::enable_raw_mode()?;

    let mut stdout = stdout();

    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All)).unwrap();

    let mut previously_selected_index: Option<u16> = None;
    let mut selected_index: u16 = 0;

    let mut query_results = query(&args.query, &args.prefix, false)?;

    query_results.truncate(200);

    if !query_results.is_empty() {
        let mut row = 1;
        let mut col = 2;

        for r in query_results.iter() {
            stdout.queue(MoveTo(col as u16, row))?;
            preview(&r, collections_cache)?;

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
                        do_move(
                            Direction::Up,
                            &mut selected_index,
                            &mut previously_selected_index,
                            query_results.len() as u16,
                            &cols,
                        );
                    }
                    KeyCode::Down => {
                        do_move(
                            Direction::Down,
                            &mut selected_index,
                            &mut previously_selected_index,
                            query_results.len() as u16,
                            &cols,
                        );
                    }
                    KeyCode::Left => {
                        do_move(
                            Direction::Left,
                            &mut selected_index,
                            &mut previously_selected_index,
                            query_results.len() as u16,
                            &cols,
                        );
                    }
                    KeyCode::Right => {
                        do_move(
                            Direction::Right,
                            &mut selected_index,
                            &mut previously_selected_index,
                            query_results.len() as u16,
                            &cols,
                        );
                    }
                    KeyCode::Char(c) => {
                        if c == 'q' {
                            quit = true;
                        }
                        if c == 'j' {
                            // Move down
                            do_move(
                                Direction::Down,
                                &mut selected_index,
                                &mut previously_selected_index,
                                query_results.len() as u16,
                                &cols,
                            );
                        }
                        if c == 'k' {
                            // Move up
                            do_move(
                                Direction::Up,
                                &mut selected_index,
                                &mut previously_selected_index,
                                query_results.len() as u16,
                                &cols,
                            );
                        }
                        if c == 'h' {
                            // Move left
                            do_move(
                                Direction::Left,
                                &mut selected_index,
                                &mut previously_selected_index,
                                query_results.len() as u16,
                                &cols,
                            );
                        }
                        if c == 'l' {
                            // Move right
                            do_move(
                                Direction::Right,
                                &mut selected_index,
                                &mut previously_selected_index,
                                query_results.len() as u16,
                                &cols,
                            );
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
                stdout.queue(SetForegroundColor(Color::Reset))?;
                stdout.queue(SetBackgroundColor(Color::Reset))?;
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

        if args.verbose {
            stdout.queue(MoveTo(1, rows - 4))?;
            stdout.queue(Clear(ClearType::CurrentLine))?;
            stdout.queue(Print(format!("Index: {}", selected_index)))?;

            stdout.queue(MoveTo(1, rows - 3))?;
            stdout.queue(Clear(ClearType::CurrentLine))?;
            stdout.queue(Print(format!("Grid width: {}", cols)))?;

            stdout.queue(MoveTo(1, rows - 2))?;
            stdout.queue(Clear(ClearType::CurrentLine))?;
            stdout.queue(Print(format!("Per row: {}", (cols - 4) / 8)))?;
        }

        stdout.queue(MoveTo(1, rows - 1))?;
        stdout.queue(Clear(ClearType::CurrentLine))?;
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
