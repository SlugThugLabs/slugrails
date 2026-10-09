use slugrails::cli::init::initialize_project;
use tempfile::tempdir;

#[test]
fn test_init_creates_planning_dir_and_contract() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a mock src/ folder with a couple of modules
    std::fs::create_dir_all(root.join("src").join("auth")).unwrap();
    std::fs::create_dir_all(root.join("src").join("billing")).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let msg = initialize_project(root).unwrap();
    assert!(msg.contains("Initialized architecture contract"));

    let arch_path = root.join(".planning").join("ARCHITECTURE.md");
    assert!(arch_path.exists());

    let content = std::fs::read_to_string(&arch_path).unwrap();
    assert!(content.contains("\"auth\""));
    assert!(content.contains("\"billing\""));
    assert!(content.contains("\"languages/rust/no-unwrap\""));
    assert!(content.contains("\"complexity/source-limits\""));
}

#[test]
fn test_init_does_not_overwrite_existing_contract() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let plan_dir = root.join(".planning");
    std::fs::create_dir_all(&plan_dir).unwrap();
    let arch_path = plan_dir.join("ARCHITECTURE.md");
    std::fs::write(&arch_path, "custom existing content").unwrap();

    let msg = initialize_project(root).unwrap();
    assert!(msg.contains("already exists"));

    // Verify existing content wasn't overwritten
    let content = std::fs::read_to_string(&arch_path).unwrap();
    assert_eq!(content, "custom existing content");
}
