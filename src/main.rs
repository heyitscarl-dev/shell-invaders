use std::{io::{stdout, Write}, thread::sleep, time::{Duration, Instant}};

use crossterm::{cursor, event::{self, Event, KeyCode, KeyEventKind}, execute, style, terminal::{self, ClearType}};
use hecs::World;

struct Position {
    x: i16,
    y: i16,
}

struct Velocity {
    x: i16,
    y: i16,
}

struct Renderable {
    glyph: char,
}

fn main() -> std::io::Result<()> {
    // Preparation
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut world = World::new();

    let player = world.spawn((
        Position { x: 10, y: 100 },
        Renderable { glyph: '@' },
    ));

    'game: loop {
        // Handle input 
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                // Game quitting
                if let KeyCode::Char('q') = key.code {
                    break 'game;
                }

                // Player controlling
                if let Ok(vel) = world.query_one_mut::<&mut Velocity>(player) {
                    match (key.kind, key.code) {
                        ( KeyEventKind::Press | KeyEventKind::Repeat, KeyCode::Left ) => vel.x = 1,
                        ( KeyEventKind::Press | KeyEventKind::Repeat, KeyCode::Right ) => vel.x = -1,
                        ( KeyEventKind::Release, KeyCode::Left | KeyCode::Right ) => vel.x = 0,
                        _ => {}
                    }
                }

                // Bullet spawning
                let bullet = if let Ok(pos) = world.get::<&Position>(player) {
                    match key.code {
                        KeyCode::Char(' ') => {
                            Some((
                                Position { x: pos.x, y: pos.y },
                                Velocity { x: 0, y: -1 },
                                Renderable { glyph: '*' }
                            ))
                        },
                        _ => None
                    }
                } else { None };

                if let Some(bullet) = bullet {
                    world.spawn(bullet);
                }
            }
        }

        // Entity movement
        for (_, (pos, vel)) in world.query_mut::<(&mut Position, &mut Velocity)>() {
            let new_x = pos.x + vel.x;
            let new_y = pos.y + vel.y;

            if new_x >= 0 {
                pos.x = new_x;
            }

            if new_y >= 0 {
                pos.y = new_y;
            }
        }

        // Render entities 
        for (_, (pos, render)) in world.query::<( &Position, &Renderable )>().iter() {
            execute!(
                stdout,
                terminal::Clear(ClearType::All),
                cursor::MoveTo(pos.x as u16, pos.y as u16), 
                style::Print(render.glyph)
            )?;
        }

        stdout.flush()?;
        sleep(Duration::from_millis(1000 / 60));
    }

    // Cleanup
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
