//! Geolocation-based defaults.
//!
//! The wizard needs an initial language, region and timezone before the user
//! has entered anything. These values are derived from the device location,
//! resolved in three stages: the GeoClue2 system daemon (real GPS via
//! ModemManager backends), ModemManager directly, and finally a coarse
//! IP-based lookup. Coordinates are mapped to the nearest timezone from an
//! embedded city table, and the timezone determines the default locale.

use std::process::Command;
use std::time::Duration;

/// Detected geographic position.
pub struct GeoInfo {
    pub latitude: f64,
    pub longitude: f64,
}

/// Locale codes parallel to `LANGUAGE_NAMES`.
pub const LANGUAGE_CODES: [&str; 5] = ["en", "ru", "de", "fr", "es"];
/// Human-readable language names shown in the language picker.
pub const LANGUAGE_NAMES: [&str; 5] = ["English", "Русский", "Deutsch", "Français", "Español"];

/// Primary keyboard layout per language; "us" is always appended as a fallback.
pub fn keyboard_layout(lang: usize) -> &'static str {
    ["us", "ru", "de", "fr", "es"][lang]
}

/// Representative city coordinates per IANA timezone. The nearest entry to
/// the detected position wins; the first entry is also the default picker
/// value when no location could be obtained.
const ZONES: &[(&str, f64, f64)] = &[
    ("Europe/Moscow", 55.8, 37.6),
    ("Europe/Kaliningrad", 54.7, 21.1),
    ("Asia/Yekaterinburg", 56.8, 60.6),
    ("Asia/Novosibirsk", 55.0, 82.9),
    ("Asia/Vladivostok", 43.1, 131.9),
    ("Pacific/Midway", 28.2, -177.4),
    ("America/Anchorage", 61.2, -149.9),
    ("America/Los_Angeles", 34.1, -118.2),
    ("America/Denver", 39.7, -105.0),
    ("America/Chicago", 41.9, -87.6),
    ("America/New_York", 40.7, -74.0),
    ("America/Santiago", -33.4, -70.7),
    ("America/Sao_Paulo", -23.5, -46.6),
    ("Atlantic/Azores", 37.7, -25.7),
    ("Europe/London", 51.5, -0.1),
    ("Europe/Paris", 48.9, 2.4),
    ("Europe/Madrid", 40.4, -3.7),
    ("Europe/Berlin", 52.5, 13.4),
    ("Europe/Helsinki", 60.2, 24.9),
    ("Asia/Dubai", 25.2, 55.3),
    ("Asia/Karachi", 24.9, 67.0),
    ("Asia/Dhaka", 23.8, 90.4),
    ("Asia/Bangkok", 13.8, 100.5),
    ("Asia/Shanghai", 31.2, 121.5),
    ("Asia/Tokyo", 35.7, 139.7),
    ("Australia/Sydney", -33.9, 151.2),
    ("Pacific/Noumea", -22.3, 166.4),
    ("Pacific/Auckland", -36.9, 174.8),
    ("Pacific/Apia", -13.8, -171.8),
];

/// Timezone picker entries.
pub fn all_timezones() -> Vec<String> {
    ZONES.iter().map(|z| z.0.to_string()).collect()
}

/// Nearest timezone to the given coordinates (equirectangular distance is
/// sufficient for a coarse lookup).
pub fn timezone_for(lat: f64, lon: f64) -> &'static str {
    let cos_lat = lat.to_radians().cos().max(0.01);
    let mut best = ZONES[0].0;
    let mut best_dist = f64::MAX;
    for &(name, zlat, zlon) in ZONES {
        let d = (lat - zlat).powi(2) + (lon - zlon).powi(2) * cos_lat.powi(2);
        if d < best_dist {
            best_dist = d;
            best = name;
        }
    }
    best
}

/// Map a timezone to (language index, primary keyboard layout).
pub fn locale_for_timezone(tz: &str) -> (usize, &'static str) {
    match tz {
        "Europe/Moscow" | "Europe/Kaliningrad" | "Asia/Yekaterinburg" | "Asia/Novosibirsk"
        | "Asia/Vladivostok" => (1, "ru"),
        "Europe/Berlin" => (2, "de"),
        "Europe/Paris" => (3, "fr"),
        "Europe/Madrid" => (4, "es"),
        _ => (0, "us"),
    }
}

/// Resolve the current position using every available source.
pub fn detect() -> Option<GeoInfo> {
    detect_geoclue()
        .or_else(detect_modem)
        .or_else(detect_via_ip)
}

/// Run a command and return its stdout when it exits successfully.
fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Ask the GeoClue2 daemon for the GPS position via D-Bus (busctl).
fn detect_geoclue() -> Option<GeoInfo> {
    let out = run(
        "busctl",
        &[
            "call",
            "org.freedesktop.GeoClue2",
            "/org/freedesktop/GeoClue2/Manager",
            "org.freedesktop.GeoClue2.Manager",
            "GetClient",
        ],
    )?;
    let path = extract_path(&out)?;
    run(
        "busctl",
        &[
            "set-property",
            "org.freedesktop.GeoClue2",
            &path,
            "org.freedesktop.GeoClue2.Client",
            "DesktopId",
            "s",
            "com.colibry.OOBE",
        ],
    );
    run(
        "busctl",
        &[
            "set-property",
            "org.freedesktop.GeoClue2",
            &path,
            "org.freedesktop.GeoClue2.Client",
            "DistanceThreshold",
            "u",
            "100",
        ],
    );
    run(
        "busctl",
        &[
            "call",
            "org.freedesktop.GeoClue2",
            &path,
            "org.freedesktop.GeoClue2.Client",
            "Start",
        ],
    );
    let mut found = None;
    for _ in 0..6 {
        std::thread::sleep(Duration::from_secs(1));
        if let Some(loc) = run(
            "busctl",
            &[
                "get-property",
                "org.freedesktop.GeoClue2",
                &path,
                "org.freedesktop.GeoClue2.Client",
                "Location",
            ],
        ) {
            if let (Some(lat), Some(lon)) = (
                number_after(&loc, "Latitude"),
                number_after(&loc, "Longitude"),
            ) {
                found = Some(GeoInfo {
                    latitude: lat,
                    longitude: lon,
                });
                break;
            }
        }
    }
    run(
        "busctl",
        &[
            "call",
            "org.freedesktop.GeoClue2",
            &path,
            "org.freedesktop.GeoClue2.Client",
            "Stop",
        ],
    );
    found
}

/// Extract the D-Bus object path from a busctl reply like: o "/org/...".
fn extract_path(out: &str) -> Option<String> {
    let start = out.find("/org/")?;
    let rest = &out[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Parse the first number appearing after `key` in a text dump. Works both
/// for busctl signatures ("Latitude" <d 55.75>) and JSON ("latitude":55.75).
fn number_after(text: &str, key: &str) -> Option<f64> {
    let from = text.find(key)? + key.len();
    let tail = &text[from..(from + 64).min(text.len())];
    let begin = tail.find(|c: char| c.is_ascii_digit() || c == '-')?;
    let num: String = tail[begin..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | '-'))
        .collect();
    num.parse().ok()
}

/// Read raw GPS coordinates from ModemManager when a cellular modem exists.
fn detect_modem() -> Option<GeoInfo> {
    run("mmcli", &["-m", "any", "--location-enable-gps-raw"]);
    std::thread::sleep(Duration::from_secs(2));
    let out = run("mmcli", &["-m", "any", "--location-get"])?;
    let from = out.find("GPS coordinates:")? + "GPS coordinates:".len();
    let nums: Vec<f64> = out[from..]
        .lines()
        .next()?
        .split(',')
        .map(|p| p.trim().parse().ok())
        .collect::<Option<Vec<_>>>()?;
    if nums.len() == 2 {
        Some(GeoInfo {
            latitude: nums[0],
            longitude: nums[1],
        })
    } else {
        None
    }
}

/// Coarse fallback: resolve the position from the public IP address.
fn detect_via_ip() -> Option<GeoInfo> {
    let out = run(
        "curl",
        &["-sS", "--max-time", "6", "https://ipapi.co/json/"],
    )?;
    Some(GeoInfo {
        latitude: number_after(&out, "\"latitude\"")?,
        longitude: number_after(&out, "\"longitude\"")?,
    })
}
