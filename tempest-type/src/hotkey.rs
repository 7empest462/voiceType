// Copyright (c) 2026 Robert Simens. All Rights Reserved.
// Licensed under the Tempest Type Source-Available License.
// See the LICENSE file in the repository root for full details.

use rdev::{listen, Event, EventType};
use crate::AppEvent;

pub fn start_listener(proxy: tao::event_loop::EventLoopProxy<AppEvent>, target_key: rdev::Key) {
    println!("⌨️  Starting global hotkey listener (Target: {:?})...", target_key);
    std::thread::spawn(move || {
        let callback = move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    // Log all keys to debug if permissions are working
                    // println!("DEBUG: Key pressed: {:?}", key); 
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
            eprintln!("❌ Hotkey listener error: {:?}. This is usually a macOS Permission issue (Accessibility/Input Monitoring).", error);
        }
    });
}
