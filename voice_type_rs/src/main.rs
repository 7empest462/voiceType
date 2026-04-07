mod audio;
mod hotkey;
mod keyboard;
mod ollama;
mod transcription;
mod config;

use config::Config;

use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::event::Event;
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};

#[derive(Debug)]
pub enum AppEvent {
    StartRecording(bool), // is_memo flag
    StopRecording,
}

fn play_sound(_name: &str) {
    #[cfg(target_os = "macos")]
    {
        let sound_path = match _name {
            "Tink" => "/System/Library/Sounds/Tink.aiff",
            "Pop" => "/System/Library/Sounds/Pop.aiff",
            _ => "/System/Library/Sounds/Glass.aiff",
        };
        let _ = std::process::Command::new("afplay")
            .arg(sound_path)
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        // Try common Linux sound players and system sounds
        let sound_path = "/usr/share/sounds/freedesktop/stereo/message-new-instant.oga";
        let players = ["pw-play", "paplay", "aplay"];
        for player in players {
            if std::process::Command::new(player)
                .arg(sound_path)
                .spawn()
                .is_ok()
            {
                break;
            }
        }
    }
}

fn open_file(path: &std::path::Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

fn main() -> anyhow::Result<()> {
    // Create async runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Initialize config
    let config = Config::load();
    let target_key = config.get_target_key();

    // Initialize transcriber (downloads model if missing)
    println!("Loading Whisper model...");
    let mut transcriber = rt.block_on(transcription::Transcriber::new())?;
    println!("Model loaded!");

    // Create the tao event loop with custom user events
    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Start the global hotkey listener
    hotkey::start_listener(proxy.clone(), target_key);

    // Create the tray menu
    let tray_menu = Menu::new();
    let record_i = MenuItem::new("Start Recording", true, None);
    let separator = PredefinedMenuItem::separator();
    let memo_i = MenuItem::new("Start Meeting Memo", true, None);
    let separator2 = PredefinedMenuItem::separator();
    let quit_i = MenuItem::new("Quit", true, None);

    let _ = tray_menu.append_items(&[&record_i, &separator, &memo_i, &separator2, &quit_i]);

    // Create a simple blank icon
    let icon_rgba = vec![0, 0, 0, 0];
    let icon = Icon::from_rgba(icon_rgba, 1, 1).unwrap();

    let mut audio_recorder = audio::AudioRecorder::new();

    let _tray_icon = {
        let builder = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Voice Type")
            .with_icon(icon);
            
        #[cfg(target_os = "macos")]
        let builder = builder.with_title("🎙️");
        
        builder.build().unwrap()
    };

    let menu_channel = tray_icon::menu::MenuEvent::receiver();

    let mut is_recording = false;
    let mut is_memo = false;

    // Run the event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Handle tray menu events
        if let Ok(menu_event) = menu_channel.try_recv() {
            if menu_event.id == quit_i.id() {
                *control_flow = ControlFlow::Exit;
            } else if menu_event.id == record_i.id() {
                if is_recording {
                    let _ = proxy.send_event(AppEvent::StopRecording);
                } else {
                    let _ = proxy.send_event(AppEvent::StartRecording(false));
                }
            } else if menu_event.id == memo_i.id() {
                if is_recording {
                    let _ = proxy.send_event(AppEvent::StopRecording);
                } else {
                    let _ = proxy.send_event(AppEvent::StartRecording(true));
                }
            }
        }

        // Handle custom application events
        match event {
            Event::UserEvent(app_event) => {
                match app_event {
                    AppEvent::StartRecording(memo) => {
                        if !is_recording {
                            is_recording = true;
                            is_memo = memo;
                            
                            if is_memo {
                                #[cfg(target_os = "macos")]
                                tray_icon.set_title(Some("📝"));
                                memo_i.set_text("Stop Meeting Memo");
                                println!("📝 Starting Meeting Memo...");
                            } else {
                                #[cfg(target_os = "macos")]
                                tray_icon.set_title(Some("🔴"));
                                record_i.set_text("Stop Recording");
                                println!("🎤 Starting recording...");
                            }
                            
                            if let Err(e) = audio_recorder.start_recording() {
                                eprintln!("Failed to start recording: {}", e);
                            } else {
                                play_sound("Tink");
                            }
                        }
                    }
                    AppEvent::StopRecording => {
                        if is_recording {
                            is_recording = false;
                            #[cfg(target_os = "macos")]
                            tray_icon.set_title(Some("🎙️"));
                            record_i.set_text("Start Recording");
                            memo_i.set_text("Start Meeting Memo");
                            
                            let audio_data = audio_recorder.stop_recording();
                            println!("⏹️ Stopped recording. Captured {} samples.", audio_data.len());
                            
                            play_sound("Pop");
                            
                            #[cfg(target_os = "macos")]
                            tray_icon.set_title(Some("⏳"));
                            if audio_data.len() > 16000 { // at least 1 second
                                match transcriber.transcribe(&audio_data) {
                                    Ok(text) => {
                                        if is_memo {
                                            println!("Raw Memo Transcription: {}", text);
                                            #[cfg(target_os = "macos")]
                                            tray_icon.set_title(Some("🧠"));
                                            match rt.block_on(ollama::summarize_memo(&text, &config.model)) {
                                                Ok(summary) => {
                                                    println!("Memo Summary: {}", summary);
                                                    
                                                    // Save to file
                                                    let ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                                                    let filename = format!("Memo_{}.md", ts);
                                                    let memo_dir = dirs::document_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join("Memos");
                                                    let _ = std::fs::create_dir_all(&memo_dir);
                                                    let filepath = memo_dir.join(&filename);
                                                    
                                                    let content = format!("# Meeting Memo - {}\n\n{}\n\n---\n### Raw Transcript\n\n{}", ts, summary, text);
                                                    if std::fs::write(&filepath, content).is_ok() {
                                                        println!("Memo saved to {:?}", filepath);
                                                        open_file(&filepath);
                                                    }
                                                }
                                                Err(e) => eprintln!("Ollama memo failed: {}", e),
                                            }
                                        } else {
                                            println!("Raw Transcription: {}", text);
                                            match rt.block_on(ollama::cleanup_text(&text, &config.model)) {
                                                Ok(cleaned) => {
                                                    println!("Cleaned Text: {}", cleaned);
                                                    if let Err(e) = keyboard::type_text(&cleaned) {
                                                        eprintln!("Failed to type text: {}", e);
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Ollama cleanup failed: {}", e);
                                                    if let Err(e) = keyboard::type_text(&text) {
                                                        eprintln!("Failed to type raw text: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => eprintln!("Transcription error: {}", e),
                                }
                            } else {
                                println!("Audio too short.");
                            }
                            #[cfg(target_os = "macos")]
                            tray_icon.set_title(Some("🎙️"));
                        }
                    }
                }
            }
            _ => {}
        }
    });
}
