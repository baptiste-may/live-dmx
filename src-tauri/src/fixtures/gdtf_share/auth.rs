use std::fs;
use std::path::Path;

use iota_stronghold::Client;
use tauri_plugin_stronghold::stronghold::Stronghold;

use super::types::GdtfShareCredentials;

// The vault encryption key is a random 32-byte secret unique to each
// installation. It is held in the OS credential store (Keychain on macOS,
// Credential Manager on Windows, Secret Service on Linux) and, when no keyring
// is available (for example on headless Linux), in a permission-restricted
// file under the app's local data directory.
const KEYRING_SERVICE: &str = "fr.baptiste-may.live-dmx";
const KEYRING_USER: &str = "gdtf-vault-key";
const KEY_FILE_NAME: &str = "vault.key";
const CLIENT_NAME: &str = "gdtf";
const CREDENTIALS_KEY: &[u8] = b"gdtf_credentials";

fn vault_path(app_data_dir: &Path) -> std::path::PathBuf {
    app_data_dir.join("vault.hold")
}

fn key_file_path(local_dir: &Path) -> std::path::PathBuf {
    local_dir.join(KEY_FILE_NAME)
}

/// Returns the installation's vault key, generating and persisting it on first
/// use. Existing keys are looked up in the OS keyring first, then in the local
/// fallback key file. A new key is only generated when both sources are
/// confirmed empty (or hold invalid data); genuine access errors are
/// propagated so a well-formed key is never silently replaced.
fn get_vault_key_with_service(local_dir: &Path, keyring_service: &str) -> Result<[u8; 32], String> {
    if let Some(key) = read_keyring_key(keyring_service)? {
        return Ok(key);
    }

    if let Some(key) = read_key_file(&key_file_path(local_dir))? {
        return Ok(key);
    }

    let mut key = [0u8; 32];
    getrandom::fill(&mut key).map_err(|e| format!("Failed to generate vault key: {}", e))?;

    if let Err(err) = write_keyring_key(keyring_service, &key) {
        log::warn!(
            "Could not persist vault key in OS keyring ({}); using local key file",
            err
        );
        write_key_file(&key_file_path(local_dir), &key)?;
    }
    Ok(key)
}

pub fn get_vault_key(local_dir: &Path) -> Result<[u8; 32], String> {
    get_vault_key_with_service(local_dir, KEYRING_SERVICE)
}

fn read_keyring_key(keyring_service: &str) -> Result<Option<[u8; 32]>, String> {
    let entry = match keyring::Entry::new(keyring_service, KEYRING_USER) {
        Ok(entry) => entry,
        Err(keyring::Error::NoDefaultStore) => {
            log::info!("OS keyring unavailable; falling back to local key file");
            return Ok(None);
        }
        Err(err) => return Err(format!("Failed to access OS keyring: {}", err)),
    };

    match entry.get_secret() {
        Ok(secret) if secret.len() == 32 => {
            let mut key = [0u8; 32];
            key.copy_from_slice(&secret);
            Ok(Some(key))
        }
        Ok(_) => {
            log::warn!("OS keyring entry holds an invalid vault key length; replacing it");
            Ok(None)
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(format!("Failed to read vault key from OS keyring: {}", err)),
    }
}

fn write_keyring_key(keyring_service: &str, key: &[u8; 32]) -> Result<(), String> {
    let entry = keyring::Entry::new(keyring_service, KEYRING_USER)
        .map_err(|e| format!("OS keyring unavailable: {}", e))?;
    entry
        .set_secret(key)
        .map_err(|e| format!("Failed to store vault key in OS keyring: {}", e))
}

fn read_key_file(path: &Path) -> Result<Option<[u8; 32]>, String> {
    match fs::read(path) {
        Ok(bytes) if bytes.len() == 32 => {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Ok(Some(key))
        }
        Ok(_) => {
            log::warn!("Invalid vault key file {}; replacing it", path.display());
            Ok(None)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!(
            "Failed to read vault key file {}: {}",
            path.display(),
            err
        )),
    }
}

fn write_key_file(path: &Path, key: &[u8; 32]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create app local data dir: {}", e))?;
    }
    write_key_file_contents(path, key)
        .map_err(|e| format!("Failed to write vault key file {}: {}", path.display(), e))
}

#[cfg(unix)]
fn write_key_file_contents(path: &Path, key: &[u8; 32]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true).mode(0o600);
    let mut file = opts.open(path)?;
    file.write_all(key)
}

#[cfg(not(unix))]
fn write_key_file_contents(path: &Path, key: &[u8; 32]) -> std::io::Result<()> {
    fs::write(path, key)
}

fn open_vault(app_data_dir: &Path, key: [u8; 32]) -> Result<Stronghold, String> {
    if !app_data_dir.exists() {
        fs::create_dir_all(app_data_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    let path = vault_path(app_data_dir);
    match Stronghold::new(&path, key.to_vec()) {
        Ok(vault) => Ok(vault),
        Err(err) => {
            // The snapshot cannot be decrypted (lost key) or is corrupt. Its
            // only payload is GDTF-Share credentials, so discard it and let a
            // fresh vault be recreated on next use.
            log::warn!(
                "Credential vault could not be opened ({}); discarding it",
                err
            );
            fs::remove_file(&path).ok();
            Stronghold::new(&path, key.to_vec())
                .map_err(|e| format!("Failed to initialize credential vault: {}", e))
        }
    }
}

fn open_client(vault: &Stronghold) -> Result<Client, String> {
    if let Ok(client) = vault.get_client(CLIENT_NAME) {
        return Ok(client);
    }
    match vault.load_client(CLIENT_NAME) {
        Ok(client) => Ok(client),
        Err(_) => vault
            .create_client(CLIENT_NAME)
            .map_err(|e| format!("Failed to create credential vault client: {}", e)),
    }
}

pub fn load_stored_credentials(
    app_data_dir: &Path,
    local_dir: &Path,
) -> Result<Option<(String, String)>, String> {
    let key = get_vault_key(local_dir)?;
    load_stored_credentials_with_key(app_data_dir, key)
}

fn load_stored_credentials_with_key(
    app_data_dir: &Path,
    key: [u8; 32],
) -> Result<Option<(String, String)>, String> {
    let vault = open_vault(app_data_dir, key)?;
    let client = match open_client(&vault) {
        Ok(client) => client,
        Err(_) => return Ok(None),
    };

    let data = client
        .store()
        .get(CREDENTIALS_KEY)
        .map_err(|e| format!("Failed to read GDTF credentials: {}", e))?;

    match data {
        Some(bytes) => {
            let creds = serde_json::from_slice::<GdtfShareCredentials>(&bytes)
                .map_err(|e| format!("Failed to parse GDTF credentials: {}", e))?;
            Ok(Some((creds.user, creds.password)))
        }
        None => Ok(None),
    }
}

pub fn save_stored_credentials(
    app_data_dir: &Path,
    local_dir: &Path,
    user: &str,
    pass: &str,
) -> Result<(), String> {
    let key = get_vault_key(local_dir)?;
    save_stored_credentials_with_key(app_data_dir, key, user, pass)
}

fn save_stored_credentials_with_key(
    app_data_dir: &Path,
    key: [u8; 32],
    user: &str,
    pass: &str,
) -> Result<(), String> {
    let vault = open_vault(app_data_dir, key)?;
    let client = open_client(&vault)?;

    let payload = serde_json::to_vec(&GdtfShareCredentials {
        user: user.to_string(),
        password: pass.to_string(),
    })
    .map_err(|e| format!("Failed to serialize GDTF credentials: {}", e))?;

    client
        .store()
        .insert(CREDENTIALS_KEY.to_vec(), payload, None)
        .map_err(|e| format!("Failed to store GDTF credentials: {}", e))?;

    vault
        .save()
        .map_err(|e| format!("Failed to persist GDTF credentials: {}", e))?;
    Ok(())
}

pub fn delete_stored_credentials(app_data_dir: &Path, local_dir: &Path) -> Result<(), String> {
    let key = get_vault_key(local_dir)?;
    delete_stored_credentials_with_key(app_data_dir, key)
}

fn delete_stored_credentials_with_key(app_data_dir: &Path, key: [u8; 32]) -> Result<(), String> {
    let vault = open_vault(app_data_dir, key)?;
    if let Ok(client) = open_client(&vault) {
        client
            .store()
            .delete(CREDENTIALS_KEY)
            .map_err(|e| format!("Failed to delete GDTF credentials: {}", e))?;
        vault
            .save()
            .map_err(|e| format!("Failed to persist credential removal: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const TEST_KEY: [u8; 32] = [7u8; 32];

    fn temp_app_data_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn test_credentials_roundtrip() {
        let app_data = temp_app_data_dir();
        save_stored_credentials_with_key(app_data.path(), TEST_KEY, "user1", "pass1").unwrap();
        let loaded = load_stored_credentials_with_key(app_data.path(), TEST_KEY).unwrap();
        assert_eq!(loaded, Some(("user1".into(), "pass1".into())));
    }

    #[test]
    fn test_load_missing_credentials_returns_none() {
        let app_data = temp_app_data_dir();
        let loaded = load_stored_credentials_with_key(app_data.path(), TEST_KEY).unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn test_delete_credentials() {
        let app_data = temp_app_data_dir();
        save_stored_credentials_with_key(app_data.path(), TEST_KEY, "user1", "pass1").unwrap();
        delete_stored_credentials_with_key(app_data.path(), TEST_KEY).unwrap();
        assert_eq!(
            load_stored_credentials_with_key(app_data.path(), TEST_KEY).unwrap(),
            None
        );
    }

    #[test]
    fn test_unreadable_vault_snapshot_is_discarded() {
        let app_data = temp_app_data_dir();
        // A vault encrypted with one key...
        save_stored_credentials_with_key(app_data.path(), [1u8; 32], "user1", "pass1").unwrap();
        // ...must be recoverable when read with a different key (lost key case)
        // instead of hard-failing.
        assert_eq!(
            load_stored_credentials_with_key(app_data.path(), [2u8; 32]).unwrap(),
            None
        );
    }

    #[test]
    fn test_vault_key_is_generated_and_stable() {
        let local_dir = TempDir::new().unwrap();
        let service = "live-dmx-test-vault-key";
        let key1 = get_vault_key_with_service(local_dir.path(), service).unwrap();
        let key2 = get_vault_key_with_service(local_dir.path(), service).unwrap();
        assert_eq!(key1, key2);

        // Clean up the keyring entry if one was created on this machine.
        if let Ok(entry) = keyring::Entry::new(service, KEYRING_USER) {
            let _ = entry.delete_credential();
        }
    }
}
