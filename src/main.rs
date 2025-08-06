use std::{io::{stdout, Write}, process::exit, thread::sleep, time::Duration};

use crossterm::{cursor::{self, MoveTo}, event::{self, Event, KeyCode}, execute, style, terminal::{self, Clear, ClearType}, QueueableCommand};
use hecs::{With, World};

struct Position {
    x: u16,
    y: u16
}

struct Renderable {
    glyph: char
}

fn main() -> std::io::Result<()> {
    // Preparation
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut world = World::new();

    world.spawn((
        Position { x: 10, y: 10 },
        Renderable { glyph: '@' }
    ));

    'game: loop {
        // Handle input 
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                for (_, pos) in world.query_mut::<&mut Position>() {
                    match key.code {
                        KeyCode::Left => {
                            if pos.x > 0 {
                                pos.x -= 1;
                            }
                        },
                        KeyCode::Right => {
                            pos.x += 1;
                        },
                        KeyCode::Char('q') => break 'game,
                        _ => {}
                    }
                }
            }
        }

        // Render entities 
        for (_, (pos, render)) in world.query::<( &Position, &Renderable )>().iter() {
            execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(pos.x, pos.y), style::Print(render.glyph))?;
        }

        stdout.flush()?;
    }

    // Cleanup
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
