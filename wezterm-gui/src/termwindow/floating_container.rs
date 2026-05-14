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
    pub zindex: i8,
}

/// Pixel height available to the caller for laying out items inside the frame.
/// Equals the resolved frame height (cfg.height, falling back to 80% of the
/// terminal cell area) minus the configured margin/padding/border so callers
/// can compute `max_rows_on_screen` without leaving dead space at the bottom.
///
/// Why margin is counted twice: `build_container` applies the same
/// `cfg.padding` to both `.margin(..)` and `.padding(..)` on the outer element.
pub fn resolved_inner_content_pixels(term_window: &TermWindow) -> f32 {
    let cfg = &term_window.config.floating_overlay;
    let cell_h = term_window.render_metrics.cell_size.height as f32;
    let avail = term_window.terminal_size.rows as f32 * cell_h;
    let ctx = DimensionContext {
        dpi: term_window.dimensions.dpi as f32,
        pixel_max: avail,
        pixel_cell: cell_h,
    };
    let frame_h = cfg
        .height
        .map(|d| d.evaluate_as_pixels(ctx))
        .unwrap_or(avail * 0.8);
    let pad_top = cfg.padding.top.evaluate_as_pixels(ctx);
    let pad_bottom = cfg.padding.bottom.evaluate_as_pixels(ctx);
    let border_top = cfg.border.top_height.evaluate_as_pixels(ctx);
    let border_bottom = cfg.border.bottom_height.evaluate_as_pixels(ctx);
    (frame_h - 2. * (pad_top + pad_bottom) - border_top - border_bottom).max(0.)
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
    let top_origin_y = top_bar_height + padding_top + border.top.get() as f32;

    let dimensions = term_window.dimensions;
    let size = term_window.terminal_size;
    let cell_w = term_window.render_metrics.cell_size.width as f32;
    let cell_h = term_window.render_metrics.cell_size.height as f32;
    let avail_pixel_width = size.cols as f32 * cell_w;
    let avail_pixel_height = size.rows as f32 * cell_h;

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

    let mut bg_color = cfg
        .bg_color
        .map(|c| c.to_linear())
        .or(opts.bg_color)
        .unwrap_or_else(|| term_window.config.command_palette_bg_color.to_linear());
    let mut border_color = cfg
        .border
        .top_color
        .map(|c| c.to_linear())
        .or(opts.border_color)
        .unwrap_or(bg_color);
    bg_color.3 *= cfg.opacity;
    border_color.3 *= cfg.opacity;

    // When the parent window itself is alpha-blended (window_background_opacity
    // or a window_background image), drawing the floating frame with its own
    // alpha < 1.0 lets the desktop bleed through. Stamp an opaque backdrop
    // matching the frame's outer shape so the frame composites over solid
    // terminal bg instead of the OS-level transparency.
    let window_is_transparent = !term_window.window_background.is_empty()
        || term_window.config.window_background_opacity != 1.0;
    let mut backdrop_color = term_window.palette().background.to_linear();
    backdrop_color.3 = 1.0;

    let mut element = Element::new(opts.font, ElementContent::Children(inner_elements))
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
        pixel_max: avail_pixel_height,
        pixel_cell: cell_h,
    };
    let frame_height = cfg
        .height
        .map(|d| d.evaluate_as_pixels(height_ctx))
        .unwrap_or(avail_pixel_height * 0.8);
    let top_pixel_y = top_origin_y + ((avail_pixel_height - frame_height) / 2.).max(0.);

    // `min_height` constrains the inner content_rect, not the outer bounds.
    // To make the visible frame equal `frame_height`, request the inner size
    // and let padding/margin/border be added back on top by `compute_rects`.
    let pad_top = cfg.padding.top.evaluate_as_pixels(height_ctx);
    let pad_bottom = cfg.padding.bottom.evaluate_as_pixels(height_ctx);
    let border_top = cfg.border.top_height.evaluate_as_pixels(height_ctx);
    let border_bottom = cfg.border.bottom_height.evaluate_as_pixels(height_ctx);
    let inner_height =
        (frame_height - 2. * (pad_top + pad_bottom) - border_top - border_bottom).max(0.);
    element = element.min_height(Some(Dimension::Pixels(inner_height)));

    let bounds = euclid::rect(
        padding_left + x_adjust,
        top_pixel_y,
        desired_pixel_width,
        frame_height,
    );

    let mut results = Vec::with_capacity(2);

    if window_is_transparent {
        let backdrop = Element::new(opts.font, ElementContent::Children(vec![]))
            .colors(ElementColors {
                border: BorderColor::new(backdrop_color),
                bg: backdrop_color.into(),
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
            .min_width(Some(Dimension::Pixels(desired_pixel_width)))
            .min_height(Some(Dimension::Pixels(inner_height)));

        let backdrop_computed = term_window.compute_element(
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
                bounds,
                metrics: &metrics,
                gl_state: term_window.render_state.as_ref().unwrap(),
                zindex: opts.zindex,
            },
            &backdrop,
        )?;
        results.push(backdrop_computed);
    }

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
            bounds,
            metrics: &metrics,
            gl_state: term_window.render_state.as_ref().unwrap(),
            zindex: opts.zindex,
        },
        &element,
    )?;
    results.push(computed);

    Ok(results)
}
