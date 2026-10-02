//! Settings drawer: Noise Suppression slider and "Start with Windows" autostart toggle.
//! Collapsible drawer opened via the Settings button in the footer.

#![cfg(windows)]

use super::gdi::*;
use super::heroicons::*;
use super::palette::*;
use super::types::*;
use super::win32::{self, Gdiplus};
use super::window::FlyoutWindow;
use std::ffi::c_void;

pub fn render_drawer_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    g_opt: Option<&Gdiplus>,
    graphics: *mut c_void,
    fonts: &FlyoutFonts,
    y: i32,
) -> i32 {
    if !flyout.settings_expanded {
        return y;
    }

    let card_w = FLYOUT_WIDTH - 28;
    let card_h = 98;
    let card_r = 14.0;
    let card_left = 14;

    if let Some(g) = g_opt {
        draw_smooth_pill(
            g,
            graphics,
            card_left as f32,
            y as f32,
            card_w as f32,
            card_h as f32,
            card_r,
            ARGB_PILL,
            ARGB_BORDER,
        );
    }

    // 1. Noise Suppression Row
    let cy = (y + 22) as f32;

    if let Some(g) = g_opt {
        draw_hero_sound(g, graphics, 36.0, cy, ARGB_TEXT_SECONDARY);
    }

    let slider_left = 56.0;
    let slider_right = (card_left + card_w - 14) as f32;
    let slider_y = y + 10;
    let slider_touch_rect = rect(
        slider_left as i32,
        slider_y,
        slider_right as i32,
        slider_y + 24,
    );
    flyout
        .button_rects
        .push((FlyoutButton::NsSlider, slider_touch_rect));

    if let Some(g) = g_opt {
        draw_ns_slider(
            g,
            graphics,
            slider_left,
            slider_right,
            cy,
            flyout.ns_strength,
        );
    }

    let pct_str = format!(
        "Noise suppression: {}%",
        (flyout.ns_strength * 100.0) as i32
    );
    draw_text(
        dc,
        fonts.body,
        COLOR_TEXT_MUTED,
        36,
        y + 38,
        card_left + card_w - 10,
        y + 54,
        &pct_str,
        0,
    );

    // 2. Subtle Divider Hairline
    if let Some(g) = g_opt {
        let div_y = (y + 58) as f32;
        draw_smooth_pill(
            g,
            graphics,
            (card_left + 14) as f32,
            div_y,
            (card_w - 28) as f32,
            1.0,
            0.0,
            ARGB_BORDER,
            0,
        );
    }

    // 3. "Start with Windows" Row
    let row_y = y + 66;
    draw_text(
        dc,
        fonts.body,
        COLOR_TEXT_PRIMARY,
        card_left + 14,
        row_y,
        card_left + card_w - 60,
        row_y + 22,
        "Start with Windows",
        DT_CENTER_V,
    );

    let start_enabled = flyout.session_manager.config().start_with_computer;
    let is_hovered = flyout.hover_btn == Some(FlyoutButton::ToggleStartWithComputer)
        || flyout.hover_btn == Some(FlyoutButton::ToggleAutostart);

    let sw_w = 38.0;
    let sw_h = 20.0;
    let sw_r = 10.0;
    let sw_x = (card_left + card_w - 14) as f32 - sw_w;
    let sw_y = (row_y + 1) as f32;

    let sw_touch = rect(
        (sw_x - 4.0) as i32,
        row_y - 2,
        (sw_x + sw_w + 4.0) as i32,
        row_y + 24,
    );
    flyout
        .button_rects
        .push((FlyoutButton::ToggleStartWithComputer, sw_touch));

    if let Some(g) = g_opt {
        let (track_bg, track_bd) = if start_enabled {
            (ARGB_MIC_ON, ARGB_MIC_ON)
        } else if is_hovered {
            (ARGB_PILL_HOVER, ARGB_BORDER_HI)
        } else {
            (ARGB_SURFACE, ARGB_BORDER)
        };

        draw_smooth_pill(
            g, graphics, sw_x, sw_y, sw_w, sw_h, sw_r, track_bg, track_bd,
        );

        let thumb_r = 7.0;
        let thumb_cy = sw_y + sw_h / 2.0;
        let thumb_cx = if start_enabled {
            sw_x + sw_w - sw_r
        } else {
            sw_x + sw_r
        };

        let thumb_color = if start_enabled || is_hovered {
            ARGB_TEXT_PRIMARY
        } else {
            ARGB_TEXT_SECONDARY
        };

        draw_smooth_circle(g, graphics, thumb_cx, thumb_cy, thumb_r, thumb_color);
    }

    y + card_h + 8
}

fn draw_ns_slider(g: &Gdiplus, graphics: *mut c_void, left: f32, right: f32, cy: f32, val: f32) {
    let track_h = 6.0;
    let r = track_h / 2.0;
    let w = right - left;
    draw_smooth_pill(
        g,
        graphics,
        left,
        cy - r,
        w,
        track_h,
        r,
        ARGB_SURFACE,
        ARGB_BORDER,
    );
    let fill_w = (w * val.clamp(0.0, 1.0)).max(r * 2.0);
    draw_smooth_pill(
        g,
        graphics,
        left,
        cy - r,
        fill_w,
        track_h,
        r,
        ARGB_MIC_ON,
        0,
    );
    let thumb_x = left + fill_w - r;
    draw_smooth_circle(g, graphics, thumb_x, cy, 7.0, ARGB_TEXT_PRIMARY);
    draw_smooth_circle(g, graphics, thumb_x, cy, 4.0, ARGB_MIC_ON);
}
