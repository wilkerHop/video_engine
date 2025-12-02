use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_interstellar-triangulum"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Digital Artisan Video Engine"));
}

#[test]
fn test_cli_template_generation() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_interstellar-triangulum"));
    let assert = cmd
        .arg("template")
        .arg("explainer")
        .arg("--duration")
        .arg("30")
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("\"duration\": 30.0"));
}

#[test]
fn test_cli_validate_simple() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_interstellar-triangulum"));
    cmd.arg("validate")
        .arg("examples/simple.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validation complete"));
}

#[test]
fn test_cli_render_simple() {
    // Ensure output directory is clean
    let _ = fs::remove_dir_all("tests/output_test");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_interstellar-triangulum"));
    cmd.arg("render")
        .arg("tests/test_config.json")
        .arg("--output")
        .arg("tests/output_test")
        .arg("--renderer")
        .arg("native")
        .arg("--force-cpu")
        .assert()
        .success()
        .stdout(predicate::str::contains("Video created successfully"));

    // Check if output directory was created and contains files
    assert!(fs::metadata("tests/output_test").is_ok());
    assert!(fs::metadata("tests/output_test/frame_0.ppm").is_ok());

    // Clean up
    let _ = fs::remove_dir_all("tests/output_test");
}
