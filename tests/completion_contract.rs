use std::path::Path;
use std::process::{Command, Output};

const BASIC_FIXTURE: &str = "tests/fixtures/basic/Makefile";

fn mate(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mate"))
        .args(args)
        .output()
        .expect("mate command should start")
}

fn mate_in(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mate"))
        .current_dir(directory)
        .args(args)
        .output()
        .expect("mate command should start")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

#[test]
fn completes_targets_from_fixture_makefile() {
    let output = mate(&["-f", BASIC_FIXTURE, "--complete", "--", ""]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("build\tbuild project"));
    assert!(stdout.contains("release\tpublish release"));
    assert!(stdout.contains("test\trun tests"));
    assert!(!stdout.contains("Cargo.toml"));
}

#[test]
fn filters_target_prefixes() {
    let output = mate(&["-f", BASIC_FIXTURE, "--complete", "--", "rel"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "release\tpublish release\n");
}

#[test]
fn completes_declared_argument_placeholders() {
    let output = mate(&["-f", BASIC_FIXTURE, "--complete", "--", "release", ""]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "<version>\n");
}

#[test]
fn completes_cli_options_without_a_makefile() {
    let temporary =
        std::env::temp_dir().join(format!("make-mate-no-makefile-{}", std::process::id()));
    std::fs::create_dir_all(&temporary).expect("temporary directory should be created");

    let output = mate_in(&temporary, &["--complete", "--", "--"]);
    std::fs::remove_dir_all(&temporary).expect("temporary directory should be removed");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("--help\tshow help"));
    assert!(stdout.contains("--list\tlist known targets"));
    assert!(stdout.contains("--makefile\tuse a specific Makefile"));
    assert!(stdout.contains("--shell-completion\tprint shell completion glue"));
}

#[test]
fn maps_positional_arguments_to_make_variables() {
    let output = mate(&["-f", BASIC_FIXTURE, "release", "1.2.3"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "VERSION=1.2.3 TARGET=all\n");
}

#[test]
fn errors_for_missing_required_arguments() {
    let output = mate(&["-f", BASIC_FIXTURE, "release"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("target 'release' requires argument 'version'"));
}
