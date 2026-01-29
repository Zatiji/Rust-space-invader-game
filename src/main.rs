use std::error::Error;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{fs, io, thread};

use crossterm::event::{Event, KeyCode};
use crossterm::{ExecutableCommand, event};
use crossterm::cursor::{Hide, Show};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use rust_space_invader_game::frame::Drawable;
use rust_space_invader_game::invaders::Invaders;
use rust_space_invader_game::player::Player;
use rust_space_invader_game::{frame, render};
use rusty_audio::Audio;

use crate::frame::new_frame;



fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();

    load_audio_from_folder(&mut audio, "src/audio");

    audio.play("startup");

    // terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a seperate thread
    let (render_transiver, render_reciever) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();

        render::render(&mut stdout, &last_frame, &last_frame, true);

        loop {
            let current_frame = match render_reciever.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &current_frame, false);
            last_frame = current_frame;
        }
    });

    // game loop
    let mut player = Player::new();
    let mut instant =Instant::now();
    let mut invaders = Invaders::new();
    'gameloop: loop {
        // per_frame initialisation
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut current_frame = new_frame();

        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('a') => player.move_left(),
                    KeyCode::Right | KeyCode::Char('d') => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Char('w') | KeyCode::Enter | KeyCode::Up => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    },
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => { }
                }
            }
        }

        // Updates
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }

        if player.detect_hits(&mut invaders) {
            audio.play("explode");
        }

        // Draw & render
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut current_frame);
        }
        let _ = render_transiver.send(current_frame);
        thread::sleep(Duration::from_millis(1));

        // WIN or LOSE
        if invaders.all_killed() {
            audio.play("win");
            break 'gameloop;
        }
        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop;
        }
    }
    
    // wait that the audio threads finish
    drop(render_transiver);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn load_audio_from_folder(audio: &mut Audio, folder_path: &str) {
    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                if let Some(extension) = file_path.extension() {
                    if extension == "wav" || extension == "mp3" {
                        if let Some(file_stem) = file_path.file_stem() {
                            if let Some(file_stem_str) = file_stem.to_str() {
                                audio.add(file_stem_str, file_path.to_str().unwrap());
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("Failed to read the folder: {}", folder_path);
    }
}
