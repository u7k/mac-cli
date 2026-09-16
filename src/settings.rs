use crate::{
    error::{Error, Result},
    native, process, secure_fs,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Kind {
    Bool,
    Integer { min: i64, max: i64 },
    Number { min: f64, max: f64 },
    Choice { values: &'static [&'static str] },
    Text,
}
#[derive(Clone, Copy, Serialize)]
pub struct Setting {
    pub id: &'static str,
    pub domain: &'static str,
    pub key: &'static str,
    pub description: &'static str,
    pub value_type: Kind,
    pub experimental: bool,
    pub restart: Option<&'static str>,
    pub minimum_macos: u32,
}
macro_rules! setting {
    ($id:literal,$domain:literal,$key:literal,$description:literal,$kind:expr,$exp:expr,$restart:expr) => {
        Setting {
            id: $id,
            domain: $domain,
            key: $key,
            description: $description,
            value_type: $kind,
            experimental: $exp,
            restart: $restart,
            minimum_macos: 14,
        }
    };
}
pub static CATALOG: &[Setting] = &[
    setting!(
        "dock.autohide",
        "com.apple.dock",
        "autohide",
        "Automatically hide the Dock",
        Kind::Bool,
        false,
        Some("Dock")
    ),
    setting!(
        "dock.size",
        "com.apple.dock",
        "tilesize",
        "Dock icon size",
        Kind::Integer { min: 16, max: 128 },
        false,
        Some("Dock")
    ),
    setting!(
        "dock.position",
        "com.apple.dock",
        "orientation",
        "Dock position",
        Kind::Choice {
            values: &["left", "bottom", "right"]
        },
        false,
        Some("Dock")
    ),
    setting!(
        "dock.magnification",
        "com.apple.dock",
        "magnification",
        "Magnify Dock icons",
        Kind::Bool,
        false,
        Some("Dock")
    ),
    setting!(
        "dock.magnification-size",
        "com.apple.dock",
        "largesize",
        "Magnified Dock icon size",
        Kind::Integer { min: 16, max: 128 },
        false,
        Some("Dock")
    ),
    setting!(
        "dock.recent-apps",
        "com.apple.dock",
        "show-recents",
        "Show recent applications",
        Kind::Bool,
        false,
        Some("Dock")
    ),
    setting!(
        "dock.minimize-effect",
        "com.apple.dock",
        "mineffect",
        "Window minimize effect",
        Kind::Choice {
            values: &["genie", "scale"]
        },
        false,
        Some("Dock")
    ),
    setting!(
        "dock.autohide-delay",
        "com.apple.dock",
        "autohide-delay",
        "Delay before showing the Dock",
        Kind::Number { min: 0., max: 10. },
        true,
        Some("Dock")
    ),
    setting!(
        "dock.animation-time",
        "com.apple.dock",
        "autohide-time-modifier",
        "Dock reveal animation duration",
        Kind::Number { min: 0., max: 10. },
        true,
        Some("Dock")
    ),
    setting!(
        "dock.static-only",
        "com.apple.dock",
        "static-only",
        "Show only running applications",
        Kind::Bool,
        true,
        Some("Dock")
    ),
    setting!(
        "dock.single-app",
        "com.apple.dock",
        "single-app",
        "Hide other apps when switching from the Dock",
        Kind::Bool,
        true,
        Some("Dock")
    ),
    setting!(
        "finder.hidden-files",
        "com.apple.finder",
        "AppleShowAllFiles",
        "Show hidden files",
        Kind::Bool,
        false,
        Some("Finder")
    ),
    setting!(
        "finder.extensions",
        "NSGlobalDomain",
        "AppleShowAllExtensions",
        "Show filename extensions",
        Kind::Bool,
        false,
        Some("Finder")
    ),
    setting!(
        "finder.path-bar",
        "com.apple.finder",
        "ShowPathbar",
        "Show the path bar",
        Kind::Bool,
        false,
        Some("Finder")
    ),
    setting!(
        "finder.status-bar",
        "com.apple.finder",
        "ShowStatusBar",
        "Show the status bar",
        Kind::Bool,
        false,
        Some("Finder")
    ),
    setting!(
        "finder.default-view",
        "com.apple.finder",
        "FXPreferredViewStyle",
        "Default view: icon, list, column, or gallery",
        Kind::Choice {
            values: &["icnv", "Nlsv", "clmv", "Flwv"]
        },
        false,
        Some("Finder")
    ),
    setting!(
        "finder.folders-first",
        "com.apple.finder",
        "_FXSortFoldersFirst",
        "Sort folders first by name",
        Kind::Bool,
        false,
        Some("Finder")
    ),
    setting!(
        "finder.quit-menu",
        "com.apple.finder",
        "QuitMenuItem",
        "Add Quit to the Finder menu",
        Kind::Bool,
        true,
        Some("Finder")
    ),
    setting!(
        "finder.title-path",
        "com.apple.finder",
        "_FXShowPosixPathInTitle",
        "Show full paths in window titles",
        Kind::Bool,
        true,
        Some("Finder")
    ),
    setting!(
        "finder.desktop-icons",
        "com.apple.finder",
        "CreateDesktop",
        "Show desktop icons",
        Kind::Bool,
        true,
        Some("Finder")
    ),
    setting!(
        "keyboard.repeat",
        "NSGlobalDomain",
        "KeyRepeat",
        "Key repeat interval; smaller is faster",
        Kind::Integer { min: 1, max: 120 },
        false,
        None
    ),
    setting!(
        "keyboard.initial-repeat",
        "NSGlobalDomain",
        "InitialKeyRepeat",
        "Delay before key repetition",
        Kind::Integer { min: 10, max: 120 },
        false,
        None
    ),
    setting!(
        "keyboard.press-and-hold",
        "NSGlobalDomain",
        "ApplePressAndHoldEnabled",
        "Show accent selection on key hold",
        Kind::Bool,
        false,
        None
    ),
    setting!(
        "keyboard.autocorrect",
        "NSGlobalDomain",
        "NSAutomaticSpellingCorrectionEnabled",
        "Correct spelling automatically",
        Kind::Bool,
        false,
        None
    ),
    setting!(
        "keyboard.smart-quotes",
        "NSGlobalDomain",
        "NSAutomaticQuoteSubstitutionEnabled",
        "Use smart quotation marks",
        Kind::Bool,
        false,
        None
    ),
    setting!(
        "keyboard.smart-dashes",
        "NSGlobalDomain",
        "NSAutomaticDashSubstitutionEnabled",
        "Use smart dashes",
        Kind::Bool,
        false,
        None
    ),
    setting!(
        "keyboard.auto-capitalization",
        "NSGlobalDomain",
        "NSAutomaticCapitalizationEnabled",
        "Capitalize words automatically",
        Kind::Bool,
        false,
        None
    ),
    setting!(
        "trackpad.tap-to-click",
        "com.apple.AppleMultitouchTrackpad",
        "Clicking",
        "Tap to click on the built-in trackpad",
        Kind::Bool,
        false,
        None
    ),
    setting!(
        "trackpad.bluetooth-tap-to-click",
        "com.apple.driver.AppleBluetoothMultitouch.trackpad",
        "Clicking",
        "Tap to click on a Bluetooth trackpad",
        Kind::Bool,
        false,
        None
    ),
    setting!(
        "trackpad.natural-scroll",
        "NSGlobalDomain",
        "com.apple.swipescrolldirection",
        "Use natural scrolling",
        Kind::Bool,
        false,
        None
    ),
    setting!(
        "trackpad.three-finger-drag",
        "com.apple.AppleMultitouchTrackpad",
        "TrackpadThreeFingerDrag",
        "Drag using three fingers",
        Kind::Bool,
        true,
        None
    ),
    setting!(
        "screenshot.location",
        "com.apple.screencapture",
        "location",
        "Folder for screenshots",
        Kind::Text,
        false,
        Some("SystemUIServer")
    ),
    setting!(
        "screenshot.format",
        "com.apple.screencapture",
        "type",
        "Screenshot file format",
        Kind::Choice {
            values: &["png", "jpg", "pdf", "tiff"]
        },
        false,
        Some("SystemUIServer")
    ),
    setting!(
        "screenshot.disable-shadow",
        "com.apple.screencapture",
        "disable-shadow",
        "Disable window screenshot shadows",
        Kind::Bool,
        false,
        Some("SystemUIServer")
    ),
    setting!(
        "screenshot.thumbnail",
        "com.apple.screencapture",
        "show-thumbnail",
        "Show the screenshot thumbnail",
        Kind::Bool,
        false,
        Some("SystemUIServer")
    ),
    setting!(
        "appearance.auto",
        "NSGlobalDomain",
        "AppleInterfaceStyleSwitchesAutomatically",
        "Switch appearance automatically",
        Kind::Bool,
        false,
        None
    ),
];
pub fn find(id: &str) -> Result<&'static Setting> {
    CATALOG
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| Error::invalid(format!("Unknown setting: {id}. Run mac settings list.")))
}
fn validate(setting: &Setting, value: &Value, experimental: bool) -> Result<()> {
    if setting.experimental && !experimental {
        return Err(Error::new(
            "experimental_required",
            format!("{} requires --experimental.", setting.id),
        ));
    }
    let valid = match setting.value_type {
        Kind::Bool => value.is_boolean(),
        Kind::Integer { min, max } => value.as_i64().is_some_and(|v| (min..=max).contains(&v)),
        Kind::Number { min, max } => value
            .as_f64()
            .is_some_and(|v| v.is_finite() && (min..=max).contains(&v)),
        Kind::Choice { values } => value.as_str().is_some_and(|v| values.contains(&v)),
        Kind::Text => value
            .as_str()
            .is_some_and(|v| !v.is_empty() && !v.contains('\0')),
    };
    if !valid {
        return Err(Error::invalid(format!(
            "Invalid value for {}. Run mac settings list for its type and allowed values.",
            setting.id
        )));
    }
    if setting.id == "screenshot.location" {
        let p = Path::new(value.as_str().unwrap());
        if !p.is_absolute() || !p.is_dir() {
            return Err(Error::invalid(
                "Screenshot location must be an existing absolute directory.",
            ));
        }
    }
    Ok(())
}
pub fn parse(id: &str, text: &str, experimental: bool) -> Result<Value> {
    let s = find(id)?;
    let value = match s.value_type {
        Kind::Text | Kind::Choice { .. } => Value::String(text.into()),
        _ => serde_json::from_str(text).map_err(|_| {
            Error::invalid(format!(
                "Invalid value for {id}: expected an English boolean or number."
            ))
        })?,
    };
    validate(s, &value, experimental)?;
    Ok(value)
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}
impl Snapshot {
    #[cfg(test)]
    fn absent() -> Self {
        Self {
            exists: false,
            value: None,
        }
    }
    fn from_value(value: Value) -> Self {
        Self {
            exists: true,
            value: Some(value),
        }
    }
    fn consistent(&self) -> bool {
        self.exists == self.value.is_some() && !self.value.as_ref().is_some_and(Value::is_null)
    }
}
pub trait Backend {
    fn read(&mut self, setting: &Setting) -> Result<Snapshot>;
    fn write(&mut self, setting: &Setting, value: &Snapshot) -> Result<()>;
    fn restart(&mut self, apps: &BTreeSet<String>) -> Result<()>;
}
pub struct MacBackend;
impl Backend for MacBackend {
    fn read(&mut self, s: &Setting) -> Result<Snapshot> {
        let value = native::call("preferences.read", json!({"domain":s.domain,"key":s.key}))?;
        decode_snapshot(value)
    }
    fn write(&mut self, s: &Setting, v: &Snapshot) -> Result<()> {
        native::call(
            "preferences.write",
            json!({"domain":s.domain,"key":s.key,"value":v.value}),
        )?;
        Ok(())
    }
    fn restart(&mut self, apps: &BTreeSet<String>) -> Result<()> {
        for app in apps {
            // A process that is not running needs no restart.
            let running = process::tool("/usr/bin/pgrep", &["-x", app]);
            if running.is_ok() {
                process::tool("/usr/bin/killall", &[app])?;
            }
        }
        Ok(())
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema_version: u32,
    pub settings: BTreeMap<String, Snapshot>,
}
fn decode_snapshot(value: Value) -> Result<Snapshot> {
    let snapshot: Snapshot = serde_json::from_value(value)?;
    if !snapshot.consistent() {
        return Err(Error::new(
            "invalid_response",
            "The preference backend returned an inconsistent snapshot.",
        ));
    }
    Ok(snapshot)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    key: String,
    before: Snapshot,
    after: Snapshot,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    id: String,
    status: String,
    changes: Vec<Change>,
    #[serde(default)]
    rollback_errors: Vec<String>,
}
pub struct Store {
    root: PathBuf,
}
fn validate_stored(setting: &Setting, snapshot: &Snapshot) -> Result<()> {
    if !snapshot.consistent() {
        return Err(Error::invalid("Inconsistent preference snapshot."));
    }
    if let Some(value) = &snapshot.value {
        let valid = match setting.value_type {
            Kind::Bool => value.is_boolean() || matches!(value.as_i64(), Some(0 | 1)),
            Kind::Integer { .. } => value.is_i64(),
            Kind::Number { .. } => value.as_f64().is_some_and(f64::is_finite),
            Kind::Choice { .. } | Kind::Text => value.as_str().is_some_and(|v| !v.contains('\0')),
        };
        if !valid {
            return Err(Error::invalid(format!(
                "Stored {} has an unexpected type; refusing to change it.",
                setting.id
            )));
        }
    }
    Ok(())
}
pub fn validate_native_preference(domain: &str, key: &str, value: &Value) -> Result<()> {
    let setting = CATALOG
        .iter()
        .find(|s| s.domain == domain && s.key == key)
        .ok_or_else(|| {
            Error::invalid("Native preference writes are limited to the settings catalog.")
        })?;
    if value.is_null() {
        return Ok(());
    }
    validate_stored(setting, &Snapshot::from_value(value.clone()))
}
fn validate_journal(journal: &Journal) -> Result<()> {
    if ![
        "pending",
        "applied",
        "undone",
        "rolled_back",
        "rollback_failed",
    ]
    .contains(&journal.status.as_str())
        || journal.changes.is_empty()
        || journal.changes.len() > CATALOG.len()
    {
        return Err(Error::invalid("Invalid operation journal."));
    }
    let mut seen = BTreeSet::new();
    for change in &journal.changes {
        if !seen.insert(&change.key) {
            return Err(Error::invalid("Duplicate settings in operation journal."));
        }
        let setting = find(&change.key)?;
        validate_stored(setting, &change.before)?;
        validate_stored(setting, &change.after)?;
    }
    Ok(())
}
impl Store {
    pub fn user() -> Result<Self> {
        let root = if let Some(base) = std::env::var_os("XDG_STATE_HOME") {
            PathBuf::from(base)
        } else {
            PathBuf::from(
                std::env::var_os("HOME")
                    .ok_or_else(|| Error::new("configuration_error", "HOME is not set."))?,
            )
            .join(".local/state")
        };
        if !root.is_absolute() {
            return Err(Error::new(
                "configuration_error",
                "The state directory must be absolute.",
            ));
        }
        Ok(Self {
            root: root.join("mac-cli"),
        })
    }
    fn save_journal(state: &secure_fs::State, j: &Journal) -> Result<()> {
        state.write(&format!("{}.json", j.id), &serde_json::to_vec_pretty(j)?)
    }
    pub fn apply<B: Backend>(
        &self,
        backend: &mut B,
        settings: &BTreeMap<String, Snapshot>,
        experimental: bool,
    ) -> Result<Value> {
        let state = secure_fs::State::open(&self.root)?;
        // Validate the entire profile and capture all previous values before the first write.
        let mut changes = Vec::new();
        for (key, value) in settings {
            let s = find(key)?;
            if !value.consistent() {
                return Err(Error::invalid(format!("Invalid snapshot for {key}.")));
            }
            if s.experimental && !experimental {
                return Err(Error::new(
                    "experimental_required",
                    format!("{key} requires --experimental."),
                ));
            }
            if let Some(v) = &value.value {
                validate(s, v, experimental)?;
            }
            let before = backend.read(s)?;
            validate_stored(s, &before)?;
            if before != *value {
                changes.push(Change {
                    key: key.clone(),
                    before,
                    after: value.clone(),
                });
            }
        }
        if changes.is_empty() {
            return Ok(json!({"changed":false,"message":"All settings already match."}));
        }
        let id = format!(
            "{}-{}",
            chrono::Utc::now().format("%Y%m%dT%H%M%S%.9fZ"),
            std::process::id()
        );
        let mut journal = Journal {
            schema_version: 1,
            id: id.clone(),
            status: "pending".into(),
            changes,
            rollback_errors: Vec::new(),
        };
        Self::save_journal(&state, &journal)?;
        let mut apps = BTreeSet::new();
        let outcome = (|| -> Result<()> {
            for change in &journal.changes {
                let s = find(&change.key)?;
                // Include the current step even if its write fails after changing the preference.
                if let Some(app) = s.restart {
                    apps.insert(app.into());
                }
                backend.write(s, &change.after)?;
                if backend.read(s)? != change.after {
                    return Err(Error::new(
                        "verification_failed",
                        format!("{} did not retain the requested value.", s.id),
                    ));
                }
            }
            backend.restart(&apps)?;
            journal.status = "applied".into();
            Self::save_journal(&state, &journal)?;
            Ok(())
        })();
        if let Err(error) = outcome {
            let rollback_errors = rollback(backend, &journal.changes, &apps);
            journal.status = if rollback_errors.is_empty() {
                "rolled_back"
            } else {
                "rollback_failed"
            }
            .into();
            journal.rollback_errors = rollback_errors;
            Self::save_journal(&state, &journal)?;
            return Err(Error::new(
                "transaction_failed",
                format!(
                    "{error} Operation: {id}. Rollback: {}.",
                    if journal.rollback_errors.is_empty() {
                        "completed".into()
                    } else {
                        journal.rollback_errors.join("; ")
                    }
                ),
            ));
        }
        Ok(
            json!({"changed":true,"operation":id,"settings":journal.changes.iter().map(|c|&c.key).collect::<Vec<_>>(),"restarted":apps,"note":"Some keyboard and trackpad preferences take effect after reopening applications or signing in again."}),
        )
    }
    pub fn undo<B: Backend>(&self, backend: &mut B, id: &str) -> Result<Value> {
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        {
            return Err(Error::invalid("Invalid operation ID."));
        }
        let state = secure_fs::State::open(&self.root)?;
        let mut journal: Journal = serde_json::from_slice(&state.read(&format!("{id}.json"))?)?;
        if journal.schema_version != 1 || journal.id != id {
            return Err(Error::invalid(
                "Unsupported or mismatched operation journal.",
            ));
        }
        validate_journal(&journal)?;
        if journal.status == "undone" || journal.status == "rolled_back" {
            return Ok(json!({"operation":id,"changed":false}));
        }
        let mut apps = BTreeSet::new();
        for change in &journal.changes {
            let s = find(&change.key)?;
            let current = backend.read(s)?;
            if current != change.after && current != change.before {
                return Err(Error::new(
                    "conflict",
                    format!(
                        "{} has changed since this operation; nothing was undone.",
                        change.key
                    ),
                ));
            }
            if let Some(app) = s.restart {
                apps.insert(app.into());
            }
        }
        let errors = rollback(backend, &journal.changes, &apps);
        journal.rollback_errors = errors;
        journal.status = if journal.rollback_errors.is_empty() {
            "undone"
        } else {
            "rollback_failed"
        }
        .into();
        Self::save_journal(&state, &journal)?;
        if !journal.rollback_errors.is_empty() {
            return Err(Error::new(
                "rollback_failed",
                journal.rollback_errors.join("; "),
            ));
        }
        Ok(json!({"operation":id,"undone":true,"restarted":apps}))
    }
}
fn rollback<B: Backend>(
    backend: &mut B,
    changes: &[Change],
    apps: &BTreeSet<String>,
) -> Vec<String> {
    let mut errors = Vec::new();
    for change in changes.iter().rev() {
        let s = match find(&change.key) {
            Ok(s) => s,
            Err(e) => {
                errors.push(e.to_string());
                continue;
            }
        };
        let attempt = (|| -> Result<()> {
            let current = backend.read(s)?;
            if current == change.before {
                return Ok(());
            }
            if current != change.after {
                return Err(Error::new(
                    "conflict",
                    format!("{} changed outside this operation.", s.id),
                ));
            }
            backend.write(s, &change.before)?;
            if backend.read(s)? != change.before {
                return Err(Error::new(
                    "verification_failed",
                    format!("Could not restore {}.", s.id),
                ));
            }
            Ok(())
        })();
        if let Err(e) = attempt {
            errors.push(e.to_string());
        }
    }
    if let Err(e) = backend.restart(apps) {
        errors.push(e.to_string());
    }
    errors
}
pub fn read_profile(path: &Path) -> Result<Profile> {
    let profile: Profile = toml::from_str(
        std::str::from_utf8(&secure_fs::read_bounded(path)?)
            .map_err(|_| Error::invalid("Profiles must contain UTF-8 text."))?,
    )
    .map_err(|e| Error::invalid(format!("Invalid TOML profile: {e}")))?;
    if profile.schema_version != 1 {
        return Err(Error::invalid("Unsupported profile schema version."));
    }
    for (key, snapshot) in &profile.settings {
        let setting = find(key)?;
        if !snapshot.consistent() {
            return Err(Error::invalid(format!("Invalid snapshot for {key}.")));
        }
        if let Some(v) = &snapshot.value {
            validate(setting, v, true)?;
        }
    }
    Ok(profile)
}
pub fn save_profile(path: &Path, keys: &[String], experimental: bool) -> Result<Value> {
    let mut settings = BTreeMap::new();
    let mut backend = MacBackend;
    for key in keys {
        let s = find(key)?;
        if s.experimental && !experimental {
            return Err(Error::new(
                "experimental_required",
                format!("{key} requires --experimental."),
            ));
        }
        let mut snapshot = backend.read(s)?;
        validate_stored(s, &snapshot)?;
        // macOS sometimes persists logical preferences as the integers zero/one.
        if matches!(s.value_type, Kind::Bool) {
            if let Some(n) = snapshot.value.as_ref().and_then(Value::as_i64) {
                snapshot.value = Some(json!(n != 0));
            }
        }
        settings.insert(key.clone(), snapshot);
    }
    let profile = Profile {
        schema_version: 1,
        settings,
    };
    let text = toml::to_string_pretty(&profile)
        .map_err(|e| Error::new("encoding_error", e.to_string()))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    Ok(json!({"saved":path,"settings":profile.settings.len()}))
}
pub fn get(key: &str) -> Result<Value> {
    let s = find(key)?;
    let current = MacBackend.read(s)?;
    Ok(
        json!({"setting":s,"current":current,"note":if current.exists{"Stored preference; effective UI behavior may require an application restart."}else{"No explicit override; macOS chooses its default."}}),
    )
}
pub fn set(key: &str, text: &str, experimental: bool) -> Result<Value> {
    let value = parse(key, text, experimental)?;
    Store::user()?.apply(
        &mut MacBackend,
        &BTreeMap::from([(key.into(), Snapshot::from_value(value))]),
        experimental,
    )
}
pub fn diff(profile: &Profile) -> Result<Value> {
    let mut rows = Vec::new();
    let mut backend = MacBackend;
    for (key, desired) in &profile.settings {
        let current = backend.read(find(key)?)?;
        rows.push(
            json!({"key":key,"current":current,"desired":desired,"changed":current!=*desired}),
        );
    }
    Ok(Value::Array(rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Mock {
        values: BTreeMap<String, Snapshot>,
        fail: Option<String>,
        writes: usize,
        restarts: Vec<BTreeSet<String>>,
    }
    impl Backend for Mock {
        fn read(&mut self, s: &Setting) -> Result<Snapshot> {
            Ok(self
                .values
                .get(s.id)
                .cloned()
                .unwrap_or_else(Snapshot::absent))
        }
        fn write(&mut self, s: &Setting, v: &Snapshot) -> Result<()> {
            self.writes += 1;
            if self.fail.as_deref() == Some(s.id) {
                self.fail = None;
                return Err(Error::new("test_failure", "Simulated write failure."));
            }
            self.values.insert(s.id.into(), v.clone());
            Ok(())
        }
        fn restart(&mut self, apps: &BTreeSet<String>) -> Result<()> {
            self.restarts.push(apps.clone());
            Ok(())
        }
    }
    #[test]
    fn catalog_validation() {
        let ids: BTreeSet<_> = CATALOG.iter().map(|s| s.id).collect();
        assert_eq!(ids.len(), CATALOG.len());
        assert!(parse("dock.size", "300", false).is_err());
        assert!(parse("dock.autohide", "evet", false).is_err());
        assert!(parse("dock.autohide-delay", "0", false).is_err());
        assert_eq!(parse("dock.autohide", "true", false).unwrap(), json!(true));
    }
    #[test]
    fn restores_absent_key_and_consolidates_restarts() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store {
            root: dir.path().join("state"),
        };
        let mut mock = Mock::default();
        let values = BTreeMap::from([
            ("dock.autohide".into(), Snapshot::from_value(json!(true))),
            ("dock.size".into(), Snapshot::from_value(json!(60))),
        ]);
        let result = store.apply(&mut mock, &values, false).unwrap();
        assert_eq!(mock.restarts[0], BTreeSet::from(["Dock".into()]));
        store
            .undo(&mut mock, result["operation"].as_str().unwrap())
            .unwrap();
        assert!(!mock.values["dock.autohide"].exists);
        assert!(!mock.values["dock.size"].exists);
    }
    #[test]
    fn partial_failure_rolls_back() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store {
            root: dir.path().join("state"),
        };
        let mut mock = Mock {
            fail: Some("dock.size".into()),
            ..Default::default()
        };
        let values = BTreeMap::from([
            ("dock.autohide".into(), Snapshot::from_value(json!(true))),
            ("dock.size".into(), Snapshot::from_value(json!(60))),
        ]);
        assert_eq!(
            store.apply(&mut mock, &values, false).unwrap_err().code,
            "transaction_failed"
        );
        assert!(!mock.values["dock.autohide"].exists);
    }
    #[test]
    fn validation_precedes_all_writes() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store {
            root: dir.path().join("state"),
        };
        let mut mock = Mock::default();
        let values = BTreeMap::from([
            ("dock.autohide".into(), Snapshot::from_value(json!(true))),
            ("dock.size".into(), Snapshot::from_value(json!(500))),
        ]);
        assert!(store.apply(&mut mock, &values, false).is_err());
        assert_eq!(mock.writes, 0);
    }
    #[test]
    fn undo_refuses_external_changes() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store {
            root: dir.path().join("state"),
        };
        let mut mock = Mock::default();
        let result = store
            .apply(
                &mut mock,
                &BTreeMap::from([("dock.size".into(), Snapshot::from_value(json!(60)))]),
                false,
            )
            .unwrap();
        mock.values
            .insert("dock.size".into(), Snapshot::from_value(json!(70)));
        assert_eq!(
            store
                .undo(&mut mock, result["operation"].as_str().unwrap())
                .unwrap_err()
                .code,
            "conflict"
        );
        assert_eq!(mock.values["dock.size"].value, Some(json!(70)));
    }
    #[test]
    fn profile_preserves_unicode_and_absence() {
        let p = Profile {
            schema_version: 1,
            settings: BTreeMap::from([("dock.autohide".into(), Snapshot::absent())]),
        };
        let text = toml::to_string(&p).unwrap();
        let decoded: Profile = toml::from_str(&text).unwrap();
        assert_eq!(decoded.settings, p.settings);
        assert!(!Snapshot {
            exists: false,
            value: Some(json!("bad"))
        }
        .consistent());
    }
    #[test]
    fn malformed_native_existence_cannot_be_mistaken_for_absence() {
        assert!(decode_snapshot(json!({"exists":1,"value":60})).is_err());
        assert!(decode_snapshot(json!({"exists":true,"value":null})).is_err());
        assert_eq!(
            decode_snapshot(json!({"exists":true,"value":60})).unwrap(),
            Snapshot::from_value(json!(60))
        );
        assert_eq!(
            decode_snapshot(json!({"exists":false,"value":null})).unwrap(),
            Snapshot::absent()
        );
    }
    #[test]
    fn malformed_journal_is_rejected_before_any_preference_access() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("state");
        let state = secure_fs::State::open(&root).unwrap();
        state.write("example.json",br#"{"schema_version":1,"id":"example","status":"applied","changes":[{"key":"dock.size","before":{"exists":true,"value":{"unexpected":true}},"after":{"exists":false}}]}"#).unwrap();
        drop(state);
        let mut mock = Mock::default();
        let store = Store { root };
        assert!(store.undo(&mut mock, "example").is_err());
        assert_eq!(mock.writes, 0);
    }
}
