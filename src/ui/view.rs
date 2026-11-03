//! Slint UI definition for the wizard. Kept separate from the Rust glue so
//! each file stays small. The window uses a light gray background so the
//! white surfaces of the standard widgets (LineEdit, ComboBox) keep a
//! visible border, while all text carries explicit dark colors and never
//! depends on the system color scheme.

slint::slint! {
    import { VerticalBox, HorizontalBox, LineEdit, ComboBox, Button } from "std-widgets.slint";

    // Localized string record with one column per supported language.
    export struct Txt { en: string, ru: string, de: string, fr: string, es: string }

    // Translation store. Rust fills `table`; `lang` selects the active column.
    export global I18n {
        in-out property <int> lang: 0;
        in-out property <[Txt]> table;
        public pure function t(i: int) -> string {
            if lang == 1 { table[i].ru } else if lang == 2 { table[i].de }
            else if lang == 3 { table[i].fr } else if lang == 4 { table[i].es }
            else { table[i].en }
        }
    }

    export component MainWindow inherits Window {
        title: I18n.t(0);
        background: #f2f2f2; // light gray so white widget surfaces stay visible
        preferred-width: 780px;
        preferred-height: 580px;
        min-width: 680px;
        min-height: 500px;

        // Step indices: 0 welcome, 1 language, 2 timezone, 3 user, 4 device, 5 wifi, 6 proxy.
        in-out property <int> step: 0;
        property <int> last-step: 6;

        // The keyboard layout always follows the selected language.
        property <[string]> layouts: ["us", "ru", "de", "fr", "es"];
        in-out property <int> language: 0;
        in-out property <bool> language-touched: false;
        in-out property <string> keyboard: "us";
        in-out property <string> timezone;
        in-out property <[string]> language-names;
        in-out property <[string]> timezones;
        in-out property <string> user-name;
        in-out property <string> user-login;
        in-out property <string> user-password;
        in-out property <string> user-password2;
        in-out property <string> hostname;
        in-out property <string> device-password;
        in-out property <string> device-password2;
        in-out property <string> wifi-ssid;
        in-out property <string> wifi-password;
        in-out property <string> http-proxy;
        in-out property <string> https-proxy;
        in-out property <string> no-proxy;
        in-out property <string> error;
        in-out property <bool> locating: true;
        in-out property <bool> applying: false;

        callback apply();

        // Derive the keyboard layout whenever the language changes.
        changed language => { keyboard = layouts[language]; }

        // Cross-fade animation played when switching between steps.
        property <float> fade: 1;
        animate fade { duration: 200ms; easing: ease-in-out; }
        function go(s: int) {
            fade = 0;
            step = s;
            error = "";
            fade = 1;
        }

        // Validation rules for the current step.
        pure function step-ok() -> bool {
            step == 3 ? (user-name != "" && user-login != "" && user-password != "" && user-password == user-password2)
            : step == 4 ? (hostname != "" && device-password != "" && device-password == device-password2)
            : step == 5 ? (wifi-ssid == "" || wifi-password != "")
            : true
        }
        pure function fail-message() -> string {
            (step == 3 && user-password != user-password2) || (step == 4 && device-password != device-password2)
                ? I18n.t(39) : step == 5 ? I18n.t(41) : I18n.t(40)
        }
        function next() {
            if !step-ok() { error = fail-message(); } else { go(step + 1); }
        }

        Rectangle {
            opacity: root.fade;
            VerticalBox {
                padding: 36px;
                spacing: 14px;

                // Step 0: welcome.
                if root.step == 0: VerticalBox {
                    spacing: 14px;
                    vertical-stretch: 1;
                    alignment: center;
                    Text { text: I18n.t(1); color: #1b1b1b; font-size: 30px; font-weight: 700; horizontal-alignment: center; }
                    Text { text: I18n.t(2); color: #333333; font-size: 15px; wrap: word-wrap; horizontal-alignment: center; }
                    Text { text: I18n.t(3); color: #555555; font-size: 13px; wrap: word-wrap; horizontal-alignment: center; }
                    if root.locating: Text { text: I18n.t(43) + "…"; color: #777777; font-size: 13px; horizontal-alignment: center; }
                }

                // Step 1: language.
                if root.step == 1: VerticalBox {
                    spacing: 14px;
                    vertical-stretch: 1;
                    Text { text: I18n.t(8); color: #1b1b1b; font-size: 22px; font-weight: 600; }
                    Text { text: I18n.t(9); color: #555555; font-size: 14px; }
                    ComboBox {
                        model: root.language-names;
                        current-index <=> root.language;
                        width: 320px;
                        // Mark the language as user-chosen so the GPS-based
                        // auto-detection no longer overrides it.
                        selected => { root.language-touched = true; }
                    }
                }

                // Step 2: timezone (preselected from GPS).
                if root.step == 2: VerticalBox {
                    spacing: 14px;
                    vertical-stretch: 1;
                    Text { text: I18n.t(10); color: #1b1b1b; font-size: 22px; font-weight: 600; }
                    Text { text: I18n.t(11); color: #555555; font-size: 14px; }
                    ComboBox { model: root.timezones; current-value <=> root.timezone; width: 320px; }
                }

                // Step 3: user account.
                if root.step == 3: VerticalBox {
                    spacing: 10px;
                    vertical-stretch: 1;
                    Text { text: I18n.t(12); color: #1b1b1b; font-size: 22px; font-weight: 600; }
                    Text { text: I18n.t(13); color: #555555; font-size: 14px; }
                    Text { text: I18n.t(14); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.user-name; placeholder-text: I18n.t(18); width: 320px; }
                    Text { text: I18n.t(15); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.user-login; placeholder-text: I18n.t(19); width: 320px; }
                    Text { text: I18n.t(16); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.user-password; input-type: password; width: 320px; }
                    Text { text: I18n.t(17); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.user-password2; input-type: password; width: 320px; }
                }

                // Step 4: device.
                if root.step == 4: VerticalBox {
                    spacing: 10px;
                    vertical-stretch: 1;
                    Text { text: I18n.t(20); color: #1b1b1b; font-size: 22px; font-weight: 600; }
                    Text { text: I18n.t(21); color: #555555; font-size: 14px; }
                    Text { text: I18n.t(22); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.hostname; placeholder-text: I18n.t(23); width: 320px; }
                    Text { text: I18n.t(24); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.device-password; input-type: password; width: 320px; }
                    Text { text: I18n.t(25); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.device-password2; input-type: password; width: 320px; }
                }

                // Step 5: wifi (optional).
                if root.step == 5: VerticalBox {
                    spacing: 10px;
                    vertical-stretch: 1;
                    Text { text: I18n.t(26); color: #1b1b1b; font-size: 22px; font-weight: 600; }
                    Text { text: I18n.t(27); color: #555555; font-size: 14px; }
                    Text { text: I18n.t(28); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.wifi-ssid; placeholder-text: I18n.t(29); width: 320px; }
                    Text { text: I18n.t(30); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.wifi-password; input-type: password; width: 320px; }
                    Text { text: I18n.t(31); color: #777777; font-size: 12px; wrap: word-wrap; }
                }

                // Step 6: proxy (optional).
                if root.step == 6: VerticalBox {
                    spacing: 10px;
                    vertical-stretch: 1;
                    Text { text: I18n.t(32); color: #1b1b1b; font-size: 22px; font-weight: 600; }
                    Text { text: I18n.t(33); color: #555555; font-size: 14px; }
                    Text { text: I18n.t(34); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.http-proxy; placeholder-text: I18n.t(37); width: 320px; }
                    Text { text: I18n.t(35); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.https-proxy; placeholder-text: I18n.t(37); width: 320px; }
                    Text { text: I18n.t(36); color: #333333; font-size: 13px; }
                    LineEdit { text <=> root.no-proxy; placeholder-text: I18n.t(38); width: 320px; }
                }

                // Inline validation error shown above the navigation buttons.
                if root.error != "": Text {
                    text: "⚠ " + root.error;
                    color: #b3261e;
                    font-size: 13px;
                    wrap: word-wrap;
                }

                // Navigation: forward only from the welcome screen, which can
                // never be returned to.
                HorizontalBox {
                    alignment: end;
                    spacing: 8px;
                    if root.step >= 2: Button { text: I18n.t(6); clicked => { root.go(root.step - 1); } }
                    if root.step < root.last-step: Button {
                        text: root.step == 0 ? I18n.t(4) : I18n.t(5);
                        clicked => { root.next(); }
                    }
                    if root.step == root.last-step: Button {
                        text: I18n.t(7);
                        enabled: !root.applying;
                        clicked => {
                            if !root.step-ok() {
                                root.error = root.fail-message();
                            } else {
                                root.applying = true;
                                root.apply();
                            }
                        }
                    }
                }
            }
        }
    }
}
