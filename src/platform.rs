//! Platform glue: save-file location, atomic writes, the gate
//! scratch root, the calendar date (read here, at the IO layer, so
//! challenge logic stays deterministic), and run logging.
//!
//! The shell is the only caller. Everything in this module is a
//! thin, honest wrapper over the operating system: paths are
//! requests, writes are atomic (temp file + rename), and failures
//! come back as values the game can narrate.

use std::path::PathBuf;

/// Emit one run-evidence line on both observable channels: stdout
/// (desktop) and the Bevy log (Android logcat via bevy_log).
pub fn log_line(line: String) {
    println!("NIFLHEIM {line}");
    bevy::log::info!("NIFLHEIM {line}");
}

/// Where the single-file save lives on this platform. The save's
/// records are identical on every platform; only its home differs.
pub fn save_path() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        // The engine's own activity handle owns the JNI side; the
        // shell only reads the path it publishes.
        if let Some(app) = bevy_android::ANDROID_APP.get() {
            if let Some(dir) = app.internal_data_path() {
                return dir.join("niflheim-save.json");
            }
        }
    }
    if let Ok(path) = std::env::var("NIFLHEIM_SAVE_PATH") {
        return PathBuf::from(path);
    }
    PathBuf::from("niflheim-save.json")
}

/// The root under which the FOE gate assembles its scratch cargo
/// projects. Bounded by the gate runner: one project directory per
/// run, removed when the run ends.
pub fn gate_scratch_root() -> PathBuf {
    if let Ok(path) = std::env::var("NIFLHEIM_GATE_DIR") {
        return PathBuf::from(path);
    }
    std::env::temp_dir().join("niflheim-gate")
}

/// Write `bytes` to `path` atomically (temp file + rename), so a
/// crash mid-write can never leave a torn save behind.
pub fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

/// Today's date as `YYYY-MM-DD` in local time, from the system
/// clock. Used only to inscribe the map when a FOE falls; it never
/// feeds challenge logic.
pub fn today_string() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Local offset: the build machines run in the player's zone; a
    // UTC date is an acceptable inscription, so no tz database is
    // consulted. Days since the epoch convert to a civil date with
    // the standard algorithm (Howard Hinnant's civil_from_days).
    let days = (seconds / 86_400) as i64;
    civil_from_days(days)
}

/// Convert days since 1970-01-01 to a `YYYY-MM-DD` string.
fn civil_from_days(days_since_epoch: i64) -> String {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::civil_from_days;

    /// Validation: known epoch landmarks convert exactly.
    #[test]
    fn civil_dates_match_known_landmarks() {
        assert_eq!(civil_from_days(0), "1970-01-01");
        assert_eq!(civil_from_days(19_723), "2024-01-01");
        assert_eq!(civil_from_days(20_381), "2025-10-20");
        assert_eq!(civil_from_days(20_398), "2025-11-06");
    }

    /// Adversarial: leap-day and year-boundary arithmetic.
    #[test]
    fn civil_dates_handle_leap_boundaries() {
        assert_eq!(civil_from_days(19_782), "2024-02-29");
        assert_eq!(civil_from_days(19_783), "2024-03-01");
        assert_eq!(civil_from_days(20_089), "2025-01-01");
    }
}
