//! OS compatibility checks used by the auto-updater.
//!
//! Installed bundles require macOS 11+ because native UI zoom relies on
//! `WKWebView.pageZoom`. WebKit freezes its user agent at 10.15 on newer macOS
//! versions, so the real version has to be read on the backend. See
//! `docs/contributing/releasing.md`.

#[cfg(target_os = "macos")]
use std::process::Command;

/// Oldest macOS release supported by current bundles, as `(major, minor)`.
const MIN_MACOS_VERSION: (u32, u32) = (11, 0);

/// Splits a `sw_vers -productVersion` value into its major and minor components.
///
/// Whitespace and the patch component are ignored; malformed input yields
/// `None` so callers decide how to report it.
pub fn parse_product_version(output: &str) -> Option<(u32, u32)> {
    let mut parts = output.trim().split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

/// Reports whether a macOS version is older than the minimum supported one.
///
/// Comparing the full `(major, minor)` pair keeps the check correct if the
/// minimum ever gains a minor-version requirement.
pub fn version_is_below_minimum(major: u32, minor: u32) -> bool {
    (major, minor) < MIN_MACOS_VERSION
}

/// Reads the running macOS version through `sw_vers`, present on all macOS releases.
#[cfg(target_os = "macos")]
fn current_macos_version() -> Result<(u32, u32), String> {
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        // Compatibility mode makes macOS 11 report 10.16 when this variable is
        // inherited. Request the real version so supported systems are not
        // mistaken for pre-11 systems.
        .env("SYSTEM_VERSION_COMPAT", "0")
        .output()
        .map_err(|error| format!("Failed to run sw_vers: {error}"))?;
    if !output.status.success() {
        return Err(format!("sw_vers exited with {}", output.status));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("Invalid sw_vers output: {error}"))?;
    parse_product_version(&stdout).ok_or_else(|| format!("Unrecognized macOS version: {stdout}"))
}

/// Returns whether the running macOS is too old to install current releases.
///
/// Always `Ok(false)` off macOS so other platforms keep the normal update flow.
/// On macOS a failed probe is an error: the frontend treats it as unknown and
/// lets the update proceed, so a broken probe cannot strand supported users.
/// Touches no `AppState`, emits no events and creates no undo entries.
///
/// # Errors
///
/// Returns a message when `sw_vers` cannot be run or its output is unrecognized.
/// Never errors on non-macOS platforms.
#[tauri::command]
pub fn is_macos_below_minimum() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let (major, minor) = current_macos_version()?;
        Ok(version_is_below_minimum(major, minor))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(false)
    }
}

/// Unit tests for the pure version helpers; `sw_vers` itself needs a Mac.
#[cfg(test)]
mod tests {
    use super::{parse_product_version, version_is_below_minimum};

    /// Product versions are `major.minor[.patch]` with optional whitespace.
    #[test]
    fn parses_product_version() {
        assert_eq!(parse_product_version("14.5.1\n"), Some((14, 5)));
        assert_eq!(parse_product_version("10.15.7"), Some((10, 15)));
        assert_eq!(parse_product_version("unknown"), None);
    }

    /// Only macOS releases before 11.0 are incompatible with current bundles.
    #[test]
    fn compares_against_minimum() {
        assert!(version_is_below_minimum(10, 15));
        assert!(version_is_below_minimum(10, 16));
        assert!(!version_is_below_minimum(11, 0));
        assert!(!version_is_below_minimum(12, 1));
    }
}
