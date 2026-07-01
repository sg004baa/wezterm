use crate::ScreenRect;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Screens {
    pub main: ScreenInfo,
    pub active: ScreenInfo,
    pub by_name: HashMap<String, ScreenInfo>,
    pub virtual_rect: ScreenRect,
}

#[derive(Debug, Clone)]
pub struct ScreenInfo {
    pub name: String,
    pub rect: ScreenRect,
    pub scale: f64,
    pub max_fps: Option<usize>,
    pub effective_dpi: Option<f64>,
}

pub fn screen_name_for_rect(
    window_rect: ScreenRect,
    screens: &HashMap<String, ScreenInfo>,
) -> Option<String> {
    let center = window_rect.center();
    screens
        .values()
        .find(|screen| screen.rect.contains(center))
        .map(|screen| screen.name.clone())
}

#[cfg(test)]
mod test {
    use super::*;

    fn screen(name: &str, rect: ScreenRect) -> ScreenInfo {
        ScreenInfo {
            name: name.to_string(),
            rect,
            scale: 1.,
            max_fps: None,
            effective_dpi: None,
        }
    }

    #[test]
    fn resolves_display_containing_window_center() {
        let screens = HashMap::from([
            (
                "left".to_string(),
                screen("left", euclid::rect(-1920, 0, 1920, 1080)),
            ),
            (
                "right".to_string(),
                screen("right", euclid::rect(0, 0, 2560, 1440)),
            ),
        ]);

        assert_eq!(
            screen_name_for_rect(euclid::rect(-1000, 100, 800, 600), &screens),
            Some("left".to_string())
        );
        assert_eq!(
            screen_name_for_rect(euclid::rect(100, 100, 800, 600), &screens),
            Some("right".to_string())
        );
    }

    #[test]
    fn returns_none_when_window_center_is_outside_all_displays() {
        let screens = HashMap::from([(
            "main".to_string(),
            screen("main", euclid::rect(0, 0, 1920, 1080)),
        )]);

        assert_eq!(
            screen_name_for_rect(euclid::rect(3000, 100, 800, 600), &screens),
            None
        );
    }
}
