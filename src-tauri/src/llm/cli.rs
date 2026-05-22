use super::{AtomizationResult, JudgeVerdict, LlmProvider, RawClaim, RawClaimKind, Stance};
use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const CLI_TIMEOUT: Duration = Duration::from_secs(180);
const SYSTEM_PATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";
const USER_BIN_DIRS: &[&str] = &[
    ".local/bin",
    ".npm-global/bin",
    ".bun/bin",
    ".cargo/bin",
    ".deno/bin",
];
const FALLBACK_BIN_DIRS: &[&str] = &[
    "/opt/homebrew/bin",
    "/usr/local/bin",
    "/usr/bin",
    "/bin",
    "/usr/sbin",
    "/sbin",
];

/// Spawns a user-configured shell command, pipes the prompt to its stdin, and
/// parses the JSON object it writes to stdout. Designed for local-CLI LLM
/// runners (Claude Code, codex, ollama run, aichat, llama-cli, …).
pub struct CliProvider {
    command: Vec<String>,
    locale: String,
}

impl CliProvider {
    pub fn new(command_line: &str, locale: String) -> AppResult<Self> {
        let parts = shlex::split(command_line).ok_or_else(|| {
            AppError::Invalid(format!("cannot parse CLI command: {command_line}"))
        })?;
        if parts.is_empty() {
            return Err(AppError::Invalid("empty CLI command".into()));
        }
        Ok(Self {
            command: parts,
            locale,
        })
    }

    fn run<'a>(
        &'a self,
        prompt: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<String>> + Send + 'a>> {
        Box::pin(async move {
            let runtime_path = cli_runtime_path();
            let program = resolve_program(&self.command[0], &runtime_path)
                .unwrap_or_else(|| PathBuf::from(&self.command[0]));
            let mut cmd = Command::new(&program);
            cmd.args(&self.command[1..])
                .env("PATH", &runtime_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = cmd
                .spawn()
                .map_err(|e| AppError::Other(format!("cli spawn '{}': {e}", self.command[0])))?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(prompt.as_bytes())
                    .await
                    .map_err(|e| AppError::Other(format!("cli stdin write: {e}")))?;
                stdin
                    .shutdown()
                    .await
                    .map_err(|e| AppError::Other(format!("cli stdin close: {e}")))?;
            }

            let output = timeout(CLI_TIMEOUT, child.wait_with_output())
                .await
                .map_err(|_| {
                    AppError::Other(format!("cli timeout after {}s", CLI_TIMEOUT.as_secs()))
                })?
                .map_err(|e| AppError::Other(format!("cli wait: {e}")))?;

            if !output.status.success() {
                return Err(AppError::Other(format!(
                    "cli '{}' exit {}: {}",
                    self.command[0],
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                )));
            }

            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        })
    }
}

fn cli_runtime_path() -> OsString {
    build_cli_path(home_dir().as_deref(), env::var_os("PATH").as_deref())
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn build_cli_path(home: Option<&Path>, inherited: Option<&OsStr>) -> OsString {
    let mut dirs = Vec::new();

    if let Some(inherited) = inherited {
        for path in env::split_paths(inherited) {
            push_unique_path(&mut dirs, path);
        }
    }

    if let Some(home) = home {
        for relative in USER_BIN_DIRS {
            push_unique_path(&mut dirs, home.join(relative));
        }
    }

    for fallback in FALLBACK_BIN_DIRS {
        push_unique_path(&mut dirs, PathBuf::from(fallback));
    }

    env::join_paths(dirs).unwrap_or_else(|_| OsString::from(SYSTEM_PATH))
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() || paths.iter().any(|candidate| candidate == &path) {
        return;
    }
    paths.push(path);
}

fn resolve_program(program: &str, runtime_path: &OsStr) -> Option<PathBuf> {
    let requested = Path::new(program);
    if requested.components().count() > 1 {
        return is_executable(requested).then(|| requested.to_path_buf());
    }

    env::split_paths(runtime_path)
        .map(|dir| dir.join(program))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.is_file()
        && path
            .metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[async_trait]
impl LlmProvider for CliProvider {
    async fn atomize(&self, input: &str) -> AppResult<AtomizationResult> {
        let system = crate::llm::prompts::atomize_prompt(
            &self.locale,
            crate::storage::settings_store::ProviderKind::Cli,
        );
        let prompt = format!("{system}\n\n=== INPUT ===\n{input}\n");
        let raw = self.run(prompt).await?;
        let parsed: RawAtomize = parse_cli_json_object(&raw, "atomize")?;
        parsed.into_result()
    }

    async fn judge(&self, claim: &str, source_text: &str) -> AppResult<JudgeVerdict> {
        let system = crate::llm::prompts::judge_prompt(
            &self.locale,
            crate::storage::settings_store::ProviderKind::Cli,
        );
        let user = match self.locale.as_str() {
            "cs" => format!(
                "Tvrzení:\n{claim}\n\nZdrojový text:\n{source_text}\n\nUrči stanovisko zdroje k tvrzení."
            ),
            _ => format!(
                "Claim:\n{claim}\n\nSource text:\n{source_text}\n\nDecide the source's stance toward the claim."
            ),
        };
        let prompt = format!("{system}\n\n=== INPUT ===\n{user}\n");
        let raw = self.run(prompt).await?;
        let parsed: RawJudge = parse_cli_json_object(&raw, "judge")?;
        parsed.into_verdict()
    }
}

#[derive(Debug, Deserialize)]
struct RawAtomize {
    claims: Vec<RawClaimRecord>,
    #[serde(default)]
    truncated: bool,
}

#[derive(Debug, Deserialize)]
struct RawClaimRecord {
    text: String,
    kind: String,
    #[serde(default)]
    reason: String,
}

impl RawAtomize {
    fn into_result(self) -> AppResult<AtomizationResult> {
        let mut claims = Vec::with_capacity(self.claims.len());
        for record in self.claims {
            let text = record.text.trim().to_string();
            if text.is_empty() {
                continue;
            }
            let kind = parse_kind(&record.kind)?;
            claims.push(RawClaim {
                text,
                kind,
                reason: record.reason.trim().to_string(),
            });
        }
        Ok(AtomizationResult {
            claims,
            truncated: self.truncated,
        })
    }
}

#[derive(Debug, Deserialize)]
struct RawJudge {
    stance: String,
    #[serde(default)]
    quote: String,
}

impl RawJudge {
    fn into_verdict(self) -> AppResult<JudgeVerdict> {
        let stance = match self.stance.trim().to_ascii_lowercase().as_str() {
            "supports" => Stance::Supports,
            "contradicts" => Stance::Contradicts,
            "mentions" => Stance::Mentions,
            other => {
                return Err(AppError::Other(format!(
                    "cli judge: unknown stance {other}"
                )))
            }
        };
        Ok(JudgeVerdict {
            stance,
            quote: self.quote.trim().to_string(),
        })
    }
}

fn parse_kind(kind: &str) -> AppResult<RawClaimKind> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "fact" => Ok(RawClaimKind::Fact),
        "inference" => Ok(RawClaimKind::Inference),
        "opinion" => Ok(RawClaimKind::Opinion),
        "contradiction" => Ok(RawClaimKind::Contradiction),
        other => Err(AppError::Other(format!("unknown claim kind: {other}"))),
    }
}

/// Tolerantly extracts the first balanced JSON object from a free-form CLI
/// output. Handles leading prose, trailing prose, and markdown code fences
/// (triple-backtick blocks optionally tagged with `json`). Returns `None` if
/// no balanced `{...}` is found.
pub(crate) fn extract_json_object(text: &str) -> Option<String> {
    // Strip code fences first if present.
    let cleaned = strip_json_fences(text);

    let bytes = cleaned.as_bytes();
    let mut start: Option<usize> = None;
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }

        match b {
            b'"' => in_string = true,
            b'{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        return Some(cleaned[s..=i].to_string());
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn strip_json_fences(text: &str) -> &str {
    text.trim()
        .trim_start_matches("```json")
        .trim_start_matches("```JSON")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
}

fn extract_jsonish_object(text: &str) -> Option<String> {
    let cleaned = strip_json_fences(text);
    let start = cleaned.find('{')?;
    let end = cleaned.rfind('}')?;
    (start <= end).then(|| cleaned[start..=end].to_string())
}

fn parse_cli_json_object<T: DeserializeOwned>(raw: &str, stage: &str) -> AppResult<T> {
    let json = extract_json_object(raw)
        .or_else(|| extract_jsonish_object(raw))
        .ok_or_else(|| AppError::Other(format!("cli {stage}: no JSON in output: {raw}")))?;

    serde_json::from_str(&json).or_else(|original_error| {
        let repaired = repair_unescaped_string_quotes(&json);
        if repaired == json {
            return Err(AppError::Other(format!(
                "cli {stage} JSON parse: {original_error}; raw={raw}"
            )));
        }

        serde_json::from_str(&repaired).map_err(|repair_error| {
            AppError::Other(format!(
                "cli {stage} JSON parse: {original_error}; repair failed: {repair_error}; raw={raw}"
            ))
        })
    })
}

fn repair_unescaped_string_quotes(json: &str) -> String {
    let mut repaired = String::with_capacity(json.len());
    let mut in_string = false;
    let mut escape = false;

    for (index, ch) in json.char_indices() {
        if !in_string {
            if ch == '"' {
                in_string = true;
            }
            repaired.push(ch);
            continue;
        }

        if escape {
            escape = false;
            repaired.push(ch);
            continue;
        }

        match ch {
            '\\' => {
                escape = true;
                repaired.push(ch);
            }
            '"' if quote_closes_json_string(json, index + ch.len_utf8()) => {
                in_string = false;
                repaired.push(ch);
            }
            '"' => repaired.push_str("\\\""),
            _ => repaired.push(ch),
        }
    }

    repaired
}

fn quote_closes_json_string(json: &str, after_quote: usize) -> bool {
    match json[after_quote..].chars().find(|ch| !ch.is_whitespace()) {
        Some(ch) => matches!(ch, ':' | ',' | '}' | ']'),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_pure_json() {
        let s = r#"{"claims":[],"truncated":false}"#;
        assert_eq!(extract_json_object(s).unwrap(), s);
    }

    #[test]
    fn extract_strips_fences() {
        let s = "```json\n{\"a\":1}\n```";
        assert_eq!(extract_json_object(s).unwrap(), "{\"a\":1}");
    }

    #[test]
    fn extract_handles_prose_around_json() {
        let s = "Here is the result:\n{\"a\":1}\nThanks!";
        assert_eq!(extract_json_object(s).unwrap(), "{\"a\":1}");
    }

    #[test]
    fn extract_handles_nested_objects() {
        let s = r#"prose {"outer": {"inner": "v"}} trailing"#;
        assert_eq!(
            extract_json_object(s).unwrap(),
            r#"{"outer": {"inner": "v"}}"#
        );
    }

    #[test]
    fn extract_handles_braces_in_strings() {
        let s = r#"x {"key": "value with } brace"} y"#;
        assert_eq!(
            extract_json_object(s).unwrap(),
            r#"{"key": "value with } brace"}"#
        );
    }

    #[test]
    fn extract_returns_none_when_no_object() {
        assert!(extract_json_object("no json here").is_none());
    }

    #[test]
    fn parse_kind_accepts_all_kinds() {
        assert_eq!(parse_kind("fact").unwrap(), RawClaimKind::Fact);
        assert_eq!(parse_kind("FACT").unwrap(), RawClaimKind::Fact);
        assert_eq!(parse_kind("inference").unwrap(), RawClaimKind::Inference);
        assert_eq!(parse_kind("opinion").unwrap(), RawClaimKind::Opinion);
        assert_eq!(
            parse_kind("contradiction").unwrap(),
            RawClaimKind::Contradiction
        );
    }

    #[test]
    fn parse_kind_rejects_unknown() {
        assert!(parse_kind("rumor").is_err());
    }

    #[test]
    fn raw_atomize_drops_empty_text_claims() {
        let raw = RawAtomize {
            claims: vec![
                RawClaimRecord {
                    text: "  ".into(),
                    kind: "fact".into(),
                    reason: "x".into(),
                },
                RawClaimRecord {
                    text: "Real".into(),
                    kind: "fact".into(),
                    reason: "x".into(),
                },
            ],
            truncated: false,
        };
        let result = raw.into_result().unwrap();
        assert_eq!(result.claims.len(), 1);
        assert_eq!(result.claims[0].text, "Real");
    }

    #[test]
    fn cli_provider_new_rejects_empty_command() {
        assert!(CliProvider::new("", "en".into()).is_err());
        assert!(CliProvider::new("   ", "en".into()).is_err());
    }

    #[test]
    fn cli_provider_new_parses_command_with_args() {
        let provider = CliProvider::new("ollama run llama3.2 --format json", "cs".into()).unwrap();
        assert_eq!(provider.command[0], "ollama");
        assert_eq!(provider.command.len(), 5);
    }

    #[test]
    fn cli_provider_new_respects_shell_quoting() {
        let provider =
            CliProvider::new(r#"my-llm --system "you are helpful""#, "en".into()).unwrap();
        assert_eq!(provider.command.len(), 3);
        assert_eq!(provider.command[2], "you are helpful");
    }

    #[cfg(unix)]
    #[test]
    fn resolve_program_finds_home_local_bin_when_inherited_path_is_gui_default() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let root =
            std::env::temp_dir().join(format!("druhy-nazor-cli-path-{}", std::process::id()));
        let home = root.join("home");
        let bin_dir = home.join(".local/bin");
        let program = bin_dir.join("claude");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(&program, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();

        let path = build_cli_path(
            Some(&home),
            Some(std::ffi::OsStr::new("/usr/bin:/bin:/usr/sbin:/sbin")),
        );

        assert_eq!(resolve_program("claude", &path).unwrap(), program);

        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn cli_atomize_via_fake_cat_returns_canned_json() {
        // Use a tiny shell pipeline that ignores stdin and prints a canned JSON.
        let canned = r#"{"claims":[{"text":"x","kind":"fact","reason":"r"}],"truncated":false}"#;
        let cmd = format!(
            r#"sh -c 'cat >/dev/null; printf %s "{}"'"#,
            canned.replace('"', r#"\""#)
        );
        let provider = CliProvider::new(&cmd, "en".into()).unwrap();
        let result = provider.atomize("ignored").await.unwrap();
        assert_eq!(result.claims.len(), 1);
        assert_eq!(result.claims[0].text, "x");
        assert!(!result.truncated);
    }

    #[tokio::test]
    async fn cli_judge_via_fake_cat_returns_canned_json() {
        let canned = r#"{"stance":"supports","quote":"q"}"#;
        let cmd = format!(
            r#"sh -c 'cat >/dev/null; printf %s "{}"'"#,
            canned.replace('"', r#"\""#)
        );
        let provider = CliProvider::new(&cmd, "en".into()).unwrap();
        let verdict = provider.judge("claim", "src").await.unwrap();
        assert_eq!(verdict.stance, Stance::Supports);
        assert_eq!(verdict.quote, "q");
    }

    #[tokio::test]
    async fn cli_atomize_tolerates_fenced_output() {
        // Single-line fenced output. `printf` interprets `\n` escape sequences.
        let cmd = r#"sh -c 'cat >/dev/null; printf "%s\n%s\n%s\n" "\`\`\`json" "{\"claims\":[],\"truncated\":false}" "\`\`\`"'"#;
        let provider = CliProvider::new(cmd, "en".into()).unwrap();
        let result = provider.atomize("ignored").await.unwrap();
        assert!(result.claims.is_empty());
    }

    #[tokio::test]
    async fn cli_atomize_surfaces_nonzero_exit_code() {
        let provider = CliProvider::new("sh -c 'exit 7'", "en".into()).unwrap();
        let err = provider.atomize("x").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("exit"), "unexpected error: {msg}");
    }

    #[tokio::test]
    async fn cli_atomize_returns_error_on_missing_json() {
        let cmd = "sh -c 'cat >/dev/null; printf %s \"no json here\"'";
        let provider = CliProvider::new(cmd, "en".into()).unwrap();
        let err = provider.atomize("x").await.unwrap_err();
        assert!(err.to_string().contains("no JSON"));
    }

    #[tokio::test]
    async fn cli_atomize_repairs_unescaped_quote_inside_claim_text() {
        let broken_json = r#"{"claims":[{"text":"kombinace: fine-tuning pro „jazykový cit" + RAG","kind":"fact","reason":"r"}],"truncated":false}"#;
        let cmd = format!(
            r#"sh -c 'cat >/dev/null; printf %s "{}"'"#,
            broken_json.replace('"', r#"\""#)
        );
        let provider = CliProvider::new(&cmd, "cs".into()).unwrap();

        let result = provider
            .atomize("kombinace: fine-tuning pro „jazykový cit\" + RAG")
            .await
            .unwrap();

        assert_eq!(result.claims.len(), 1);
        assert_eq!(
            result.claims[0].text,
            "kombinace: fine-tuning pro „jazykový cit\" + RAG"
        );
    }
}
