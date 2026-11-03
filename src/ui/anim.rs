//! Page transition: the outgoing page accelerates through one quarter of
//! the window width, keeps driving while it fades out completely; after a
//! short pause the incoming page appears from the opposite side with the
//! reverse curve.

use gtk4::prelude::*;
use gtk4::Widget;
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::Ui;

/// Phase durations of the transition, milliseconds.
const SLIDE_MS: u128 = 300;
const FADE_MS: u128 = 200;
const WAIT_MS: u128 = 250;
const ENTER_MS: u128 = 350;
const TOTAL_MS: u128 = SLIDE_MS + FADE_MS + WAIT_MS + ENTER_MS;

/// Slide distance is one quarter of the current window width.
fn slide_distance(ui: &Rc<Ui>) -> f64 {
    let width = ui
        .window
        .borrow()
        .as_ref()
        .map(|w| w.width())
        .unwrap_or(780);
    f64::from((width / 4).max(120))
}

/// Shift a widget horizontally: `entering` pages ride the opposite margin
/// from the outgoing ones so both move in the same visual direction.
fn set_offset(w: &Widget, forward: bool, entering: bool, px: f64) {
    let px = px as i32;
    match (forward, entering) {
        (true, false) => w.set_margin_end(px),
        (true, true) => w.set_margin_start(px),
        (false, false) => w.set_margin_start(px),
        (false, true) => w.set_margin_end(px),
    }
}

/// Drive the four-phase transition with a timer tick.
pub fn animate(ui: &Rc<Ui>, old: Option<Widget>, new_page: Widget, forward: bool) {
    let slide = slide_distance(ui);
    let start = Instant::now();
    new_page.set_opacity(0.0);
    set_offset(&new_page, forward, true, slide);
    ui.back.set_sensitive(false);
    ui.next.set_sensitive(false);
    ui.apply.set_sensitive(false);
    let ui2 = Rc::clone(ui);
    let mut old_removed = old.is_none();
    gtk4::glib::timeout_add_local(Duration::from_millis(16), move || {
        let e = start.elapsed().as_millis();
        if e < SLIDE_MS {
            // Accelerating drive through one quarter of the window.
            let k = (e as f64 / SLIDE_MS as f64).powi(2);
            if let Some(old) = &old {
                old.set_opacity(1.0);
                set_offset(old, forward, false, k * slide);
            }
        } else if e < SLIDE_MS + FADE_MS {
            // The page keeps driving while it fades out completely.
            let k = (e - SLIDE_MS) as f64 / FADE_MS as f64;
            if let Some(old) = &old {
                old.set_opacity(1.0 - k);
                set_offset(old, forward, false, slide * (1.0 + k));
            }
        } else if !old_removed {
            if let Some(old) = &old {
                ui2.host.remove_overlay(old);
            }
            old_removed = true;
        }
        if e > SLIDE_MS + FADE_MS + WAIT_MS {
            // The new page arrives after the pause, decelerating in.
            let k = ((e - SLIDE_MS - FADE_MS - WAIT_MS) as f64 / ENTER_MS as f64).min(1.0);
            let ease = 1.0 - (1.0 - k).powi(2);
            new_page.set_opacity(ease);
            set_offset(&new_page, forward, true, (1.0 - ease) * slide);
        }
        if e < TOTAL_MS {
            return gtk4::glib::ControlFlow::Continue;
        }
        new_page.set_opacity(1.0);
        new_page.set_margin_start(0);
        new_page.set_margin_end(0);
        ui2.back.set_sensitive(true);
        ui2.next.set_sensitive(true);
        // Apply stays disabled until the license is accepted.
        ui2.apply.set_sensitive(ui2.license_check.is_active());
        gtk4::glib::ControlFlow::Break
    });
}
