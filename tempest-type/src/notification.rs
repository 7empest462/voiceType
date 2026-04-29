// Copyright (c) 2026 Robert Simens. All Rights Reserved.
// Licensed under the Tempest Type Source-Available License.
// See the LICENSE file in the repository root for full details.

use notify_rust::Notification;

pub fn show_notification(summary: &str, body: &str) {
    let _ = Notification::new()
        .summary(summary)
        .body(body)
        .appname("Tempest Type")
        .show();
}
