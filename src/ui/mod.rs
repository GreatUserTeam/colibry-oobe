mod anim;
mod auth;
mod icons;
mod pages;

use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Orientation, Widget};
use std::error::Error;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::{config::Config, crypto, geo, i18n};
use pages::Ui;

/// Styles applied to the wizard widgets. Sizes target a phone-like
/// narrow window: large fonts, tall fields and buttons, muted colors.
const CSS: &str = "
.ttl { font-size: 28px; font-weight: 700; }
.big { font-size: 38px; font-weight: 700; }
.dim { color: #555555; font-size: 15px; }
.faint { color: #777777; font-size: 13px; }
.err { color: #b3261e; font-size: 15px; }
entry { min-height: 52px; font-size: 16px; }
dropdown { min-height: 52px; font-size: 16px; }
button { min-height: 48px; font-size: 15px; }
entry.fail { border-color: #b3261e; border-width: 2px; }
";

/// Last step index. Steps: 0 welcome, 1 language, 2 timezone, 3 ColibryID,
/// 4 wifi, 5 license.
const LAST_STEP: usize = 5;

/// Initialize GTK (picks Wayland or X11 automatically through GDK), force
/// the light Adwaita theme and open the wizard window.
pub fn run() -> Result<(), Box<dyn Error>> {
    gtk4::init()?;
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_theme_name(Some("Adwaita"));
    }
    let app = Application::builder()
        .application_id("com.colibry.OOBE")
        .build();
    app.connect_activate(build_ui);
    app.run();
    Ok(())
}

/// Construct the phone-like window, wire navigation and start the checks.
fn build_ui(app: &Application) {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(CSS);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let ui = Ui::new(initial_language());
    let window = ApplicationWindow::builder()
        .application(app)
        .title(i18n::t(ui.lang.get(), 0))
        .default_width(420)
        .default_height(800)
        .build();
    *ui.window.borrow_mut() = Some(window.clone());

    // The overlay grabs all vertical space so the button bar is pushed to
    // the bottom of the window.
    ui.host.set_vexpand(true);
    ui.host.set_hexpand(true);
    ui.host.set_halign(Align::Fill);
    ui.host.set_valign(Align::Fill);

    let nav = GtkBox::new(Orientation::Horizontal, 8);
    nav.set_margin_start(24);
    nav.set_margin_end(24);
    nav.set_margin_bottom(20);
    ui.back.add_css_class("flat");
    ui.error.set_hexpand(true);
    ui.error.set_halign(Align::Start);
    ui.error.set_wrap(true);
    ui.error.add_css_class("err");
    nav.append(&ui.back);
    nav.append(&ui.error);
    nav.append(&ui.next);
    nav.append(&ui.apply);

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.append(&ui.host);
    root.append(&nav);
    window.set_child(Some(&root));

    {
        let back = ui.back.clone();
        let ui2 = Rc::clone(&ui);
        back.connect_clicked(move |_| {
            if ui2.step.get() == 0 {
                return;
            }
            go_to(&ui2, ui2.step.get() - 1, false);
        });
    }
    {
        let next = ui.next.clone();
        let ui2 = Rc::clone(&ui);
        next.connect_clicked(move |_| {
            if let Some(msg) = validation_error(&ui2) {
                ui2.error.set_label(msg);
                return;
            }
            ui2.error.set_label("");
            if ui2.step.get() == 3 {
                verify_and_advance(&ui2);
                return;
            }
            advance(&ui2);
        });
    }
    {
        let apply_btn = ui.apply.clone();
        let ui2 = Rc::clone(&ui);
        apply_btn.connect_clicked(move |_| {
            apply(&ui2);
        });
    }
    {
        let check = ui.license_check.clone();
        let apply_btn = ui.apply.clone();
        check.connect_toggled(move |c| apply_btn.set_sensitive(c.is_active()));
    }
    {
        // Editing either field clears the red error frame immediately.
        let ui2 = Rc::clone(&ui);
        let email_entry = ui.email.borrow();
        email_entry.connect_changed(move |_| mark_fields(&ui2, false));
    }
    {
        let ui2 = Rc::clone(&ui);
        let pass_entry = ui.pass.borrow();
        pass_entry.connect_changed(move |_| mark_fields(&ui2, false));
    }

    refresh_nav(&ui);
    window.present();
    detect_location(&ui);
}

/// Paint or clear the red error frame on the ColibryID input fields.
fn mark_fields(ui: &Rc<Ui>, fail: bool) {
    for field in [&ui.email, &ui.pass] {
        let entry = field.borrow();
        if fail {
            entry.add_css_class("fail");
        } else {
            entry.remove_css_class("fail");
        }
    }
}

/// Move forward from the current step, or finish when it is the last one.
fn advance(ui: &Rc<Ui>) {
    if ui.step.get() >= LAST_STEP {
        apply(ui);
        return;
    }
    go_to(ui, ui.step.get() + 1, true);
}

/// Verify the ColibryID credentials against the colibryid.guteam.net
/// server, showing a spinner while waiting. Wrong credentials paint the
/// input fields red; a network problem gets its own message; both keep the
/// user on the page.
fn verify_and_advance(ui: &Rc<Ui>) {
    let email = ui.email.borrow().text().to_string();
    let password = ui.pass.borrow().text().to_string();
    mark_fields(ui, false);
    ui.verify_spinner.start();
    ui.verify_spinner.set_visible(true);
    ui.verify_status.set_visible(true);
    ui.back.set_sensitive(false);
    ui.next.set_sensitive(false);
    ui.apply.set_sensitive(false);
    // The network call runs on a worker thread; only the small enum travels
    // back through the channel, the UI update stays on the main loop.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(auth::verify(&email, &password));
    });
    let rx = Arc::new(Mutex::new(rx));
    let ui2 = Rc::clone(ui);
    gtk4::glib::idle_add_local(move || {
        let done = rx.lock().unwrap().try_recv();
        let outcome = match done {
            Err(std::sync::mpsc::TryRecvError::Empty) => return gtk4::glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => auth::Outcome::Unavailable,
            Ok(outcome) => outcome,
        };
        ui2.verify_spinner.stop();
        ui2.verify_spinner.set_visible(false);
        ui2.verify_status.set_visible(false);
        ui2.back.set_sensitive(true);
        ui2.next.set_sensitive(true);
        ui2.apply.set_sensitive(ui2.license_check.is_active());
        let lang = ui2.lang.get();
        match outcome {
            auth::Outcome::Verified => advance(&ui2),
            auth::Outcome::WrongCredentials => {
                mark_fields(&ui2, true);
                ui2.error.set_label(i18n::t(lang, 50));
            }
            auth::Outcome::Unavailable => {
                ui2.error.set_label(i18n::t(lang, 51));
            }
        }
        gtk4::glib::ControlFlow::Break
    });
}

/// Show Back only from step 2 onward (the welcome screen is unreachable),
/// swap Next for Apply on the last step and keep every label translated.
fn refresh_nav(ui: &Rc<Ui>) {
    let step = ui.step.get();
    let lang = ui.lang.get();
    let key = if step == 0 { 4 } else { 5 };
    ui.back.set_visible(step >= 2);
    ui.next.set_visible(step < LAST_STEP);
    ui.apply.set_visible(step == LAST_STEP);
    ui.next_label.set_label(i18n::t(lang, key));
}

/// Switch to page `target` and run the slide-and-fade transition on it.
fn go_to(ui: &Rc<Ui>, target: usize, forward: bool) {
    let new_page: Widget = ui.pages.borrow()[target].clone().upcast();
    let old = ui.current.borrow_mut().replace(new_page.clone());
    ui.host.add_overlay(&new_page);
    ui.step.set(target);
    refresh_nav(ui);
    anim::animate(ui, old, new_page, forward);
}

/// Return the reason the current step cannot be left, if any.
fn validation_error(ui: &Rc<Ui>) -> Option<&'static str> {
    let lang = ui.lang.get();
    let email = ui.email.borrow().text();
    match ui.step.get() {
        3 if email.is_empty() || !email.contains('@') => Some(i18n::t(lang, 40)),
        3 if ui.pass.borrow().text().is_empty() => Some(i18n::t(lang, 40)),
        4 if ui.ssid.borrow().text().is_empty() => Some(i18n::t(lang, 40)),
        4 if ui.wifipass.borrow().text().is_empty() => Some(i18n::t(lang, 41)),
        5 if !ui.license_check.is_active() => Some(i18n::t(lang, 47)),
        _ => None,
    }
}

/// Hash/encrypt the collected values, save the config and close the wizard.
fn apply(ui: &Rc<Ui>) {
    match build_config(ui).and_then(|cfg| cfg.save()) {
        Ok(()) => {
            if let Some(window) = ui.window.borrow().as_ref() {
                window.close();
            }
        }
        Err(e) => ui
            .error
            .set_label(&format!("{}: {e}", i18n::t(ui.lang.get(), 42))),
    }
}

/// Collect every widget value into a Config. The login and hostname are
/// derived from the ColibryID email and the device password equals the
/// account password, so the wizard asks for the credentials only once.
fn build_config(ui: &Rc<Ui>) -> Result<Config, Box<dyn Error>> {
    let email = ui.email.borrow().text().to_string();
    let local = email.split('@').next().unwrap_or("user");
    let host_part: String = local
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let user_hash = crypto::hash_password(ui.pass.borrow().text().as_str())?;
    let device_hash = user_hash.clone();
    let (wifi_enc, wifi_salt) =
        crypto::encrypt_wifi_password(ui.wifipass.borrow().text().as_str(), &device_hash)?;
    let lang = ui.lang.get();
    Ok(Config {
        config_version: "1.0".into(),
        language: geo::LANGUAGE_CODES[lang].into(),
        // Layouts follow the selected language, with US always available.
        keyboard_layout: format!("{},us", geo::keyboard_layout(lang)),
        timezone: ui.timezone_list.borrow()[ui.tz_dd.selected() as usize].clone(),
        user_name: email.clone(),
        user_login: host_part.clone(),
        user_password_hash: user_hash,
        hostname: format!("{host_part}-computer"),
        device_password_hash: device_hash,
        wifi_ssid: ui.ssid.borrow().text().to_string(),
        wifi_password_encrypted: wifi_enc,
        wifi_key_salt: wifi_salt,
        http_proxy: String::new(),
        https_proxy: String::new(),
        no_proxy: String::new(),
        configured_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    })
}

/// Detect the GPS position on a background thread and hand the result back
/// to the GTK main loop. The timezone moves to the top of the picker and
/// the matching locale is preselected unless the user already chose a
/// language.
fn detect_location(ui: &Rc<Ui>) {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(geo::detect());
    });
    let rx = Arc::new(Mutex::new(rx));
    let ui2 = Rc::clone(ui);
    // idle_add_local is used because the closure touches Rc<Ui>, which is
    // not Send; it is registered from the main thread, which is allowed.
    gtk4::glib::idle_add_local(move || {
        let found = rx.lock().unwrap().try_recv();
        match found {
            Err(std::sync::mpsc::TryRecvError::Empty) => gtk4::glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => gtk4::glib::ControlFlow::Break,
            Ok(info) => {
                if let Some(info) = info {
                    let tz = geo::timezone_for(info.latitude, info.longitude);
                    let (lang_idx, _) = geo::locale_for_timezone(tz);
                    let mut list = geo::all_timezones();
                    list.retain(|t| t != tz);
                    list.insert(0, tz.to_string());
                    *ui2.timezone_list.borrow_mut() = list.clone();
                    let model =
                        gtk4::StringList::new(&list.iter().map(|s| s.as_str()).collect::<Vec<_>>());
                    ui2.tz_dd.set_model(Some(&model));
                    ui2.tz_dd.set_selected(0);
                    if !ui2.lang_touched.get() {
                        ui2.suppress.set(true);
                        ui2.lang_dd.set_selected(lang_idx as u32);
                        ui2.suppress.set(false);
                    }
                }
                ui2.locating.set_visible(false);
                gtk4::glib::ControlFlow::Break
            }
        }
    });
}

/// Bootstrap language from LANG until the GPS fix arrives; the geo result
/// overrides it as soon as detection completes (see detect_location).
fn initial_language() -> usize {
    let env = std::env::var("LANG")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var("LC_ALL").ok())
        .unwrap_or_default();
    let code = env.split('.').next().unwrap_or("");
    geo::LANGUAGE_CODES
        .iter()
        .position(|c| code.starts_with(c))
        .unwrap_or(0)
}
