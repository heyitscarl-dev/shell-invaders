use std::{io::{stdout, Write}, thread::sleep, time::Duration};

use crossterm::{cursor, event::{self, Event, KeyCode, KeyEventKind}, execute, style, terminal::{self, size, Clear, ClearType}};
use hecs::World;

struct Position {
    x: u16,
    y: u16,
}

struct ScreenPosition {
    x: i16,
    y: i16
}

struct Velocity {
    x: i16,
    y: i16,
}

struct Renderable {
    data: Vec<&'static str>,
}

struct Collider {
    width: u16,
    height: u16
}

fn main() -> std::io::Result<()> {
    let (width, height) = size().unwrap();

    // Preparation
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut world = World::new();

    let player = world.spawn((
        Position { x: 10, y: 25 },
        Velocity { x: 0, y: 0 },
        Renderable { data: vec!["^"] },
    ));

    world.spawn((
        ScreenPosition { x: 2, y: 2 },
        Renderable { data: vec!["Entities: $1", "FPS: $2", "TPS: $3"] }
    ));

    for x in 0 .. 5 {
        for y in 0 .. 2 {
            world.spawn((
                Position { x: x * 9, y: y * 5},
                Renderable { data: vec!["***", "***", " * "] },
                Collider { width: 3, height: 3 }
            ));
        }
    }

    'game: loop {
        let mut bullet = None;

        // Handle input 
        while event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                // Game quitting
                if let KeyCode::Char('q') = key.code {
                    break 'game;
                }

                // Player controlling
                if let Ok(vel) = world.query_one_mut::<&mut Velocity>(player) {
                    match (key.kind, key.code) {
                        ( KeyEventKind::Press | KeyEventKind::Repeat, KeyCode::Char('a') ) => vel.x = -1,
                        ( KeyEventKind::Press | KeyEventKind::Repeat, KeyCode::Char('d') ) => vel.x = 1,
                        _ => {
                            vel.x = 0
                        }
                    }
                }

                // Bullet spawning
                bullet = if let Ok(pos) = world.get::<&Position>(player) {
                    match key.code {
                        KeyCode::Char('w') => {
                            Some((
                                Position { x: pos.x, y: pos.y },
                                Velocity { x: 0, y: -1 },
                                Collider { width: 1, height: 1 },
                                Renderable { data: vec!["|"] }
                            ))
                        },
                        _ => None
                    }
                } else { None };
            }
        }

        if let Some(bullet) = bullet {
            world.spawn(bullet);
        }

        let mut to_despawn = Vec::new();
            
        // Entity movement
        for (entity, (pos, vel)) in world.query_mut::<(&mut Position, &mut Velocity)>() {
            let new_x = (pos.x as i16) + vel.x;
            let new_y = (pos.y as i16) + vel.y;

            if new_x >= 0 && (new_x as u16) < (width as u16) {
                pos.x = new_x as u16;
            }

            if new_y >= 0 && (new_y as u16) < (height as u16) {
                pos.y = new_y as u16;
            } else {
                to_despawn.push(entity);
            }
        }

        for (inner, (pos_1, col_1)) in world.query::<(&Position, &Collider)>().iter() {
            for (outer, (pos_2, col_2)) in world.query::<(&Position, &Collider)>().iter() {
                if inner == outer {
                    continue;
                }

                if pos_1.x >= pos_2.x && pos_1.x + col_1.width <= pos_2.x + col_2.width {
                    if pos_1.y >= pos_2.y && pos_1.y + col_1.height <= pos_2.y + col_2.height {
                        to_despawn.push(inner);
                        to_despawn.push(outer);
                    }
                }
            }
        }

        // Despawn entities
        for entity in to_despawn {
            let _ = world.despawn(entity);
        }

        // Render entities 
        execute!(stdout, Clear(ClearType::All))?;
        for (_, (pos, render)) in world.query::<( &Position, &Renderable )>().iter() {
            for ( row_index, row_content ) in render.data.iter().enumerate() {
                let x = pos.x;
                let y = pos.y + row_index as u16;

                execute!(
                    stdout,
                    cursor::MoveTo(x as u16, y as u16), 
                    style::Print(row_content)
                )?;
            }
        }

        // Render UI
        let count = world.len();

        for (_, (pos, render)) in world.query::<( &ScreenPosition, &Renderable )>().iter() {
            for ( row_index, row_content ) in render.data.iter().enumerate() {
                let x = pos.x;
                let y = pos.y + ( row_index as i16 );

                let row_content = row_content
                    .replace("$1", &format!("{}", count));

                execute!(
                    stdout,
                    cursor::MoveTo(x as u16, y as u16), 
                    style::Print(row_content)
                )?;
            }
        }

        stdout.flush()?;
        sleep(Duration::from_millis(1000 / 60));
    }

    // Cleanup
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
