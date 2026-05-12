use crate::termwindow::box_model::*;
use crate::termwindow::render::corners::{
    BOTTOM_LEFT_ROUNDED_CORNER, BOTTOM_RIGHT_ROUNDED_CORNER, TOP_LEFT_ROUNDED_CORNER,
    TOP_RIGHT_ROUNDED_CORNER,
};
use crate::termwindow::{DimensionContext, TermWindow};
use crate::utilsprites::RenderMetrics;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct FloatingContainerOptions<'a> {
    pub font: &'a Rc<LoadedFont>,
    /// Modal's per-Modal default bg (e.g. `command_palette_bg_color`).
    /// Overridden by `floating_overlay.bg_color` when set; otherwise used as-is.
    pub bg_color: Option<LinearRgba>,
    pub text_color: LinearRgba,
    /// Modal's per-Modal default border color.
    /// Overridden by `floating_overlay.border.top_color` when set; falls back to bg.
    pub border_color: Option<LinearRgba>,
    /// Caller-supplied width override; takes precedence over
    /// `floating_overlay.width` from config.
    pub width_override: Option<Dimension>,
    /// Pixel height of the bounds rectangle handed to layout.
    /// `None` defers to `floating_overlay.height`, then to terminal rows.
    pub max_height: Option<f32>,
    pub zindex: i8,
}

pub fn build_container(
    term_window: &mut TermWindow,
    inner_elements: Vec<Element>,
    opts: FloatingContainerOptions,
) -> anyhow::Result<Vec<ComputedElement>> {
    let cfg = term_window.config.floating_overlay.clone();
    let metrics = RenderMetrics::with_font_metrics(&opts.font.metrics());

    let top_bar_height = if term_window.show_tab_bar && !term_window.config.tab_bar_at_bottom {
        term_window.tab_bar_pixel_height().unwrap()
    } else {
        0.
    };
    let (padding_left, padding_top) = term_window.padding_left_top();
    let border = term_window.get_os_border();
    let top_pixel_y = top_bar_height + padding_top + border.top.get() as f32;

    let dimensions = term_window.dimensions;
    let size = term_window.terminal_size;
    let cell_w = term_window.render_metrics.cell_size.width as f32;
    let cell_h = term_window.render_metrics.cell_size.height as f32;
    let avail_pixel_width = size.cols as f32 * cell_w;

    let width_ctx = DimensionContext {
        dpi: dimensions.dpi as f32,
        pixel_max: avail_pixel_width,
        pixel_cell: cell_w,
    };

    let desired_pixel_width = opts
        .width_override
        .or(cfg.width)
        .map(|d| d.evaluate_as_pixels(width_ctx))
        .unwrap_or_else(|| (size.cols / 3).max(120).min(size.cols) as f32 * cell_w);

    let bg_color = cfg
        .bg_color
        .map(|c| c.to_linear())
        .or(opts.bg_color)
        .unwrap_or_else(|| term_window.config.command_palette_bg_color.to_linear());
    let border_color = cfg
        .border
        .top_color
        .map(|c| c.to_linear())
        .or(opts.border_color)
        .unwrap_or(bg_color);

    let element = Element::new(opts.font, ElementContent::Children(inner_elements))
        .colors(ElementColors {
            border: BorderColor::new(border_color),
            bg: bg_color.into(),
            text: opts.text_color.into(),
        })
        .margin(BoxDimension {
            left: cfg.padding.left,
            right: cfg.padding.right,
            top: cfg.padding.top,
            bottom: cfg.padding.bottom,
        })
        .padding(BoxDimension {
            left: cfg.padding.left,
            right: cfg.padding.right,
            top: cfg.padding.top,
            bottom: cfg.padding.bottom,
        })
        .border(BoxDimension {
            left: cfg.border.left_width,
            right: cfg.border.right_width,
            top: cfg.border.top_height,
            bottom: cfg.border.bottom_height,
        })
        .border_corners(Some(Corners {
            top_left: SizedPoly {
                width: cfg.corner_radius,
                height: cfg.corner_radius,
                poly: TOP_LEFT_ROUNDED_CORNER,
            },
            top_right: SizedPoly {
                width: cfg.corner_radius,
                height: cfg.corner_radius,
                poly: TOP_RIGHT_ROUNDED_CORNER,
            },
            bottom_left: SizedPoly {
                width: cfg.corner_radius,
                height: cfg.corner_radius,
                poly: BOTTOM_LEFT_ROUNDED_CORNER,
            },
            bottom_right: SizedPoly {
                width: cfg.corner_radius,
                height: cfg.corner_radius,
                poly: BOTTOM_RIGHT_ROUNDED_CORNER,
            },
        }))
        .min_width(Some(Dimension::Pixels(desired_pixel_width)));

    let x_adjust = ((avail_pixel_width - padding_left) - desired_pixel_width) / 2.;
    let height_ctx = DimensionContext {
        dpi: dimensions.dpi as f32,
        pixel_max: size.rows as f32 * cell_h,
        pixel_cell: cell_h,
    };
    let max_height = opts
        .max_height
        .or_else(|| cfg.height.map(|d| d.evaluate_as_pixels(height_ctx)))
        .unwrap_or_else(|| size.rows as f32 * cell_h);

    let computed = term_window.compute_element(
        &LayoutContext {
            height: DimensionContext {
                dpi: dimensions.dpi as f32,
                pixel_max: dimensions.pixel_height as f32,
                pixel_cell: metrics.cell_size.height as f32,
            },
            width: DimensionContext {
                dpi: dimensions.dpi as f32,
                pixel_max: dimensions.pixel_width as f32,
                pixel_cell: metrics.cell_size.width as f32,
            },
            bounds: euclid::rect(
                padding_left + x_adjust,
                top_pixel_y,
                desired_pixel_width,
                max_height,
            ),
            metrics: &metrics,
            gl_state: term_window.render_state.as_ref().unwrap(),
            zindex: opts.zindex,
        },
        &element,
    )?;

    Ok(vec![computed])
}
