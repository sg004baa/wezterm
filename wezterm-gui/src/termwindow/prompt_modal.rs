use crate::scripting::guiwin::GuiWin;
use crate::termwindow::box_model::*;
use crate::termwindow::floating_container::{build_container, FloatingContainerOptions};
use crate::termwindow::modal::Modal;
use crate::TermWindow;
use config::keyassignment::{KeyAssignment, PromptInputLine};
use mux_lua::MuxPane;
use std::cell::{Ref, RefCell};
use wezterm_term::{KeyCode, KeyModifiers, MouseEvent};
use window::color::LinearRgba;

pub struct FloatingPromptInputLine {
    description: String,
    prompt: String,
    event_name: String,
    input: RefCell<String>,
    gui_win: GuiWin,
    mux_pane: MuxPane,
    element: RefCell<Option<Vec<ComputedElement>>>,
}

impl FloatingPromptInputLine {
    pub fn new(term_window: &mut TermWindow, args: PromptInputLine) -> anyhow::Result<Self> {
        let event_name = match *args.action {
            KeyAssignment::EmitEvent(id) => id,
            _ => anyhow::bail!(
                "ToggleFloatingOverlay with PromptInputLine requires the inner action \
                 to be wezterm.action_callback"
            ),
        };

        let pane = term_window
            .get_active_pane_or_overlay()
            .ok_or_else(|| anyhow::anyhow!("no active pane"))?;

        Ok(Self {
            description: args.description,
            prompt: args.prompt,
            event_name,
            input: RefCell::new(args.initial_value.unwrap_or_default()),
            gui_win: GuiWin::new(term_window),
            mux_pane: MuxPane(pane.pane_id()),
            element: RefCell::new(None),
        })
    }

    fn finish(&self, term_window: &mut TermWindow, line: Option<String>) {
        term_window.cancel_modal();
        crate::overlay::prompt::trampoline(
            self.event_name.clone(),
            self.gui_win.clone(),
            self.mux_pane,
            line,
        );
    }

    fn compute(&self, term_window: &mut TermWindow) -> anyhow::Result<Vec<ComputedElement>> {
        let font = term_window
            .fonts
            .command_palette_font()
            .expect("to resolve command palette font");

        let bg = term_window.config.command_palette_bg_color.to_linear();
        let fg = term_window.config.command_palette_fg_color.to_linear();

        let mut elements = Vec::new();
        for line in self.description.lines() {
            elements.push(
                Element::new(&font, ElementContent::Text(line.to_string()))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: LinearRgba::TRANSPARENT.into(),
                        text: fg.into(),
                    })
                    .display(DisplayType::Block),
            );
        }

        let input = self.input.borrow();
        elements.push(
            Element::new(
                &font,
                ElementContent::Text(format!("{}{}_", self.prompt, input)),
            )
            .colors(ElementColors {
                border: BorderColor::default(),
                bg: LinearRgba::TRANSPARENT.into(),
                text: fg.into(),
            })
            .display(DisplayType::Block),
        );

        build_container(
            term_window,
            elements,
            FloatingContainerOptions {
                font: &font,
                bg_color: bg,
                text_color: fg,
                border_color: bg,
                width_override: None,
                max_height: None,
                zindex: 100,
            },
        )
    }
}

impl Modal for FloatingPromptInputLine {
    fn mouse_event(&self, _event: MouseEvent, _term_window: &mut TermWindow) -> anyhow::Result<()> {
        Ok(())
    }

    fn key_down(
        &self,
        key: KeyCode,
        mods: KeyModifiers,
        term_window: &mut TermWindow,
    ) -> anyhow::Result<bool> {
        match (key, mods) {
            (KeyCode::Escape, KeyModifiers::NONE) | (KeyCode::Char('g'), KeyModifiers::CTRL) => {
                self.finish(term_window, None);
                return Ok(true);
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let line = self.input.borrow().clone();
                self.finish(term_window, Some(line));
                return Ok(true);
            }
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                self.input.borrow_mut().push(c);
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                self.input.borrow_mut().pop();
            }
            (KeyCode::Char('u'), KeyModifiers::CTRL) => {
                self.input.borrow_mut().clear();
            }
            _ => return Ok(false),
        }
        term_window.invalidate_modal();
        Ok(true)
    }

    fn computed_element(
        &self,
        term_window: &mut TermWindow,
    ) -> anyhow::Result<Ref<'_, [ComputedElement]>> {
        if self.element.borrow().is_none() {
            let element = self.compute(term_window)?;
            self.element.borrow_mut().replace(element);
        }
        Ok(Ref::map(self.element.borrow(), |v| {
            v.as_ref().unwrap().as_slice()
        }))
    }

    fn reconfigure(&self, _term_window: &mut TermWindow) {
        self.element.borrow_mut().take();
    }
}
