use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MAKEFILES: &[&str] = &["GNUmakefile", "makefile", "Makefile"];
const ARG_DIRECTIVE: &str = "args:";
const DESCRIPTION_MARKER: &str = "##";

#[derive(Debug)]
pub enum MateError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    NoMakefile,
    MakeFailedToStart(io::Error),
    UnknownTarget(String),
    MissingArgument {
        target: String,
        argument: String,
    },
    TooManyArguments {
        target: String,
        expected: usize,
        got: usize,
    },
    InvalidInvocation(String),
}

impl std::fmt::Display for MateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            Self::NoMakefile => write!(formatter, "no Makefile found in the current directory"),
            Self::MakeFailedToStart(source) => write!(formatter, "failed to start make: {source}"),
            Self::UnknownTarget(target) => write!(formatter, "unknown target '{target}'"),
            Self::MissingArgument { target, argument } => {
                write!(
                    formatter,
                    "target '{target}' requires argument '{argument}'"
                )
            }
            Self::TooManyArguments {
                target,
                expected,
                got,
            } => write!(
                formatter,
                "target '{target}' expects {expected} positional argument(s), got {got}"
            ),
            Self::InvalidInvocation(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for MateError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<Argument>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Argument {
    pub name: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct MakeInvocation {
    pub target: String,
    pub variables: Vec<String>,
}

pub fn run(args: Vec<String>) -> Result<i32, MateError> {
    let options = Options::parse(args)?;

    if options.help {
        print_help();
        return Ok(0);
    }

    if let Some(shell) = options.shell_completion {
        print_shell_completion(&shell)?;
        return Ok(0);
    }

    if let Some(words) = &options.complete_words {
        if let Some(completions) = cli_option_completions(words) {
            print_lines(&completions);
            return Ok(0);
        }
    }

    let makefile = options.makefile.map_or_else(find_makefile, Ok)?;
    let targets = load_targets(&makefile)?;

    if options.list {
        print_targets(&targets);
        return Ok(0);
    }

    if let Some(words) = options.complete_words {
        print_completions(&targets, &words);
        return Ok(0);
    }

    let Some(target) = options.target else {
        print_targets(&targets);
        return Ok(0);
    };

    let invocation = build_invocation(&targets, &target, &options.target_args)?;
    run_make(&makefile, &invocation)
}

#[derive(Debug, Default)]
struct Options {
    help: bool,
    list: bool,
    makefile: Option<PathBuf>,
    shell_completion: Option<String>,
    complete_words: Option<Vec<String>>,
    target: Option<String>,
    target_args: Vec<String>,
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self, MateError> {
        let mut options = Self::default();
        let mut index = 0;

        while index < args.len() {
            match args[index].as_str() {
                "-h" | "--help" => options.help = true,
                "-l" | "--list" => options.list = true,
                "-f" | "--makefile" => {
                    index += 1;
                    let Some(path) = args.get(index) else {
                        return Err(MateError::InvalidInvocation(
                            "--makefile requires a path".to_string(),
                        ));
                    };
                    options.makefile = Some(PathBuf::from(path));
                }
                "--complete" => {
                    index += 1;
                    if args.get(index).is_some_and(|word| word == "--") {
                        index += 1;
                    }
                    options.complete_words = Some(args[index..].to_vec());
                    return Ok(options);
                }
                "--shell-completion" => {
                    index += 1;
                    let Some(shell) = args.get(index) else {
                        return Err(MateError::InvalidInvocation(
                            "--shell-completion requires bash, zsh, or fish".to_string(),
                        ));
                    };
                    options.shell_completion = Some(shell.clone());
                }
                "--" => {
                    index += 1;
                    if let Some(target) = args.get(index) {
                        options.target = Some(target.clone());
                        options.target_args = args[index + 1..].to_vec();
                    }
                    return Ok(options);
                }
                word if word.starts_with('-') => {
                    return Err(MateError::InvalidInvocation(format!(
                        "unknown option '{word}'"
                    )));
                }
                word => {
                    options.target = Some(word.to_string());
                    options.target_args = args[index + 1..].to_vec();
                    return Ok(options);
                }
            }
            index += 1;
        }

        Ok(options)
    }
}

fn print_help() {
    println!(
        "mate - easier make\n\nUSAGE:\n    mate [OPTIONS] [TARGET] [ARGS...]\n\nOPTIONS:\n    -f, --makefile PATH             Use a specific Makefile\n    -l, --list                      List known targets\n        --complete -- ...           Print completion candidates\n        --shell-completion SHELL    Print static completion glue for bash, zsh, or fish\n    -h, --help                      Show this help\n\nARGUMENTS:\n    TARGET                          Make target to run\n    ARGS                            Positional task arguments declared with '# args:'"
    );
}

fn print_shell_completion(shell: &str) -> Result<(), MateError> {
    match shell {
        "bash" => println!(
            r#"_mate()
{{
    local IFS=$'\n'
    COMPREPLY=( $(mate --complete -- "${{COMP_WORDS[@]:1}}") )
}}
complete -F _mate mate"#
        ),
        "zsh" => println!(
            r#"#compdef mate
_mate() {{
    local -a candidates
    candidates=(${{(f)"$(mate --complete -- ${{words[2,-1]}})"}})
    _describe 'mate target' candidates
}}
compdef _mate mate"#
        ),
        "fish" => println!(
            r#"function __mate_complete
    mate --complete -- (commandline -opc)[2..-1]
end
complete -c mate -f -a '(__mate_complete)'"#
        ),
        _ => {
            return Err(MateError::InvalidInvocation(format!(
                "unsupported shell '{shell}'; expected bash, zsh, or fish"
            )));
        }
    }
    Ok(())
}

fn find_makefile() -> Result<PathBuf, MateError> {
    for candidate in MAKEFILES {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(MateError::NoMakefile)
}

pub fn load_targets(makefile: &Path) -> Result<BTreeMap<String, Target>, MateError> {
    let metadata = parse_makefile_metadata(makefile)?;
    let discovered = discover_make_targets(makefile);
    let target_names = if discovered.is_empty() {
        metadata.keys().cloned().collect()
    } else {
        discovered
    };

    let mut targets = BTreeMap::new();
    for name in target_names {
        let metadata_target = metadata.get(&name);
        targets.insert(
            name.clone(),
            Target {
                name,
                description: metadata_target.and_then(|target| target.description.clone()),
                arguments: metadata_target.map_or_else(Vec::new, |target| target.arguments.clone()),
            },
        );
    }
    Ok(targets)
}

fn parse_makefile_metadata(makefile: &Path) -> Result<BTreeMap<String, Target>, MateError> {
    let content = fs::read_to_string(makefile).map_err(|source| MateError::Io {
        path: makefile.to_path_buf(),
        source,
    })?;
    Ok(parse_metadata(&content))
}

pub fn parse_metadata(content: &str) -> BTreeMap<String, Target> {
    let mut targets = BTreeMap::new();
    let mut pending_arguments = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(comment) = trimmed.strip_prefix('#') {
            let comment = comment.trim();
            if let Some(arguments) = comment.strip_prefix(ARG_DIRECTIVE) {
                pending_arguments = parse_arguments(arguments.trim());
            }
            continue;
        }

        if trimmed.starts_with('.') {
            pending_arguments.clear();
            continue;
        }

        let Some((target_part, rest)) = trimmed.split_once(':') else {
            pending_arguments.clear();
            continue;
        };

        if target_part.contains('=') || target_part.contains('%') || target_part.trim().is_empty() {
            pending_arguments.clear();
            continue;
        }

        let description = rest
            .split_once(DESCRIPTION_MARKER)
            .map(|(_, description)| description.trim().to_string())
            .filter(|description| !description.is_empty());

        for name in target_part.split_whitespace() {
            targets.insert(
                name.to_string(),
                Target {
                    name: name.to_string(),
                    description: description.clone(),
                    arguments: pending_arguments.clone(),
                },
            );
        }
        pending_arguments.clear();
    }

    targets
}

fn parse_arguments(arguments: &str) -> Vec<Argument> {
    arguments
        .split_whitespace()
        .filter_map(|token| {
            let (raw_name, default) = token
                .split_once('=')
                .map_or((token, None), |(name, value)| {
                    (name, Some(value.to_string()))
                });
            let (name, optional_marker) = raw_name
                .strip_suffix('?')
                .map_or((raw_name, false), |name| (name, true));
            if name.is_empty() {
                return None;
            }
            Some(Argument {
                name: name.to_string(),
                required: !optional_marker && default.is_none(),
                default,
            })
        })
        .collect()
}

fn discover_make_targets(makefile: &Path) -> BTreeSet<String> {
    let Ok(output) = Command::new("make")
        .arg("-pRrq")
        .arg("-f")
        .arg(makefile)
        .arg(":")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    else {
        return BTreeSet::new();
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_make_database_targets(&stdout)
}

pub fn parse_make_database_targets(database: &str) -> BTreeSet<String> {
    let mut targets = BTreeSet::new();
    let mut in_files_section = false;

    for line in database.lines() {
        if line == "# Files" {
            in_files_section = true;
            continue;
        }
        if line.starts_with("# Finished Make data base") {
            break;
        }
        if !in_files_section || line.is_empty() || line.starts_with('#') || line.starts_with('\t') {
            continue;
        }

        let Some((target_part, _)) = line.split_once(':') else {
            continue;
        };
        if target_part.contains('=') || target_part.contains('%') || target_part.trim().is_empty() {
            continue;
        }

        for target in target_part.split_whitespace() {
            if should_include_target(target) {
                targets.insert(target.to_string());
            }
        }
    }

    targets
}

fn should_include_target(target: &str) -> bool {
    !target.starts_with('.')
        && !target.contains('/')
        && !target.contains('%')
        && !target.ends_with(".mk")
        && target != "Makefile"
        && target != "makefile"
        && target != "GNUmakefile"
}

pub fn build_invocation(
    targets: &BTreeMap<String, Target>,
    target_name: &str,
    raw_arguments: &[String],
) -> Result<MakeInvocation, MateError> {
    let Some(target) = targets.get(target_name) else {
        return Err(MateError::UnknownTarget(target_name.to_string()));
    };

    let mut positional_arguments = Vec::new();
    let mut variables = Vec::new();
    for argument in raw_arguments {
        if argument.contains('=') {
            variables.push(argument.clone());
        } else {
            positional_arguments.push(argument.clone());
        }
    }

    if positional_arguments.len() > target.arguments.len() {
        return Err(MateError::TooManyArguments {
            target: target.name.clone(),
            expected: target.arguments.len(),
            got: positional_arguments.len(),
        });
    }

    for (index, declaration) in target.arguments.iter().enumerate() {
        if let Some(value) = positional_arguments.get(index) {
            variables.push(format!("{}={value}", make_variable_name(&declaration.name)));
        } else if let Some(default) = &declaration.default {
            variables.push(format!(
                "{}={default}",
                make_variable_name(&declaration.name)
            ));
        } else if declaration.required {
            return Err(MateError::MissingArgument {
                target: target.name.clone(),
                argument: declaration.name.clone(),
            });
        }
    }

    Ok(MakeInvocation {
        target: target.name.clone(),
        variables,
    })
}

fn make_variable_name(argument: &str) -> String {
    argument.replace('-', "_").to_ascii_uppercase()
}

fn print_targets(targets: &BTreeMap<String, Target>) {
    for target in targets.values() {
        if let Some(description) = &target.description {
            println!("{}\t{}", target.name, description);
        } else {
            println!("{}", target.name);
        }
    }
}

fn print_completions(targets: &BTreeMap<String, Target>, words: &[String]) {
    print_lines(&completion_candidates(targets, words));
}

fn completion_candidates(targets: &BTreeMap<String, Target>, words: &[String]) -> Vec<String> {
    if let Some(completions) = cli_option_completions(words) {
        return completions;
    }

    let words = completion_words(words);
    let Some(target_word) = words.first() else {
        return target_lines(targets);
    };

    if words.len() == 1 && !targets.contains_key(*target_word) {
        return targets
            .values()
            .filter(|target| target.name.starts_with(target_word))
            .map(target_line)
            .collect();
    }

    let Some(target) = targets.get(*target_word) else {
        return Vec::new();
    };
    let supplied = words.len().saturating_sub(1);
    target
        .arguments
        .get(supplied)
        .map(|argument| vec![format!("<{}>", argument.name)])
        .unwrap_or_default()
}

fn cli_option_completions(words: &[String]) -> Option<Vec<String>> {
    let words = completion_words(words);
    let current = words.last().copied().unwrap_or_default();

    if matches!(words.first(), Some(&"--shell-completion")) {
        return Some(
            ["bash", "fish", "zsh"]
                .into_iter()
                .filter(|shell| shell.starts_with(current))
                .map(str::to_string)
                .collect(),
        );
    }

    if current.starts_with('-') {
        return Some(
            [
                "--help\tshow help",
                "--list\tlist known targets",
                "--makefile\tuse a specific Makefile",
                "--shell-completion\tprint shell completion glue",
            ]
            .into_iter()
            .filter(|option| option.starts_with(current))
            .map(str::to_string)
            .collect(),
        );
    }

    None
}

fn completion_words(words: &[String]) -> Vec<&str> {
    words
        .iter()
        .map(String::as_str)
        .filter(|word| *word != "mate")
        .collect()
}

fn target_lines(targets: &BTreeMap<String, Target>) -> Vec<String> {
    targets.values().map(target_line).collect()
}

fn target_line(target: &Target) -> String {
    if let Some(description) = &target.description {
        format!("{}\t{}", target.name, description)
    } else {
        target.name.clone()
    }
}

fn print_lines(lines: &[String]) {
    for line in lines {
        println!("{line}");
    }
}

fn run_make(makefile: &Path, invocation: &MakeInvocation) -> Result<i32, MateError> {
    let status = Command::new("make")
        .arg("-f")
        .arg(makefile)
        .arg(&invocation.target)
        .args(invocation.variables.iter().map(OsString::from))
        .status()
        .map_err(MateError::MakeFailedToStart)?;

    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_descriptions_and_arguments() {
        let metadata = parse_metadata(
            "# args: version target?=all\nrelease: ## publish a release\n\t./release $(VERSION) $(TARGET)\n\ntest: ## run tests\n\tcargo test\n",
        );

        assert_eq!(
            metadata["release"].description.as_deref(),
            Some("publish a release")
        );
        assert_eq!(metadata["release"].arguments.len(), 2);
        assert_eq!(metadata["release"].arguments[0].name, "version");
        assert!(metadata["release"].arguments[0].required);
        assert_eq!(
            metadata["release"].arguments[1].default.as_deref(),
            Some("all")
        );
        assert_eq!(metadata["test"].description.as_deref(), Some("run tests"));
    }

    #[test]
    fn builds_make_variables_from_positional_arguments() {
        let metadata = parse_metadata("# args: version target?=all\nrelease:\n");
        let invocation = build_invocation(&metadata, "release", &["1.2.3".to_string()]).unwrap();

        assert_eq!(
            invocation,
            MakeInvocation {
                target: "release".to_string(),
                variables: vec!["VERSION=1.2.3".to_string(), "TARGET=all".to_string()],
            }
        );
    }

    #[test]
    fn reports_missing_required_arguments() {
        let metadata = parse_metadata("# args: version\nrelease:\n");
        let error = build_invocation(&metadata, "release", &[]).unwrap_err();

        assert!(matches!(error, MateError::MissingArgument { .. }));
    }

    #[test]
    fn parses_make_database_targets() {
        let targets = parse_make_database_targets(
            "# Files\nall:\n#  Implicit rule search has not been done.\n%.o: %.c\n.PHONY: all test\ntest:\nMakefile:\n# Finished Make data base\n",
        );

        assert!(targets.contains("all"));
        assert!(targets.contains("test"));
        assert!(!targets.contains(".PHONY"));
        assert!(!targets.contains("%.o"));
        assert!(!targets.contains("Makefile"));
    }

    #[test]
    fn completes_long_options_without_makefile_metadata() {
        let completions = cli_option_completions(&["--".to_string()]).unwrap();

        assert!(completions.iter().any(|item| item.starts_with("--list")));
        assert!(completions.iter().any(|item| item.starts_with("--help")));
        assert!(completions
            .iter()
            .any(|item| item.starts_with("--shell-completion")));
    }

    #[test]
    fn completes_shell_completion_values() {
        let completions =
            cli_option_completions(&["--shell-completion".to_string(), "z".to_string()]).unwrap();

        assert_eq!(completions, vec!["zsh".to_string()]);
    }
}
