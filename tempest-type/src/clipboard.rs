use arboard::Clipboard;

pub fn copy_text(text: &str) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new().map_err(|e| anyhow::anyhow!("Failed to access clipboard: {}", e))?;
    clipboard.set_text(text.to_owned()).map_err(|e| anyhow::anyhow!("Failed to set clipboard text: {}", e))?;
    Ok(())
}
