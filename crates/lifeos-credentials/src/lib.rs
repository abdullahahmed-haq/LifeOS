use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lifeos_domain::AppError;

const SERVICE_NAME: &str = "com.lifeos.desktop";

/// Rust-only credential seam. Values never enter SQLite, contracts, logs, or
/// frontend persistence; callers retain only an opaque reference ID.
pub trait CredentialStore: Send + Sync {
    fn write(&self, reference: &str, secret: &str) -> Result<(), AppError>;
    fn read(&self, reference: &str) -> Result<String, AppError>;
    fn revoke(&self, reference: &str) -> Result<(), AppError>;
}

pub struct NativeCredentialStore;

impl CredentialStore for NativeCredentialStore {
    fn write(&self, reference: &str, secret: &str) -> Result<(), AppError> {
        entry(reference)?.set_password(secret).map_err(unavailable)
    }

    fn read(&self, reference: &str) -> Result<String, AppError> {
        entry(reference)?.get_password().map_err(unavailable)
    }

    fn revoke(&self, reference: &str) -> Result<(), AppError> {
        entry(reference)?.delete_credential().map_err(unavailable)
    }
}

fn entry(reference: &str) -> Result<keyring::Entry, AppError> {
    keyring::Entry::new(SERVICE_NAME, reference).map_err(unavailable)
}

fn unavailable(_error: impl std::fmt::Display) -> AppError {
    AppError::Unavailable {
        service: "native credential store".into(),
    }
}

#[derive(Clone, Default)]
pub struct MemoryCredentialStore {
    values: Arc<Mutex<HashMap<String, String>>>,
}

impl CredentialStore for MemoryCredentialStore {
    fn write(&self, reference: &str, secret: &str) -> Result<(), AppError> {
        self.values
            .lock()
            .map_err(|_| unavailable("memory credential store lock"))?
            .insert(reference.to_owned(), secret.to_owned());
        Ok(())
    }

    fn read(&self, reference: &str) -> Result<String, AppError> {
        self.values
            .lock()
            .map_err(|_| unavailable("memory credential store lock"))?
            .get(reference)
            .cloned()
            .ok_or_else(|| unavailable("credential is absent"))
    }

    fn revoke(&self, reference: &str) -> Result<(), AppError> {
        self.values
            .lock()
            .map_err(|_| unavailable("memory credential store lock"))?
            .remove(reference);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_replaces_reads_and_revokes_without_exposing_values() {
        let store = MemoryCredentialStore::default();
        store.write("test", "first").unwrap();
        store.write("test", "second").unwrap();
        assert_eq!(store.read("test").unwrap(), "second");
        store.revoke("test").unwrap();
        assert!(matches!(
            store.read("test"),
            Err(AppError::Unavailable { .. })
        ));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_macos_keychain_round_trip() {
        let store = NativeCredentialStore;
        let reference = format!("lifeos-test-{}", uuid::Uuid::now_v7());
        store.write(&reference, "test-value").unwrap();
        assert_eq!(store.read(&reference).unwrap(), "test-value");
        store.revoke(&reference).unwrap();
    }
}
