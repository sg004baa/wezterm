//! Persists the display used by the most recently resized window.
//!
//! Manual Windows verification:
//! 1. Enable `remember_window_display`, move a window to a secondary display,
//!    resize it, and exit WezTerm.
//! 2. Start WezTerm and verify that its window opens on that display.
//! 3. Disconnect that display and verify that WezTerm opens on the primary
//!    display instead.

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct WindowDisplayState {
    display: String,
}

lazy_static! {
    static ref LAST_SAVED_DISPLAY: Mutex<Option<String>> = Mutex::new(load_display());
}

fn state_file_name() -> PathBuf {
    config::DATA_DIR.join("window-display-state.json")
}

fn load_from(path: &Path) -> anyhow::Result<WindowDisplayState> {
    let file = std::fs::File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

pub fn load_display() -> Option<String> {
    load_from(&state_file_name())
        .map(|state| state.display)
        .map_err(|err| {
            log::debug!("Unable to load saved window display: {err:#}");
        })
        .ok()
}

pub fn save_display_if_changed(display: &str) {
    let mut last_saved = LAST_SAVED_DISPLAY.lock().unwrap();
    if last_saved.as_deref() == Some(display) {
        return;
    }

    let state = WindowDisplayState {
        display: display.to_string(),
    };
    let result = std::fs::create_dir_all(&*config::DATA_DIR)
        .map_err(anyhow::Error::from)
        .and_then(|()| serde_json::to_vec(&state).map_err(anyhow::Error::from))
        .and_then(|json| std::fs::write(state_file_name(), json).map_err(anyhow::Error::from));

    match result {
        Ok(()) => {
            last_saved.replace(display.to_string());
        }
        Err(err) => {
            log::warn!("Unable to save window display: {err:#}");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let state = WindowDisplayState {
            display: r"\\.\DISPLAY2".to_string(),
        };

        let json = serde_json::to_string(&state).unwrap();
        let decoded: WindowDisplayState = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, state);
    }
}
