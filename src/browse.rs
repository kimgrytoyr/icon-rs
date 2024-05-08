use std::{
    error::Error,
    io::{stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo},
    event::{poll, read, Event, KeyCode},
    terminal::{self, size, Clear, ClearType},
    QueueableCommand,
};
use icon::{preview, query};

use crate::cli::Cli;

pub fn browse(args: &Cli) -> Result<(), Box<dyn Error>> {
    let (cols, _rows) = size()?;

    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All)).unwrap();

    if args.query.is_some() {
        let query_result = query(&args.query, &args.prefix, false)?;

        let mut row = 1;
        let mut col = 2;

        for r in query_result.iter() {
            stdout.queue(MoveTo(col as u16, row))?;
            preview(&r)?;

            if col + 18 > cols {
                col = 2;
                row += 4;
                (col, row)
            } else {
                col += 8;
                (col, row)
            };
        }
    }

    let mut quit = false;

    while !quit {
        if poll(Duration::from_millis(500)).unwrap() {
            match read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Char(c) => {
                        if c == 'q' {
                            quit = true;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(33));
    }

    terminal::disable_raw_mode()?;
    Ok(())
}
