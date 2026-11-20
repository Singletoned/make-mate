use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BASIC_FIXTURE_DIR: &str = "tests/fixtures/basic";

fn mate_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mate"))
}

fn mate_bin_dir() -> PathBuf {
    mate_path()
        .parent()
        .expect("mate binary should have a parent directory")
        .to_path_buf()
}

fn output_text(output: &Output) -> (String, String) {
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8");
    (stdout, stderr)
}

fn shell_path() -> String {
    let existing = std::env::var("PATH").expect("PATH should be set");
    format!("{}:{existing}", mate_bin_dir().display())
}

fn write_stub(shell: &str) -> PathBuf {
    let output = Command::new(mate_path())
        .args(["--shell-completion", shell])
        .output()
        .expect("mate should print completion glue");
    assert!(output.status.success());

    let path = std::env::temp_dir().join(format!(
        "make-mate-{shell}-completion-{}.sh",
        std::process::id()
    ));
    std::fs::write(&path, output.stdout).expect("completion stub should be written");
    path
}

#[test]
fn bash_stub_inserts_option_without_description() {
    let stub = write_stub("bash");
    let script = format!(
        r#"source "{}"
COMP_WORDS=(mate --)
COMP_CWORD=1
_mate
printf '%s\n' "${{COMPREPLY[@]}}"
"#,
        stub.display()
    );

    let output = Command::new("bash")
        .arg("-c")
        .arg(script)
        .env("PATH", shell_path())
        .output()
        .expect("bash should run completion smoke test");
    let (stdout, stderr) = output_text(&output);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stdout.lines().any(|line| line == "--list"));
    assert!(
        !stdout.contains('\t'),
        "inserted values must not contain descriptions: {stdout:?}"
    );
}

#[test]
fn bash_stub_completes_target_prefix_without_description() {
    let stub = write_stub("bash");
    let script = format!(
        r#"source "{}"
COMP_WORDS=(mate rel)
COMP_CWORD=1
_mate
printf '%s\n' "${{COMPREPLY[@]}}"
"#,
        stub.display()
    );

    let output = Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(BASIC_FIXTURE_DIR)
        .env("PATH", shell_path())
        .output()
        .expect("bash should run completion smoke test");
    let (stdout, stderr) = output_text(&output);

    assert!(output.status.success(), "stderr: {stderr}");
    assert_eq!(stdout, "release\n");
}

#[test]
fn bash_stub_does_not_reinsert_exact_option() {
    let stub = write_stub("bash");
    let script = format!(
        r#"source "{}"
COMP_WORDS=(mate --help)
COMP_CWORD=1
_mate
printf '%s\n' "${{COMPREPLY[@]}}"
"#,
        stub.display()
    );

    let output = Command::new("bash")
        .arg("-c")
        .arg(script)
        .env("PATH", shell_path())
        .output()
        .expect("bash should run completion smoke test");
    let (stdout, stderr) = output_text(&output);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(
        stdout.trim().is_empty(),
        "expected no completions, got {stdout:?}"
    );
}

#[test]
fn zsh_stub_inserts_option_without_description() {
    let stub = write_stub("zsh");
    let script = zsh_harness(&stub, "words=(mate --)");

    let output = Command::new("zsh")
        .arg("-f")
        .arg("-c")
        .arg(script)
        .env("PATH", shell_path())
        .output()
        .expect("zsh should be installed for completion smoke tests");
    let (stdout, stderr) = output_text(&output);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(
        stdout
            .lines()
            .any(|line| line == "--list:list known targets")
    );
    assert!(
        !stdout.contains('\t'),
        "zsh candidates must use colon descriptions, not tabs: {stdout:?}"
    );
}

#[test]
fn zsh_stub_completes_target_prefix_without_description() {
    let stub = write_stub("zsh");
    let script = zsh_harness(&stub, "words=(mate rel)");

    let output = Command::new("zsh")
        .arg("-f")
        .arg("-c")
        .arg(script)
        .current_dir(BASIC_FIXTURE_DIR)
        .env("PATH", shell_path())
        .output()
        .expect("zsh should be installed for completion smoke tests");
    let (stdout, stderr) = output_text(&output);

    assert!(output.status.success(), "stderr: {stderr}");
    assert_eq!(stdout, "release:publish release\n");
}

#[test]
fn zsh_stub_does_not_reinsert_exact_option() {
    let stub = write_stub("zsh");
    let script = zsh_harness(&stub, "words=(mate --help)");

    let output = Command::new("zsh")
        .arg("-f")
        .arg("-c")
        .arg(script)
        .env("PATH", shell_path())
        .output()
        .expect("zsh should be installed for completion smoke tests");
    let (stdout, stderr) = output_text(&output);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(
        stdout.trim().is_empty(),
        "expected no completions, got {stdout:?}"
    );
}

fn zsh_harness(stub: &Path, words_assignment: &str) -> String {
    format!(
        r#"compdef() {{ :; }}
_describe() {{
    local candidate_array_name="${{@[-1]}}"
    eval "print -rl -- \"\${{${{candidate_array_name}}[@]}}\""
}}
source "{}"
{}
_mate
"#,
        stub.display(),
        words_assignment
    )
}
