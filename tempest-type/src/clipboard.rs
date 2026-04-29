// Copyright (c) 2026 Robert Simens. All Rights Reserved.
// Licensed under the Tempest Type Source-Available License.
// See the LICENSE file in the repository root for full details.

use arboard::Clipboard;

pub fn copy_text(text: &str) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new().map_err(|e| anyhow::anyhow!("Failed to access clipboard: {}", e))?;
    clipboard.set_text(text.to_owned()).map_err(|e| anyhow::anyhow!("Failed to set clipboard text: {}", e))?;
    Ok(())
}
