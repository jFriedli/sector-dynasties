//! Save format versioning and migration.
//!
//! `SimState::to_json`/`from_json` are thin wrappers around this module: it
//! isolates version detection, corrupt-data reporting, and the migration
//! chain from the rest of `state.rs`, so a future schema change has one
//! obvious place to land (see `migrate_step` below) instead of touching
//! `SimState::from_json` directly. See docs/ARCHITECTURE.md's persistence
//! section.

use std::fmt;

use serde_json::Value;

use crate::state::SAVE_SCHEMA_VERSION;

/// Why a save failed to load. Distinguishing these matters: a corrupt file
/// and a save from a future version are different problems with different
/// fixes, and neither should ever be silently coerced into a valid-looking
/// `SimState`.
#[derive(Debug)]
pub enum SaveError {
    /// The bytes are not valid JSON, or are valid JSON missing/mismatching
    /// a required field (including a missing or non-numeric
    /// `schema_version`).
    Corrupt(String),
    /// `schema_version` is newer than this build understands. Loading it
    /// anyway could silently drop or misinterpret fields, so it's refused.
    UnsupportedFutureVersion { found: u32, supported: u32 },
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Corrupt(msg) => write!(f, "corrupt save data: {msg}"),
            SaveError::UnsupportedFutureVersion { found, supported } => write!(
                f,
                "save format version {found} is newer than this build supports (up to {supported})"
            ),
        }
    }
}

impl std::error::Error for SaveError {}

/// Parse `json`, migrate it up to `SAVE_SCHEMA_VERSION` if it's an older
/// but known version, and return the resulting JSON `Value`. Returns a
/// `Value` rather than a typed `SimState` so `state.rs` owns the final
/// typed deserialization (and its errors, mapped to `SaveError::Corrupt`).
pub fn load_and_migrate(json: &str) -> Result<Value, SaveError> {
    let mut value: Value =
        serde_json::from_str(json).map_err(|e| SaveError::Corrupt(e.to_string()))?;

    let version = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| SaveError::Corrupt("missing or non-numeric schema_version".to_string()))?
        as u32;

    if version > SAVE_SCHEMA_VERSION {
        return Err(SaveError::UnsupportedFutureVersion {
            found: version,
            supported: SAVE_SCHEMA_VERSION,
        });
    }

    let mut current = version;
    while current < SAVE_SCHEMA_VERSION {
        value = migrate_step(current, value)?;
        current += 1;
    }

    Ok(value)
}

/// One migration step: `from_version` -> `from_version + 1`. Add a match
/// arm here each time `SAVE_SCHEMA_VERSION` bumps in a save-breaking way;
/// never edit an arm once it has shipped, since real saves may depend on
/// its exact behavior.
fn migrate_step(from_version: u32, mut value: Value) -> Result<Value, SaveError> {
    match from_version {
        // Synthetic scaffolding step: there has never been a real
        // schema_version 0 save (versioning shipped with v1), but this
        // proves the dispatch/chain shape a real migration will use. See
        // the tests below for the synthetic fixture this exercises.
        0 => {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("schema_version".to_string(), Value::from(1));
            }
            Ok(value)
        }
        other => Err(SaveError::Corrupt(format!(
            "no migration registered from schema_version {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SimState;

    #[test]
    fn a_synthetic_old_version_fixture_migrates_to_the_current_schema() {
        let current = SimState::new(2026).to_json();
        let mut fixture: Value = serde_json::from_str(&current).unwrap();
        fixture["schema_version"] = Value::from(0);

        let migrated = load_and_migrate(&fixture.to_string()).unwrap();
        assert_eq!(migrated["schema_version"], Value::from(SAVE_SCHEMA_VERSION));

        let restored: SimState = serde_json::from_value(migrated).unwrap();
        assert_eq!(restored.schema_version, SAVE_SCHEMA_VERSION);
    }

    #[test]
    fn an_unsupported_future_version_is_rejected_before_migration_runs() {
        let err = load_and_migrate(r#"{"schema_version": 999}"#).unwrap_err();
        assert!(matches!(
            err,
            SaveError::UnsupportedFutureVersion {
                found: 999,
                supported: SAVE_SCHEMA_VERSION
            }
        ));
    }

    #[test]
    fn malformed_json_is_rejected_as_corrupt_not_a_panic() {
        assert!(matches!(
            load_and_migrate("{not valid"),
            Err(SaveError::Corrupt(_))
        ));
    }

    #[test]
    fn a_value_missing_schema_version_is_rejected_as_corrupt() {
        assert!(matches!(load_and_migrate("{}"), Err(SaveError::Corrupt(_))));
    }

    #[test]
    fn a_non_object_value_is_rejected_as_corrupt_not_a_panic() {
        assert!(matches!(load_and_migrate("42"), Err(SaveError::Corrupt(_))));
    }
}
