use super::*;

#[test]
fn test_random_hex_generation() {
    let hex16 = generate_random_hex(16);
    let hex32 = generate_random_hex(32);
    assert_eq!(hex16.len(), 32);
    assert_eq!(hex32.len(), 64);
    assert_ne!(hex16, generate_random_hex(16));
}

#[test]
fn test_config_serialization_roundtrip() {
    let mut cfg = Config::default();
    cfg.add_or_update_device(
        "dev-123".into(),
        "Pixel 7".into(),
        "tok-abc".into(),
        Some(1),
    );

    let toml_str = toml::to_string_pretty(&cfg).unwrap();
    let loaded: Config = toml::from_str(&toml_str).unwrap();

    assert_eq!(cfg.pc_id, loaded.pc_id);
    assert_eq!(loaded.trusted_devices.len(), 1);
    assert!(loaded.is_trusted("dev-123", "tok-abc"));
    assert!(!loaded.is_trusted("dev-123", "wrong-tok"));
}

#[test]
fn test_forget_device() {
    let mut cfg = Config::default();
    cfg.add_or_update_device(
        "dev-123".into(),
        "Pixel 7".into(),
        "tok-abc".into(),
        Some(1),
    );
    assert!(cfg.forget_device("dev-123"));
    assert!(!cfg.is_trusted("dev-123", "tok-abc"));
}

#[test]
fn test_old_config_dir_moves_once() {
    let root = std::env::temp_dir().join(format!("owlmic_move_test_{}", generate_random_hex(8)));
    let old = root.join(OLD_DIR_NAME);
    let new = root.join("Owlmic");
    fs::create_dir_all(old.join("logs")).unwrap();
    fs::write(old.join("config.toml"), "pc_name = \"Desk\"").unwrap();

    move_old_dir(&new);
    assert!(new.join("config.toml").exists());
    assert!(new.join("logs").is_dir());
    assert!(!old.exists());

    // Never over settings Owlmic already has.
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("config.toml"), "pc_name = \"Old\"").unwrap();
    move_old_dir(&new);
    assert_eq!(
        fs::read_to_string(new.join("config.toml")).unwrap(),
        "pc_name = \"Desk\""
    );
    assert!(old.exists());

    let _ = fs::remove_dir_all(root);
}
