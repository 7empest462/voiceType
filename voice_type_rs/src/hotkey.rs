use rdev::{listen, Event, EventType};
use crate::AppEvent;

pub fn start_listener(proxy: tao::event_loop::EventLoopProxy<AppEvent>, target_key: rdev::Key) {
    std::thread::spawn(move || {
        let callback = move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    if key == target_key {
                        let _ = proxy.send_event(AppEvent::StartRecording(false));
                    }
                }
                EventType::KeyRelease(key) => {
                    if key == target_key {
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
