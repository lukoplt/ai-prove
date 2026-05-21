use super::{AtomizationResult, JudgeVerdict, LlmProvider, RawClaim, RawClaimKind, Stance};
use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use serde::Deserialize;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const CLI_TIMEOUT: Duration = Duration::from_secs(180);

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
            let mut cmd = Command::new(&self.command[0]);
            cmd.args(&self.command[1..])
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

#[async_trait]
impl LlmProvider for CliProvider {
    async fn atomize(&self, input: &str) -> AppResult<AtomizationResult> {
        let system = crate::llm::prompts::atomize_prompt(
            &self.locale,
            crate::storage::settings_store::ProviderKind::Cli,
        );
        let prompt = format!("{system}\n\n=== INPUT ===\n{input}\n");
        let raw = self.run(prompt).await?;
        let json = extract_json_object(&raw)
            .ok_or_else(|| AppError::Other(format!("cli atomize: no JSON in output: {raw}")))?;
        let parsed: RawAtomize = serde_json::from_str(&json)
            .map_err(|e| AppError::Other(format!("cli atomize JSON parse: {e}; raw={raw}")))?;
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
        let json = extract_json_object(&raw)
            .ok_or_else(|| AppError::Other(format!("cli judge: no JSON in output: {raw}")))?;
        let parsed: RawJudge = serde_json::from_str(&json)
            .map_err(|e| AppError::Other(format!("cli judge JSON parse: {e}; raw={raw}")))?;
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
    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```JSON")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

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
}
