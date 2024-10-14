use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute,
    style::{Print, ResetColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use rand::{Rng, seq::SliceRandom};
use std::{
    io::{stdout, Write},
    time::{Duration, Instant},
};

const PLANE: char = '🛩';
const GROUND: char = '▁';
const JUMP_HEIGHT: usize = 5;
const GAME_WIDTH: usize = 80;
const GAME_HEIGHT: usize = 10;
const GAME_SPEED: u64 = 50; // Lower value means faster game

const BUILDINGS: [&str; 5] = ["🏠", "🏢", "🏫", "🏛️", "🏰"];

struct Game {
    plane_y: usize,
    jumping: bool,
    obstacles: Vec<(usize, &'static str)>,
    score: u32,
    last_update: Instant,
    buffer: Vec<Vec<char>>,
    ground_offset: usize,
}

impl Game {
    fn new() -> Self {
        Game {
            plane_y: 0,
            jumping: false,
            obstacles: vec![],
            score: 0,
            last_update: Instant::now(),
            buffer: vec![vec![' '; GAME_WIDTH]; GAME_HEIGHT],
            ground_offset: 0,
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) < Duration::from_millis(GAME_SPEED) {
            return;
        }
        self.last_update = now;

        if self.jumping {
            self.plane_y = self.plane_y.saturating_add(1);
            if self.plane_y >= JUMP_HEIGHT {
                self.jumping = false;
            }
        } else if self.plane_y > 0 {
            self.plane_y = self.plane_y.saturating_sub(1);
        }

        // Move ground from right to left
        self.ground_offset = (self.ground_offset + GAME_WIDTH - 1) % GAME_WIDTH;

        // Move obstacles from right to left
        self.obstacles.iter_mut().for_each(|(x, _)| *x = x.saturating_sub(1));

        // Remove obstacles that have moved off the screen or passed the plane
        self.obstacles.retain(|&(x, _)| x > 0);

        // Spawn new obstacles on the right side
        if rand::thread_rng().gen_ratio(1, 20) && !self.obstacles.iter().any(|&(x, _)| x == GAME_WIDTH - 1) {
            let building = *BUILDINGS.choose(&mut rand::thread_rng()).unwrap();
            self.obstacles.push((GAME_WIDTH - 1, building));
        }

        self.score += 1;
    }

    fn draw(&mut self) {
        // Clear buffer
        for row in self.buffer.iter_mut() {
            for cell in row.iter_mut() {
                *cell = ' ';
            }
        }

        // Draw ground
        for x in 0..GAME_WIDTH {
            let ground_x = (x + self.ground_offset) % GAME_WIDTH;
            self.buffer[GAME_HEIGHT - 1][ground_x] = GROUND;
        }

        // Draw plane on the left side
        let plane_y = GAME_HEIGHT - 2 - self.plane_y;
        self.buffer[plane_y][2] = PLANE;

        // Draw obstacles (buildings)
        for &(x, building) in &self.obstacles {
            if x < GAME_WIDTH {
                self.buffer[GAME_HEIGHT - 2][x] = building.chars().next().unwrap();
            }
        }
    }

    fn render(&self) -> crossterm::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, Hide, MoveTo(0, 0))?;

        for (i, row) in self.buffer.iter().enumerate() {
            execute!(
                stdout,
                MoveTo(0, i as u16),
                Print(row.iter().collect::<String>())
            )?;
        }

        execute!(
            stdout,
            MoveTo(0, GAME_HEIGHT as u16),
            Print(format!("Score: {}", self.score))
        )?;
        // Add a newline character to ensure clean output
        execute!(stdout, Print("\n"))?;

        stdout.flush()?;
        Ok(())
    }

    fn is_collision(&self) -> bool {
        self.obstacles.iter().any(|&(x, _)| x == 2 && self.plane_y == 0)
    }
}

fn main() -> crossterm::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All))?;

    let mut game = Game::new();

    loop {
        if poll(Duration::from_millis(10))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Char(' ') | KeyCode::Up if !game.jumping && game.plane_y == 0 => {
                        game.jumping = true;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => break,
                    _ => {}
                }
            }
        }

        game.update();
        game.draw();
        game.render()?;

        if game.is_collision() {
            execute!(
                stdout,
                MoveTo(0, (GAME_HEIGHT + 1) as u16),
                Clear(ClearType::FromCursorDown),
                Print(format!("Game Over! Final Score: {}", game.score)),
                Print("\n\n")
            )?;
            stdout.flush()?;
            break;
        }
    }

    execute!(
        stdout,
        Clear(ClearType::FromCursorDown),
        MoveTo(0, (GAME_HEIGHT + 3) as u16),  // Move cursor below the game over message
        Show,
        ResetColor
    )?;
    disable_raw_mode()?;
    stdout.flush()?;
    Ok(())
}
