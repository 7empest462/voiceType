use rdev::{listen, Event, EventType, Key};
use crate::AppEvent;

pub fn start_listener(proxy: tao::event_loop::EventLoopProxy<AppEvent>) {
    std::thread::spawn(move || {
        let callback = move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    if key == Key::AltGr {
                        let _ = proxy.send_event(AppEvent::StartRecording(false));
                    }
                }
                EventType::KeyRelease(key) => {
                    if key == Key::AltGr {
                        let _ = proxy.send_event(AppEvent::StopRecording);
                    }
                }
                _ => {}
            }
        };

        if let Err(error) = listen(callback) {
            eprintln!("Hotkey listener error: {:?}", error);
        }
    });
}
