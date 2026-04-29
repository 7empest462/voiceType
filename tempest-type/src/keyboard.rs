// Copyright (c) 2026 Robert Simens. All Rights Reserved.
// Licensed under the Tempest Type Source-Available License.
// See the LICENSE file in the repository root for full details.

use enigo::{Enigo, Keyboard, Settings};

pub fn type_text(text: &str) -> anyhow::Result<()> {
    // Some versions of enigo 0.6.x use Enigo::new(&Settings::default())
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("Failed to initialize enigo: {:?}", e))?;
    
    // Type the text as if the user were typing it
    enigo.text(text)
        .map_err(|e| anyhow::anyhow!("Failed to inject text: {:?}", e))?;
        
    Ok(())
}
