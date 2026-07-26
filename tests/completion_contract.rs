use std::path::Path;
use std::process::{Command, Output};

const BASIC_FIXTURE: &str = "tests/fixtures/basic/Makefile";
const PHONY_FILTER_FIXTURE: &str = "tests/fixtures/phony-filter/Makefile";

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
    assert!(stdout.contains("new\tcreate new post"));
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
fn lists_declared_arguments_next_to_targets() {
    let output = mate(&["-f", BASIC_FIXTURE, "--list"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("build"));
    assert!(stdout.contains("new <title>"));
    assert!(stdout.contains("release <version> [target=all]  publish release"));
    assert!(stdout.contains("test"));

    let description_columns = stdout
        .lines()
        .filter_map(|line| {
            line.find("build project")
                .or_else(|| line.find("create new post"))
                .or_else(|| line.find("publish release"))
                .or_else(|| line.find("run tests"))
        })
        .collect::<Vec<_>>();
    assert_eq!(description_columns, vec![32, 32, 32, 32]);
}

#[test]
fn prefers_phony_targets_over_file_targets() {
    let output = mate(&["-f", PHONY_FILTER_FIXTURE, "--complete", "--", ""]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("release\tpublish release"));
    assert!(stdout.contains("test\trun tests"));
    assert!(!stdout.contains("build/output.txt"));
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
    assert!(stdout.contains("--makefile-syntax\texplain Makefile metadata conventions"));
    assert!(stdout.contains("--llm\tprint guidance for coding agents and LLMs"));
    assert!(stdout.contains("--shell-completion\tprint shell completion glue"));
    assert!(stdout.contains("--version\tshow installed version"));
}

#[test]
fn prints_makefile_syntax_guidance() {
    let output = mate(&["--makefile-syntax"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("# mate Makefile syntax"));
    assert!(stdout.contains("# args: version target?=all"));
    assert!(stdout.contains("make release VERSION=1.2.3 TARGET=all"));
    assert!(stdout.contains("mate new \"Post title\""));
}

#[test]
fn prints_llm_guidance() {
    let output = mate(&["--llm"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("# mate guidance for coding agents and LLMs"));
    assert!(stdout.contains("Use `mate --list`"));
    assert!(stdout.contains("mate idea \"Rough thought\""));
    assert!(stdout.contains("plain make"));
}

#[test]
fn prints_the_installed_version() {
    let output = mate(&["--version"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "mate 0.1.14\n");
}

#[test]
fn maps_positional_arguments_to_make_variables() {
    let output = mate(&["-f", BASIC_FIXTURE, "release", "1.2.3"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "VERSION=1.2.3 TARGET=all\n");
}

#[test]
fn preserves_quoted_positional_arguments_with_spaces() {
    let output = mate(&["-f", BASIC_FIXTURE, "new", "Post title"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "TITLE=Post title\n");
}

#[test]
fn errors_for_missing_required_arguments() {
    let output = mate(&["-f", BASIC_FIXTURE, "release"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("target 'release' requires argument 'version'"));
}
