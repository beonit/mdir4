use std::fs;

use mdir4::config::{
    self,
    schema::{Config, QcdEntry, ViewMode},
};
use tempfile::tempdir;

#[test]
fn config_roundtrip_partial_and_unknown_fields_are_supported() {
    let temporary = tempdir().unwrap();
    let path = temporary.path().join("config.toml");
    let mut expected = Config {
        view: ViewMode::Long,
        theme: "dark".to_string(),
        ..Config::default()
    };
    expected.qcd.push(QcdEntry {
        label: "Work 한글".to_string(),
        path: temporary.path().to_path_buf(),
        position: 0,
    });
    config::save_atomic(&path, &expected).unwrap();
    assert_eq!(config::load(&path).unwrap(), expected);

    fs::write(
        &path,
        "version = 1\nshow_hidden = false\nfuture_field = 42\n",
    )
    .unwrap();
    let partial = config::load(&path).unwrap();
    assert!(!partial.show_hidden);
    assert_eq!(partial.theme, "classic");
}

#[test]
fn corrupt_config_is_preserved_and_does_not_block_startup() {
    let temporary = tempdir().unwrap();
    let path = temporary.path().join("config.toml");
    fs::write(&path, "this is not = [toml").unwrap();
    let loaded = config::load_or_default(&path);
    assert_eq!(loaded.config, Config::default());
    assert!(loaded.warning.is_some());
    let broken = loaded.broken_copy.unwrap();
    assert_eq!(fs::read_to_string(broken).unwrap(), "this is not = [toml");
}

#[test]
fn missing_last_path_falls_back_to_home_then_current() {
    let temporary = tempdir().unwrap();
    let current = temporary.path().join("current");
    let home = temporary.path().join("home");
    fs::create_dir_all(&current).unwrap();
    fs::create_dir_all(&home).unwrap();
    assert_eq!(
        config::resolve_start_path(
            Some(&temporary.path().join("missing")),
            Some(&home),
            &current
        ),
        home
    );
    assert_eq!(config::resolve_start_path(None, None, &current), current);
}

#[test]
fn m3_fragments_roundtrip_unicode_keymap_history_and_qcd() {
    let temporary = tempdir().unwrap();
    let path = temporary.path().join("config.toml");
    let mut value = Config::default();
    value
        .keymap
        .insert("refresh".to_string(), "Ctrl+L".to_string());
    value.mcd_history.push(temporary.path().join("한글"));
    value.qcd.push(QcdEntry {
        label: "즐겨찾기".to_string(),
        path: temporary.path().to_path_buf(),
        position: 0,
    });
    config::save_atomic(&path, &value).unwrap();
    assert_eq!(config::load(&path).unwrap(), value);
}

#[test]
fn atomic_save_reports_an_unwritable_parent_without_losing_config() {
    let temporary = tempdir().unwrap();
    let blocker = temporary.path().join("not-a-directory");
    fs::write(&blocker, "keep").unwrap();
    assert!(config::save_atomic(&blocker.join("config.toml"), &Config::default()).is_err());
    assert_eq!(fs::read_to_string(blocker).unwrap(), "keep");
}
