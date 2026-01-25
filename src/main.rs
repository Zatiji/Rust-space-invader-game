use std::error::Error;
use std::fs;

use rusty_audio::Audio;



fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();

    load_audio_from_folder(&mut audio, "src/sounds");

    audio.play("startup");
    
    // wait that the audio threads finish
    audio.wait();
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
