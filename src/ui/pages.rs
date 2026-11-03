//! Page construction and retranslation for the GTK wizard.
//!
//! Every visible string is registered with its i18n key when the widget is
//! created, so `retranslate` can refresh the whole interface in place when
//! the user switches the language. Each page shows a large centered brand
//! icon at the top, then the title, the subtitle and the content. Field
//! icons come from the system GTK theme by name; the logo, the user
//! pictogram and the license text are embedded from assets/.
use gtk4::prelude::*;
use gtk4::{
    Align, ApplicationWindow, Box as GtkBox, Button, CheckButton, DropDown, Entry,
    EntryIconPosition, Image, InputPurpose, Label, Orientation, Overlay, ScrolledWindow, Spinner,
    StringList, TextView, Widget, WrapMode,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::icons;
use crate::geo;
use crate::i18n;

/// License text embedded at compile time from assets/LICENSE.
const LICENSE: &str = include_str!("../../assets/LICENSE");

type Registry<T> = RefCell<Vec<(T, usize)>>;

/// All wizard widgets plus the label registry used by `retranslate`.
pub struct Ui {
    pub lang: Cell<usize>,
    pub step: Cell<usize>,
    pub lang_touched: Cell<bool>,
    /// Guards against reacting to programmatic dropdown selection changes.
    pub suppress: Cell<bool>,
    pub window: RefCell<Option<ApplicationWindow>>,
    pub host: Overlay,
    pub pages: RefCell<Vec<GtkBox>>,
    pub current: RefCell<Option<Widget>>,
    pub back: Button,
    pub next: Button,
    pub apply: Button,
    pub next_label: Label,
    pub error: Label,
    pub locating: Label,
    pub verify_spinner: Spinner,
    pub verify_status: Label,
    pub lang_dd: DropDown,
    pub tz_dd: DropDown,
    pub timezone_list: RefCell<Vec<String>>,
    pub email: RefCell<Entry>,
    pub pass: RefCell<Entry>,
    pub ssid: RefCell<Entry>,
    pub wifipass: RefCell<Entry>,
    pub license_check: CheckButton,
    labels: Registry<Label>,
    entries: Registry<Entry>,
}

/// Shared context passed to the page builders before the Rc<Ui> exists.
struct Ctx<'a> {
    lang: usize,
    labels: &'a Registry<Label>,
    entries: &'a Registry<Entry>,
}

/// Load an icon from the system GTK theme at the given pixel size.
fn theme_icon(name: &str, size: i32) -> Image {
    let img = Image::from_icon_name(name);
    img.set_pixel_size(size);
    img
}

/// Load a brand icon embedded from assets/ at the given pixel size.
fn brand_icon(pixbuf: &gtk4::gdk_pixbuf::Pixbuf, size: i32) -> Image {
    let img = Image::from_pixbuf(Some(pixbuf));
    img.set_pixel_size(size);
    img
}

impl Ui {
    /// Build every page, register all translatable strings, place the first
    /// page into the overlay host and connect the language dropdown:
    /// selecting a language retranslates the interface and marks the choice
    /// as manual so GPS no longer overrides it.
    pub fn new(lang: usize) -> Rc<Ui> {
        let labels = RefCell::new(Vec::new());
        let entries = RefCell::new(Vec::new());
        let ctx = Ctx {
            lang,
            labels: &labels,
            entries: &entries,
        };

        let lang_names: Vec<&str> = geo::LANGUAGE_NAMES.to_vec();
        let lang_dd = DropDown::default();
        lang_dd.set_model(Some(&StringList::new(&lang_names)));
        lang_dd.set_selected(lang as u32);
        lang_dd.set_size_request(340, 52);
        lang_dd.set_halign(Align::Center);
        let tz_list = geo::all_timezones();
        let tz_dd = DropDown::default();
        tz_dd.set_model(Some(&StringList::new(
            &tz_list.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )));
        tz_dd.set_size_request(340, 52);
        tz_dd.set_halign(Align::Center);

        let email = RefCell::new(Entry::new());
        let pass = RefCell::new(Entry::new());
        let ssid = RefCell::new(Entry::new());
        let wifipass = RefCell::new(Entry::new());
        let license_check = CheckButton::new();
        let error = Label::new(None);
        let locating = mk_label(&ctx, 43);
        locating.add_css_class("faint");
        locating.set_halign(Align::Center);
        let verify_spinner = Spinner::new();
        let verify_status = mk_label(&ctx, 44);
        verify_status.add_css_class("dim");

        let pages_vec = vec![
            welcome(&ctx, &locating),
            language(&ctx, &lang_dd),
            timezone(&ctx, &tz_dd),
            user(&ctx, &email, &pass, &verify_spinner, &verify_status),
            wifi(&ctx, &ssid, &wifipass),
            license(&ctx, &license_check),
        ];

        let (back, _) = mk_button(&ctx, 6, Some("go-previous"));
        let (next, next_label) = mk_button(&ctx, 5, Some("go-next"));
        let (apply, _) = mk_button(&ctx, 7, Some("emblem-ok"));
        let ui = Rc::new(Ui {
            lang: Cell::new(lang),
            step: Cell::new(0),
            lang_touched: Cell::new(false),
            suppress: Cell::new(false),
            window: RefCell::new(None),
            host: Overlay::new(),
            pages: RefCell::new(pages_vec),
            current: RefCell::new(None),
            back,
            next,
            apply,
            next_label,
            error,
            locating,
            verify_spinner,
            verify_status,
            lang_dd,
            tz_dd,
            timezone_list: RefCell::new(tz_list),
            email,
            pass,
            ssid,
            wifipass,
            license_check,
            labels,
            entries,
        });
        ui.verify_spinner.set_visible(false);
        ui.verify_status.set_visible(false);
        ui.apply.set_sensitive(false);
        let first: Widget = ui.pages.borrow()[0].clone().upcast();
        ui.host.add_overlay(&first);
        *ui.current.borrow_mut() = Some(first);
        let ui_ref = Rc::clone(&ui);
        ui.lang_dd.connect_selected_notify(move |dd| {
            if ui_ref.suppress.get() {
                return;
            }
            ui_ref.lang_touched.set(true);
            ui_ref.lang.set(dd.selected() as usize);
            retranslate(&ui_ref);
        });
        ui
    }
}

/// Create a label bound to the translation key `key`.
fn mk_label(ctx: &Ctx, key: usize) -> Label {
    let l = Label::new(Some(i18n::t(ctx.lang, key)));
    l.set_halign(Align::Start);
    l.set_xalign(0.0);
    ctx.labels.borrow_mut().push((l.clone(), key));
    l
}

/// Create an entry with a translated placeholder. Secret entries hide the
/// input and get an eye icon that toggles the visibility on click.
fn mk_entry(ctx: &Ctx, key: Option<usize>, secret: bool) -> Entry {
    let e = Entry::new();
    if let Some(key) = key {
        e.set_placeholder_text(Some(i18n::t(ctx.lang, key)));
        ctx.entries.borrow_mut().push((e.clone(), key));
    }
    e.set_size_request(340, 52);
    e.set_halign(Align::Center);
    if secret {
        e.set_visibility(false);
        e.set_icon_from_icon_name(EntryIconPosition::Secondary, Some("view-reveal-symbolic"));
        // gtk4-rs exposes no visibility getter, so the toggle state is
        // tracked locally and mirrored into the entry.
        let revealed = Cell::new(false);
        e.connect_icon_press(move |entry, _pos| {
            let show = !revealed.get();
            revealed.set(show);
            entry.set_visibility(show);
            let icon = if show {
                "view-conceal-symbolic"
            } else {
                "view-reveal-symbolic"
            };
            entry.set_icon_from_icon_name(EntryIconPosition::Secondary, Some(icon));
        });
    }
    e
}

/// Create a button with a themed icon and a retranslated caption.
fn mk_button(ctx: &Ctx, key: usize, icon: Option<&str>) -> (Button, Label) {
    let b = Button::new();
    let content = GtkBox::new(Orientation::Horizontal, 6);
    if let Some(name) = icon {
        let img = theme_icon(name, 20);
        img.set_valign(Align::Center);
        content.append(&img);
    }
    let l = mk_label(ctx, key);
    l.set_valign(Align::Center);
    content.append(&l);
    b.set_child(Some(&content));
    (b, l)
}

/// Wrap a page in a vertical box centered in the window.
fn page_box(spacing: i32) -> GtkBox {
    let b = GtkBox::new(Orientation::Vertical, spacing);
    b.set_halign(Align::Center);
    b.set_valign(Align::Center);
    b.set_vexpand(true);
    b.set_size_request(360, -1);
    b
}

/// Page header: large centered brand icon, title, wrapped subtitle.
fn header(ctx: &Ctx, p: &GtkBox, icon: Image, title_key: usize, subtitle_key: usize) {
    icon.set_halign(Align::Center);
    p.append(&icon);
    let t = mk_label(ctx, title_key);
    t.add_css_class("ttl");
    t.set_halign(Align::Center);
    p.append(&t);
    let s = mk_label(ctx, subtitle_key);
    s.add_css_class("dim");
    s.set_halign(Align::Center);
    s.set_wrap(true);
    p.append(&s);
}

/// Helper: a dimmed field label, then the entry with a leading icon.
fn field(ctx: &Ctx, page: &GtkBox, label_key: usize, entry: &Entry, icon: Image) {
    let l = mk_label(ctx, label_key);
    l.add_css_class("dim");
    page.append(&l);
    let h = GtkBox::new(Orientation::Horizontal, 10);
    icon.set_size_request(24, 24);
    icon.set_valign(Align::Center);
    h.append(&icon);
    h.append(entry);
    page.append(&h);
}

/// Step 0: welcome.
fn welcome(ctx: &Ctx, locating: &Label) -> GtkBox {
    let p = page_box(18);
    let logo = brand_icon(&icons::logo(), 96);
    logo.set_halign(Align::Center);
    let t = mk_label(ctx, 1);
    t.add_css_class("big");
    t.set_halign(Align::Center);
    let sub = mk_label(ctx, 2);
    sub.add_css_class("dim");
    sub.set_halign(Align::Center);
    sub.set_wrap(true);
    let note = mk_label(ctx, 3);
    note.add_css_class("faint");
    note.set_halign(Align::Center);
    note.set_wrap(true);
    p.append(&logo);
    p.append(&t);
    p.append(&sub);
    p.append(&note);
    p.append(locating);
    p
}

/// Step 1: language.
fn language(ctx: &Ctx, dd: &DropDown) -> GtkBox {
    let p = page_box(14);
    header(ctx, &p, brand_icon(&icons::logo(), 64), 8, 9);
    p.append(dd);
    p
}

/// Step 2: timezone (preselected from GPS).
fn timezone(ctx: &Ctx, dd: &DropDown) -> GtkBox {
    let p = page_box(14);
    header(ctx, &p, brand_icon(&icons::logo(), 64), 10, 11);
    p.append(dd);
    p
}

/// Step 3: ColibryID account (email and its password only).
fn user(
    ctx: &Ctx,
    email: &RefCell<Entry>,
    pass: &RefCell<Entry>,
    spinner: &Spinner,
    status: &Label,
) -> GtkBox {
    let p = page_box(12);
    header(ctx, &p, brand_icon(&icons::user(), 64), 12, 13);
    let e = mk_entry(ctx, Some(18), false);
    e.set_input_purpose(InputPurpose::Email);
    *email.borrow_mut() = e;
    *pass.borrow_mut() = mk_entry(ctx, None, true);
    field(
        ctx,
        &p,
        14,
        &email.borrow(),
        theme_icon("internet-mail", 24),
    );
    field(
        ctx,
        &p,
        16,
        &pass.borrow(),
        theme_icon("system-lock-screen", 24),
    );
    let row = GtkBox::new(Orientation::Horizontal, 10);
    row.set_halign(Align::Center);
    row.append(spinner);
    status.set_halign(Align::Center);
    row.append(status);
    p.append(&row);
    p
}

/// Step 4: wifi (always required, the page can never be skipped).
fn wifi(ctx: &Ctx, ssid: &RefCell<Entry>, wifipass: &RefCell<Entry>) -> GtkBox {
    let p = page_box(12);
    header(ctx, &p, brand_icon(&icons::logo(), 64), 26, 27);
    *ssid.borrow_mut() = mk_entry(ctx, Some(29), false);
    *wifipass.borrow_mut() = mk_entry(ctx, None, true);
    field(
        ctx,
        &p,
        28,
        &ssid.borrow(),
        theme_icon("network-wireless", 24),
    );
    field(
        ctx,
        &p,
        30,
        &wifipass.borrow(),
        theme_icon("system-lock-screen", 24),
    );
    let note = mk_label(ctx, 31);
    note.add_css_class("faint");
    note.set_wrap(true);
    p.append(&note);
    p
}

/// Step 5: license agreement, mandatory before applying the configuration.
fn license(ctx: &Ctx, check: &CheckButton) -> GtkBox {
    let p = page_box(12);
    header(ctx, &p, brand_icon(&icons::logo(), 64), 48, 49);
    let text = TextView::new();
    text.buffer().set_text(LICENSE);
    text.set_editable(false);
    text.set_cursor_visible(false);
    text.set_wrap_mode(WrapMode::Word);
    let scroll = ScrolledWindow::new();
    scroll.set_child(Some(&text));
    scroll.set_size_request(340, 300);
    p.append(&scroll);
    let row = GtkBox::new(Orientation::Horizontal, 10);
    row.set_halign(Align::Center);
    row.append(check);
    let l = mk_label(ctx, 46);
    l.set_valign(Align::Center);
    row.append(&l);
    p.append(&row);
    p
}

/// Refresh every registered string from the active language column.
pub fn retranslate(ui: &Rc<Ui>) {
    let lang = ui.lang.get();
    for (w, key) in ui.labels.borrow().iter() {
        w.set_label(i18n::t(lang, *key));
    }
    for (w, key) in ui.entries.borrow().iter() {
        w.set_placeholder_text(Some(i18n::t(lang, *key)));
    }
    let key = if ui.step.get() == 0 { 4 } else { 5 };
    ui.next_label.set_label(i18n::t(lang, key));
    if let Some(window) = ui.window.borrow().as_ref() {
        window.set_title(Some(i18n::t(lang, 0)));
    }
}
