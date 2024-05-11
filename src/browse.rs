use std::{
    collections::HashMap,
    error::Error,
    io::{stdout, Stdout, Write},
    time::Duration,
};

use crate::{
    config::read_config_file,
    files::{get_collection, get_icon_xml, preview, query},
};
use arboard::Clipboard;
use chrono::{DateTime, TimeDelta, Utc};
use crossterm::{
    cursor::{self, MoveTo},
    event::{poll, read, Event, KeyCode},
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{self, size, Clear, ClearType},
    QueueableCommand,
};
use resvg::usvg::fontdb::Database;

use crate::{cli::Cli, enums::IconCollection};

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Message {
    message: String,
    color: Color,
    delete_at: DateTime<Utc>,
}

fn do_move<'a>(
    direction: Direction,
    selected_index: &'a mut u16,
    previously_selected_index: &mut Option<u16>,
    items: u16,
    cols: &u16,
) -> () {
    let items_per_row = (cols - 4) / 8;

    match direction {
        Direction::Up => {
            if *selected_index + 1 > items_per_row {
                *previously_selected_index = Some(*selected_index);
                *selected_index -= items_per_row;
            }
        }
        Direction::Down => {
            if *selected_index < items - items_per_row && items >= *selected_index + items_per_row {
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

fn render_query<'a>(
    stdout: &mut Stdout,
    query_results: &mut Vec<String>,
    selected_index: &'a mut u16,
    previously_selected_index: &mut Option<u16>,
    collections_cache: &mut HashMap<String, IconCollection>,
    fontdb: &mut Database,
) -> Result<(), Box<dyn Error>> {
    if query_results.is_empty() {
        *selected_index = 0;
        *previously_selected_index = None;
        stdout.queue(Clear(ClearType::All))?;

        Ok(())
    } else {
        let (cols, rows) = size()?;
        let icons_per_row = (cols as usize - 4) / 8;
        let max_icons = (((rows as usize) - 6) / 4) * icons_per_row;

        query_results.truncate(max_icons);

        *selected_index = 0;
        *previously_selected_index = None;
        stdout.queue(Clear(ClearType::All))?;

        let mut row = 1;
        let mut col = 2;

        for (i, r) in query_results.iter().enumerate() {
            stdout.queue(MoveTo(col as u16, row))?;
            preview(&r, collections_cache, fontdb)?;

            let i = i as u16;
            if let Some(psi) = previously_selected_index {
                if i == *psi {
                    stdout.queue(MoveTo(col as u16, row))?;
                    stdout.queue(Clear(ClearType::UntilNewLine))?;
                    stdout.queue(MoveTo(col as u16, row + 1))?;
                    stdout.queue(Clear(ClearType::UntilNewLine))?;
                    stdout.queue(MoveTo(col as u16, row + 2))?;
                    stdout.queue(Clear(ClearType::UntilNewLine))?;
                    stdout.flush().unwrap();
                }
            }

            if i == *selected_index {
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

            stdout.queue(MoveTo(1, rows - 1))?;
            stdout.queue(Clear(ClearType::CurrentLine))?;
            stdout.queue(Print(query_results[*selected_index as usize].clone()))?;

            if col + 18 > cols {
                col = 2;
                row += 4;
            } else {
                col += 8;
            };
        }

        Ok(())
    }
}

fn parse_original_search_string(args: &Cli) -> Result<String, Box<dyn Error>> {
    let orig_query = if let Some(q) = &args.query {
        q.to_owned()
    } else {
        String::new()
    };

    let orig_prefix = if let Some(p) = &args.prefix {
        p.to_owned()
    } else {
        String::new()
    };

    if orig_prefix.is_empty() {
        Ok(orig_query)
    } else {
        Ok(format!("{}:{}", orig_prefix, orig_query))
    }
}

fn parse_search_string(
    search_string: &str,
) -> Result<(Option<String>, Option<String>), Box<dyn Error>> {
    if search_string.contains(":") {
        let (p, q) = search_string.split_once(":").unwrap();

        Ok((Some(p.to_string()), Some(q.to_string())))
    } else {
        Ok((None, Some(search_string.to_string())))
    }
}

pub fn browse(
    args: &Cli,
    collections_cache: &mut HashMap<String, IconCollection>,
    fontdb: &mut Database,
) -> Result<(), Box<dyn Error>> {
    let mut clipboard = Clipboard::new()?;

    // State START
    let mut quit = false;
    let mut selected: Option<String> = None;
    let mut search_mode = false;
    let mut search_string = parse_original_search_string(&args)?;
    let mut previously_selected_index: Option<u16> = None;
    let mut selected_index: u16 = 0;
    let mut messages = Vec::<Message>::new();
    // State END

    let mut query_results = query(&args.query, &args.prefix)?;

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();

    stdout.queue(cursor::Hide)?;
    stdout.queue(Clear(ClearType::All)).unwrap();

    render_query(
        &mut stdout,
        &mut query_results,
        &mut selected_index,
        &mut previously_selected_index,
        collections_cache,
        fontdb,
    )?;

    while !quit {
        let (cols, rows) = size()?;

        if poll(Duration::from_millis(500)).unwrap() {
            match read().unwrap() {
                Event::Resize(_cols, _rows) => {
                    let (p, q) = parse_search_string(&search_string)?;
                    query_results = query(&q, &p)?;

                    render_query(
                        &mut stdout,
                        &mut query_results,
                        &mut selected_index,
                        &mut previously_selected_index,
                        collections_cache,
                        fontdb,
                    )?;
                }
                Event::Key(event) => match event.code {
                    KeyCode::Backspace => {
                        if search_mode {
                            search_string.pop();
                        }
                    }
                    KeyCode::Enter if !search_mode => {
                        selected = Some(query_results[selected_index as usize].clone());
                        quit = true;
                    }
                    KeyCode::Enter if search_mode => {
                        search_mode = false;

                        let (p, q) = parse_search_string(&search_string)?;
                        query_results = query(&q, &p)?;

                        if query_results.len() == 0 {
                            messages.push(Message {
                                message: "No icons matching search string.".to_string(),
                                color: Color::Yellow,
                                delete_at: chrono::Utc::now()
                                    .checked_add_signed(TimeDelta::seconds(2))
                                    .unwrap(),
                            });
                            search_mode = true;
                        }

                        render_query(
                            &mut stdout,
                            &mut query_results,
                            &mut selected_index,
                            &mut previously_selected_index,
                            collections_cache,
                            fontdb,
                        )?;
                    }
                    KeyCode::Esc if search_mode => {
                        search_mode = false;
                    }
                    KeyCode::Esc if !search_mode => {
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
                    KeyCode::Char(c) if search_mode => {
                        search_string.push(c);
                    }
                    KeyCode::Char('q') => {
                        quit = true;
                    }
                    KeyCode::Char('s') => {
                        search_mode = true;
                    }
                    KeyCode::Char('c') => {
                        let id = query_results[selected_index as usize].clone();
                        clipboard.set_text(id.clone())?;

                        messages.push(Message {
                            message: format!("Copied '{}' to clipboard!", id),
                            color: Color::Green,
                            delete_at: chrono::Utc::now()
                                .checked_add_signed(TimeDelta::seconds(2))
                                .unwrap(),
                        });
                    }
                    KeyCode::Char('C') => {
                        let id = query_results[selected_index as usize].clone();

                        let config = read_config_file()?;

                        match config.custom_output {
                            Some(output) if !output.is_empty() => {
                                clipboard.set_text(output.replace("{icon}", &id))?;
                                messages.push(Message {
                                    message: "Custom output copied to clipboard.".to_string(),
                                    color: Color::Green,
                                    delete_at: chrono::Utc::now()
                                        .checked_add_signed(TimeDelta::seconds(2))
                                        .unwrap(),
                                });
                            }
                            _ => {
                                messages.push(Message {
                                    message: "No custom output defined.".to_string(),
                                    color: Color::Yellow,
                                    delete_at: chrono::Utc::now()
                                        .checked_add_signed(TimeDelta::seconds(2))
                                        .unwrap(),
                                });
                            }
                        }
                    }
                    KeyCode::Char('g') => {
                        let current = query_results[selected_index as usize].clone();

                        let (collection_id, _) = current.split_once(":").unwrap();
                        search_string = format!("{}:", collection_id);
                        let (p, _) = parse_search_string(search_string.as_str())?;
                        query_results = query(&None, &p)?;

                        messages.push(Message {
                            message: format!("Showing collection '{}'", collection_id),
                            color: Color::Blue,
                            delete_at: chrono::Utc::now()
                                .checked_add_signed(TimeDelta::seconds(2))
                                .unwrap(),
                        });
                        render_query(
                            &mut stdout,
                            &mut query_results,
                            &mut selected_index,
                            &mut previously_selected_index,
                            collections_cache,
                            fontdb,
                        )?;
                    }
                    KeyCode::Char('j') => {
                        // Move down
                        do_move(
                            Direction::Down,
                            &mut selected_index,
                            &mut previously_selected_index,
                            query_results.len() as u16,
                            &cols,
                        );
                    }
                    KeyCode::Char('k') => {
                        // Move up
                        do_move(
                            Direction::Up,
                            &mut selected_index,
                            &mut previously_selected_index,
                            query_results.len() as u16,
                            &cols,
                        );
                    }
                    KeyCode::Char('h') => {
                        // Move left
                        do_move(
                            Direction::Left,
                            &mut selected_index,
                            &mut previously_selected_index,
                            query_results.len() as u16,
                            &cols,
                        );
                    }
                    KeyCode::Char('l') => {
                        // Move right
                        do_move(
                            Direction::Right,
                            &mut selected_index,
                            &mut previously_selected_index,
                            query_results.len() as u16,
                            &cols,
                        );
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // TODO: Improve this section.
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
            stdout.queue(MoveTo(1, rows - 5))?;
            stdout.queue(Clear(ClearType::CurrentLine))?;
            stdout.queue(Print(format!("Index: {}", selected_index)))?;

            stdout.queue(MoveTo(1, rows - 4))?;
            stdout.queue(Clear(ClearType::CurrentLine))?;
            stdout.queue(Print(format!("Grid width: {}", cols)))?;

            stdout.queue(MoveTo(1, rows - 3))?;
            stdout.queue(Clear(ClearType::CurrentLine))?;
            stdout.queue(Print(format!("Per row: {}", (cols - 4) / 8)))?;
        }

        stdout.queue(MoveTo(1, rows - 2))?;
        stdout.queue(Clear(ClearType::CurrentLine))?;
        if !messages.is_empty() {
            match messages.last() {
                Some(last_message) => {
                    stdout.queue(SetForegroundColor(last_message.color))?;
                    stdout.queue(Print(&last_message.message))?;
                    stdout.queue(SetForegroundColor(Color::Reset))?
                }
                None => stdout.queue(Print(""))?,
            };
        };

        stdout.queue(MoveTo(1, rows - 1))?;
        stdout.queue(Clear(ClearType::CurrentLine))?;

        if search_mode {
            stdout.queue(Print(format!("Enter search: {}", search_string)))?;
        } else if !query_results.is_empty() {
            stdout.queue(Print(query_results[selected_index as usize].clone()))?;
        }

        stdout.flush().unwrap();

        let mut message_indexes_to_delete = Vec::new();
        for (i, message) in messages.iter().enumerate() {
            let utc_now = Utc::now();

            if message.delete_at < utc_now {
                message_indexes_to_delete.push(i);
            }
        }

        for message_index_to_delete in message_indexes_to_delete.iter() {
            messages.remove(*message_index_to_delete);
        }

        std::thread::sleep(std::time::Duration::from_millis(33));
    }

    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(Clear(ClearType::All))?;

    terminal::disable_raw_mode()?;

    if let Some(selected) = selected {
        println!("{}", selected);
        println!("");
        print!("  ");
        preview(&selected, collections_cache, fontdb)?;
        println!("");

        match &selected.split_once(":") {
            Some((collection_id, _icon_id)) => {
                let collection = get_collection(collection_id)?;
                print!("License: {}", collection.info.license.title);
                if let Some(license_url) = collection.info.license.url {
                    println!(": {}", license_url);
                } else {
                    println!("");
                }
            }
            None => {}
        }

        let config = read_config_file()?;

        if let Some(output) = config.custom_output {
            if !output.is_empty() {
                println!("");
                println!("{}", output.replace("{icon}", &selected));
            }
        }

        if args.output_svg {
            let (width, height, body) = get_icon_xml(&selected, collections_cache)?;

            let header = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96" color="white" viewBox="0 0 {} {}">"#,
                width, height
            );
            let footer = r#"</svg>"#;

            let body = body.replace("stroke=\"#000\"", "stroke=\"#fff\"");
            println!("");
            println!("{}{}{}", header, body, footer);
        }

        println!("");
    }

    stdout.queue(cursor::Show)?;

    Ok(())
}
