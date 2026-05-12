use crate::scripting::guiwin::GuiWin;
use crate::termwindow::box_model::*;
use crate::termwindow::floating_container::{build_container, FloatingContainerOptions};
use crate::termwindow::modal::Modal;
use crate::utilsprites::RenderMetrics;
use crate::TermWindow;
use config::keyassignment::{InputSelector, InputSelectorEntry, KeyAssignment};
use mux_lua::MuxPane;
use std::cell::{Ref, RefCell};
use wezterm_term::{KeyCode, KeyModifiers, MouseEvent};
use window::color::LinearRgba;

pub struct FloatingInputSelector {
    title: String,
    description: String,
    choices: Vec<InputSelectorEntry>,
    event_name: String,
    selected_row: RefCell<usize>,
    top_row: RefCell<usize>,
    max_rows_on_screen: RefCell<usize>,
    gui_win: GuiWin,
    mux_pane: MuxPane,
    element: RefCell<Option<Vec<ComputedElement>>>,
}

impl FloatingInputSelector {
    pub fn new(term_window: &mut TermWindow, args: InputSelector) -> anyhow::Result<Self> {
        let event_name = match *args.action {
            KeyAssignment::EmitEvent(id) => id,
            _ => anyhow::bail!(
                "ToggleFloatingOverlay with InputSelector requires the inner action \
                 to be wezterm.action_callback"
            ),
        };

        let pane = term_window
            .get_active_pane_or_overlay()
            .ok_or_else(|| anyhow::anyhow!("no active pane"))?;

        Ok(Self {
            title: args.title,
            description: args.description,
            choices: args.choices,
            event_name,
            selected_row: RefCell::new(0),
            top_row: RefCell::new(0),
            max_rows_on_screen: RefCell::new(0),
            gui_win: GuiWin::new(term_window),
            mux_pane: MuxPane(pane.pane_id()),
            element: RefCell::new(None),
        })
    }

    fn finish(&self, term_window: &mut TermWindow, entry: Option<InputSelectorEntry>) {
        term_window.cancel_modal();
        crate::overlay::selector::trampoline(
            self.event_name.clone(),
            self.gui_win.clone(),
            self.mux_pane,
            entry,
        );
    }

    fn move_up(&self) {
        let mut row = self.selected_row.borrow_mut();
        *row = row.saturating_sub(1);
        let mut top_row = self.top_row.borrow_mut();
        if *row < *top_row {
            *top_row = *row;
        }
    }

    fn move_down(&self) {
        let limit = self.choices.len().saturating_sub(1);
        let max_rows_on_screen = *self.max_rows_on_screen.borrow();
        let mut row = self.selected_row.borrow_mut();
        *row = row.saturating_add(1).min(limit);
        let mut top_row = self.top_row.borrow_mut();
        if max_rows_on_screen > 0 && *row > *top_row + max_rows_on_screen - 1 {
            *top_row = row.saturating_sub(max_rows_on_screen - 1);
        }
    }

    fn compute(&self, term_window: &mut TermWindow) -> anyhow::Result<Vec<ComputedElement>> {
        let font = term_window
            .fonts
            .command_palette_font()
            .expect("to resolve command palette font");
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());

        let bg = term_window.config.command_palette_bg_color.to_linear();
        let fg = term_window.config.command_palette_fg_color.to_linear();
        let bg_color: InheritableColor = bg.into();
        let fg_color: InheritableColor = fg.into();

        let frame_h = crate::termwindow::floating_container::resolved_frame_height_pixels(
            term_window,
        );
        let max_rows_on_screen =
            (frame_h as usize / metrics.cell_size.height as usize).saturating_sub(4);
        *self.max_rows_on_screen.borrow_mut() = max_rows_on_screen;

        let selected_row = *self.selected_row.borrow();
        let top_row = *self.top_row.borrow();

        let mut elements = Vec::new();
        if !self.title.is_empty() {
            elements.push(
                Element::new(&font, ElementContent::Text(self.title.clone()))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: LinearRgba::TRANSPARENT.into(),
                        text: fg_color.clone(),
                    })
                    .display(DisplayType::Block),
            );
        }
        if !self.description.is_empty() {
            for line in self.description.lines() {
                elements.push(
                    Element::new(&font, ElementContent::Text(line.to_string()))
                        .colors(ElementColors {
                            border: BorderColor::default(),
                            bg: LinearRgba::TRANSPARENT.into(),
                            text: fg_color.clone(),
                        })
                        .display(DisplayType::Block),
                );
            }
        }

        for (display_idx, choice) in self
            .choices
            .iter()
            .enumerate()
            .skip(top_row)
            .take(max_rows_on_screen.max(1))
        {
            let (row_bg, row_fg) = if display_idx == selected_row {
                (fg_color.clone(), bg_color.clone())
            } else {
                (LinearRgba::TRANSPARENT.into(), fg_color.clone())
            };
            elements.push(
                Element::new(&font, ElementContent::Text(choice.label.clone()))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: row_bg,
                        text: row_fg,
                    })
                    .padding(BoxDimension {
                        left: config::Dimension::Cells(0.25),
                        right: config::Dimension::Cells(0.25),
                        top: config::Dimension::Cells(0.),
                        bottom: config::Dimension::Cells(0.),
                    })
                    .min_width(Some(config::Dimension::Percent(1.)))
                    .display(DisplayType::Block),
            );
        }

        build_container(
            term_window,
            elements,
            FloatingContainerOptions {
                font: &font,
                bg_color: None,
                text_color: fg,
                border_color: None,
                width_override: None,
                max_height: None,
                zindex: 100,
            },
        )
    }
}

impl Modal for FloatingInputSelector {
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
                let idx = *self.selected_row.borrow();
                let chosen = self.choices.get(idx).cloned();
                self.finish(term_window, chosen);
                return Ok(true);
            }
            (KeyCode::UpArrow, KeyModifiers::NONE) | (KeyCode::Char('p'), KeyModifiers::CTRL) => {
                self.move_up();
            }
            (KeyCode::DownArrow, KeyModifiers::NONE) | (KeyCode::Char('n'), KeyModifiers::CTRL) => {
                self.move_down();
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
