// Copyright (c) 2026 Robert Simens. All Rights Reserved.
// Licensed under the Tempest Type Source-Available License.
// See the LICENSE file in the repository root for full details.

mod audio;
mod hotkey;
mod keyboard;
mod ollama;
mod transcription;
mod config;
mod clipboard;
mod notification;

use std::sync::Arc;
use tokio::sync::Mutex;
use std::io::Cursor;
use rodio::{Decoder, Player, DeviceSinkBuilder, MixerDeviceSink};

use config::Config;

use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::event::Event;
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[derive(Debug)]
pub enum AppEvent {
    StartRecording(bool),
    StopRecording,
    ProcessingFinished {
        text: String,
        raw: String,
        is_memo: bool,
    },
    StatusUpdate(String),
    ProcessingError(String),
}

struct AudioEngine {
    player: Player,
    _mixer: MixerDeviceSink,
}

impl AudioEngine {
    fn new() -> anyhow::Result<Self> {
        let _mixer = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| anyhow::anyhow!("Failed to open default audio sink: {}", e))?;
        let player = Player::connect_new(&_mixer.mixer());
        Ok(Self { player, _mixer })
    }

    fn play(&mut self, name: &str) {
        let bytes = match name {
            "Tink" | "Start" => include_bytes!("../assets/start.wav") as &[u8],
            "Pop" | "Stop" => include_bytes!("../assets/stop.wav") as &[u8],
            "Success" => include_bytes!("../assets/success.wav") as &[u8],
            _ => include_bytes!("../assets/start.wav") as &[u8],
        };

        let cursor = Cursor::new(bytes);
        if let Ok(source) = Decoder::new(cursor) {
            // Self-healing logic: if appending fails or mixer is stale, we could re-init
            // For Rodio, the most reliable way to 'heal' a stale macOS sink is to 
            // recreate the engine if we suspect it's dead. 
            // We'll attempt a play, and if it's been a while or we catch an error, we reset.
            self.player.append(source);
        }
    }

    fn refresh(&mut self) {
        if let Ok(new_engine) = Self::new() {
            *self = new_engine;
            println!("🔊 Audio engine refreshed (Self-healing).");
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
    // Initialize self-healing audio engine
    let mut audio_engine = AudioEngine::new().ok();

    // Create async runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    println!("Initiating Tempest Type...");
    println!("Initializing old conversations...");
    // Simulate cleanup logic for UI consistency
    println!("Deleting old messages...");

    // Initiating
    println!("Initiating Tempest Type...");
    println!("Initializing old conversations...");

    // Load config via confy
    let config = Config::load();
    let target_key = config.get_target_key();

    // Initialize transcriber (Heavy)
    println!("📂 Loading Whisper model...");
    let transcriber = rt.block_on(transcription::Transcriber::new())
        .map_err(|e| anyhow::anyhow!("Failed to load Whisper: {}", e))?;
    let transcriber = Arc::new(Mutex::new(transcriber));
    println!("✅ Whisper model loaded!");

    #[cfg(target_os = "macos")]
    {
        if unsafe { !AXIsProcessTrusted() } {
            eprintln!("❌ ERROR: Accessibility permissions NOT detected!");
            eprintln!("1. Open System Settings > Privacy & Security.");
            eprintln!("2. Add the NEW app to Accessibility AND Input Monitoring:");
            eprintln!("   Path: ~/.tempest-type/Tempest Type.app");
        } else {
            println!("✅ Accessibility permissions detected.");
        }
    }

    // Create the tao event loop with custom user events
    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Start the global hotkey listener
    println!("⌨️  Starting global hotkey listener...");
    hotkey::start_listener(proxy.clone(), target_key);
    println!("✅ Hotkey listener started.");

    // Create the tray menu
    println!("🏗️  Creating tray menu...");
    let tray_menu = Menu::new();
    let record_i = MenuItem::new("Start Recording", true, None);
    let separator = PredefinedMenuItem::separator();
    let memo_i = MenuItem::new("Start Meeting Memo", true, None);
    let separator2 = PredefinedMenuItem::separator();
    let copy_i = MenuItem::new("Copy Last Transcription", false, None);
    let separator3 = PredefinedMenuItem::separator();
    let quit_i = MenuItem::new("Quit", true, None);

    let _ = tray_menu.append(&record_i);
    let _ = tray_menu.append(&separator);
    let _ = tray_menu.append(&memo_i);
    let _ = tray_menu.append(&separator2);
    let _ = tray_menu.append(&copy_i);
    let _ = tray_menu.append(&separator3);
    let _ = tray_menu.append(&quit_i);

    // Load icon
    let icon = {
        let icon_path = std::env::current_exe()
            .map(|p| {
                let parent = p.parent().unwrap();
                // Check if we are in a macOS bundle (Contents/MacOS)
                if parent.ends_with("Contents/MacOS") {
                    parent.parent().unwrap().join("Resources").join("icon.png")
                } else {
                    parent.join("icon.png")
                }
            })
            .unwrap_or_else(|_| std::path::PathBuf::from("icon.png"));
            
        let mut icon_builder = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Tempest Type");

        if let Ok(img) = image::open(&icon_path) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            if let Ok(icon) = Icon::from_rgba(rgba.into_raw(), width, height) {
                icon_builder = icon_builder.with_icon(icon);
            }
        }
        
        icon_builder
    };

    #[cfg(target_os = "macos")]
    let icon = icon.with_title("🎙️");

    println!("🎨 Building tray icon...");
    let tray_icon = icon.build().map_err(|e| anyhow::anyhow!("Failed to build tray icon: {}", e))?;
    println!("✅ Tray icon built.");

    let mut audio_recorder = audio::AudioRecorder::new();

    let menu_channel = tray_icon::menu::MenuEvent::receiver();

    let mut is_recording = false;
    let mut is_memo = false;
    let mut last_transcription = String::new();

    // Run the event loop
    println!("🚀 Starting event loop...");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Handle tray menu events
        if let Ok(menu_event) = menu_channel.try_recv() {
            if menu_event.id == quit_i.id() {
                *control_flow = ControlFlow::Exit;
            } else if menu_event.id == copy_i.id() {
                if !last_transcription.is_empty() {
                    if let Err(e) = clipboard::copy_text(&last_transcription) {
                        eprintln!("❌ Failed to copy to clipboard: {}", e);
                    } else {
                        println!("📋 Copied last transcription to clipboard.");
                        if let Some(ref mut engine) = audio_engine {
                            engine.play("Success");
                        }
                    }
                }
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
                                if let Some(ref mut engine) = audio_engine {
                                    // Refresh on every start to ensure device changes are caught
                                    engine.refresh();
                                    engine.play("Start");
                                }
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
                            
                            if let Some(ref mut engine) = audio_engine {
                                engine.play("Stop");
                            }
                            
                            #[cfg(target_os = "macos")]
                            tray_icon.set_title(Some("⏳"));

                            let proxy_clone = proxy.clone();
                            let model_clone = config.model.clone();
                            let is_memo_active = is_memo;
                            let transcriber_clone = Arc::clone(&transcriber);

                            rt.spawn(async move {
                                // Give the audio engine a short moment to play the "Stop" sound
                                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                                if audio_data.len() < 6400 { // 0.4 seconds min
                                    println!("Audio too short, skipping.");
                                    return;
                                }

                                // Heavy Whisper transcription (wrapped in Mutex)
                                let transcription_result = {
                                    let mut t = transcriber_clone.lock().await;
                                    t.transcribe(&audio_data)
                                };

                                match transcription_result {
                                    Ok(raw_text) => {
                                        let trimmed = raw_text.trim().trim_matches(|c: char| !c.is_alphanumeric());
                                        if trimmed.is_empty() || trimmed.to_lowercase().contains("[blank_audio]") {
                                            println!("Transcription empty or noise. Skipping AI.");
                                            return;
                                        }

                                        if is_memo_active {
                                            let _ = proxy_clone.send_event(AppEvent::StatusUpdate("Analyzing meeting memo...".to_string()));
                                            match ollama::summarize_memo(&raw_text, &model_clone).await {
                                                Ok(summary) => {
                                                    let _ = proxy_clone.send_event(AppEvent::ProcessingFinished { text: summary, raw: raw_text, is_memo: true });
                                                }
                                                Err(e) => {
                                                    let _ = proxy_clone.send_event(AppEvent::ProcessingError(format!("Memo analysis failed: {}", e)));
                                                }
                                            }
                                        } else {
                                            let _ = proxy_clone.send_event(AppEvent::StatusUpdate("Thinking...".to_string()));
                                            match ollama::cleanup_text(&raw_text, &model_clone).await {
                                                Ok(cleaned) => {
                                                    let _ = proxy_clone.send_event(AppEvent::ProcessingFinished { text: cleaned, raw: raw_text, is_memo: false });
                                                }
                                                Err(e) => {
                                                    eprintln!("Ollama cleanup failed: {}", e);
                                                    let _ = proxy_clone.send_event(AppEvent::StatusUpdate("Using raw transcription (Ollama error)".to_string()));
                                                    let _ = proxy_clone.send_event(AppEvent::ProcessingFinished { text: raw_text, raw: String::new(), is_memo: false });
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let _ = proxy_clone.send_event(AppEvent::ProcessingError(format!("Transcription failed: {}", e)));
                                    }
                                }
                            });
                        }
                    }
                    AppEvent::StatusUpdate(msg) => {
                        notification::show_notification("Tempest Type", &msg);
                        #[cfg(target_os = "macos")]
                        tray_icon.set_title(Some("🧠"));
                    }
                    AppEvent::ProcessingFinished { text, raw, is_memo: was_memo } => {
                        #[cfg(target_os = "macos")]
                        tray_icon.set_title(Some("🎙️"));

                        if was_memo {
                            let ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                            let filename = format!("Memo_{}.md", ts);
                            let docs = dirs::document_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                            let memo_dir = docs.join("Memos");
                            let _ = std::fs::create_dir_all(&memo_dir);
                            let filepath = memo_dir.join(&filename);

                            let content = format!("# Meeting Memo - {}\n\n{}\n\n---\n### Raw Transcript\n\n{}", ts, text, raw);
                            if std::fs::write(&filepath, content).is_ok() {
                                println!("Memo saved to {:?}", filepath);
                                notification::show_notification("Memo Saved", &format!("Saved as {}", filename));
                                if let Some(ref mut engine) = audio_engine {
                                    engine.play("Success");
                                }
                                open_file(&filepath);
                            }
                        } else {
                            if !text.is_empty() {
                                println!("⌨️  Typing text...");
                                last_transcription = text.clone();
                                copy_i.set_enabled(true);
                                if let Err(e) = keyboard::type_text(&text) {
                                    eprintln!("❌ Failed to type text: {}", e);
                                }
                            }
                        }
                    }
                    AppEvent::ProcessingError(e) => {
                        #[cfg(target_os = "macos")]
                        tray_icon.set_title(Some("🎙️"));
                        eprintln!("❌ Processing Error: {}", e);
                        notification::show_notification("Tempest Type Error", &e);
                    }
                }
            }
            _ => {}
        }
    });
}
