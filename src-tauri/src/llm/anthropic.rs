use crate::error::{AppError, AppResult};
use crate::llm::{AtomizationResult, JudgeVerdict, LlmProvider, RawClaim, RawClaimKind, Stance};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

pub const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";
pub const API_VERSION: &str = "2023-06-01";
pub const ENDPOINT: &str = "https://api.anthropic.com/v1/messages";

pub const TOOL_NAME: &str = "submit_analysis";
pub const JUDGE_TOOL_NAME: &str = "submit_judgement";

#[derive(Debug, Serialize)]
pub(crate) struct Request<'a> {
    pub model: &'a str,
    pub max_tokens: u32,
    pub system: &'a str,
    pub tools: Vec<Value>,
    pub tool_choice: Value,
    pub messages: Vec<Message<'a>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Message<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Debug, Deserialize)]
struct Response {
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text {
        #[serde(rename = "text")]
        _text: String,
    },
    ToolUse {
        name: String,
        input: Value,
    },
}

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    locale: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String, locale: String) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|error| AppError::Other(format!("reqwest builder: {error}")))?;
        Ok(Self {
            client,
            api_key,
            model,
            locale,
        })
    }

    async fn post(&self, body: &Value) -> AppResult<String> {
        let response = self
            .client
            .post(ENDPOINT)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|error| AppError::Other(format!("anthropic http: {error}")))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| AppError::Other(format!("anthropic body: {error}")))?;

        if !status.is_success() {
            return Err(AppError::Other(format!("anthropic {status}: {text}")));
        }

        Ok(text)
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn atomize(&self, input: &str) -> AppResult<AtomizationResult> {
        let system = crate::llm::prompts::atomize_prompt(
            &self.locale,
            crate::storage::settings_store::ProviderKind::Anthropic,
        );
        let message = Message {
            role: "user",
            content: input,
        };
        let body = serde_json::to_value(build_atomize_request(&self.model, system, &message))?;
        let text = self.post(&body).await?;
        parse_atomize_response(&text)
    }

    async fn judge(&self, claim: &str, source_text: &str) -> AppResult<JudgeVerdict> {
        let system = crate::llm::prompts::judge_prompt(
            &self.locale,
            crate::storage::settings_store::ProviderKind::Anthropic,
        );
        let user_message = match self.locale.as_str() {
            "cs" => format!(
                "Tvrzení:\n{claim}\n\nZdrojový text:\n{source_text}\n\nUrči stanovisko zdroje k tvrzení."
            ),
            _ => format!(
                "Claim:\n{claim}\n\nSource text:\n{source_text}\n\nDecide the source's stance toward the claim."
            ),
        };
        let body = json!({
            "model": self.model,
            "max_tokens": 512,
            "system": system,
            "tools": [judge_tool_schema()],
            "tool_choice": {"type": "tool", "name": JUDGE_TOOL_NAME},
            "messages": [{"role": "user", "content": user_message}]
        });
        let text = self.post(&body).await?;
        parse_judge_response(&text)
    }
}

pub(crate) fn build_atomize_request<'a>(
    model: &'a str,
    system: &'a str,
    user_message: &'a Message<'a>,
) -> Request<'a> {
    Request {
        model,
        max_tokens: 4096,
        system,
        tools: vec![atomize_tool_schema()],
        tool_choice: json!({"type": "tool", "name": TOOL_NAME}),
        messages: vec![Message {
            role: user_message.role,
            content: user_message.content,
        }],
    }
}

pub(crate) fn atomize_tool_schema() -> Value {
    json!({
        "name": TOOL_NAME,
        "description": "Submit the atomized and classified claims for the analyzed AI response.",
        "input_schema": {
            "type": "object",
            "properties": {
                "claims": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "text": {"type": "string", "description": "Verbatim substring of the input."},
                            "kind": {"type": "string", "enum": ["fact", "inference", "opinion", "contradiction"]},
                            "reason": {"type": "string", "description": "One short sentence explaining the classification."}
                        },
                        "required": ["text", "kind", "reason"]
                    }
                },
                "truncated": {"type": "boolean"}
            },
            "required": ["claims", "truncated"]
        }
    })
}

pub(crate) fn judge_tool_schema() -> Value {
    json!({
        "name": JUDGE_TOOL_NAME,
        "description": "Submit a stance verdict for a claim given a source excerpt.",
        "input_schema": {
            "type": "object",
            "properties": {
                "stance": {"type": "string", "enum": ["supports", "contradicts", "mentions"]},
                "quote": {"type": "string", "description": "Short quote from the source supporting the verdict."}
            },
            "required": ["stance", "quote"]
        }
    })
}

pub(crate) fn parse_atomize_response(body: &str) -> AppResult<AtomizationResult> {
    let response: Response = serde_json::from_str(body)?;
    if !matches!(
        response.stop_reason.as_deref(),
        Some("tool_use" | "end_turn")
    ) {
        return Err(AppError::Other(format!(
            "anthropic returned unexpected stop_reason: {:?}",
            response.stop_reason
        )));
    }

    for block in response.content {
        if let ContentBlock::ToolUse { name, input } = block {
            if name == TOOL_NAME {
                return parse_atomize_input(&input);
            }
        }
    }

    Err(AppError::Other(
        "anthropic response missing tool_use".into(),
    ))
}

fn parse_atomize_input(value: &Value) -> AppResult<AtomizationResult> {
    let claims_value = value
        .get("claims")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Other("tool input missing claims[]".into()))?;
    let truncated = value
        .get("truncated")
        .and_then(Value::as_bool)
        .ok_or_else(|| AppError::Other("tool input missing truncated bool".into()))?;

    let mut claims = Vec::with_capacity(claims_value.len());
    for claim in claims_value {
        let text = claim
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::Other("claim missing text".into()))?
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }

        let kind = parse_kind(
            claim
                .get("kind")
                .and_then(Value::as_str)
                .ok_or_else(|| AppError::Other("claim missing kind".into()))?,
        )?;
        let reason = claim
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        claims.push(RawClaim { text, kind, reason });
    }

    Ok(AtomizationResult { claims, truncated })
}

pub(crate) fn parse_judge_response(body: &str) -> AppResult<JudgeVerdict> {
    let response: Response = serde_json::from_str(body)?;
    for block in response.content {
        if let ContentBlock::ToolUse { name, input } = block {
            if name == JUDGE_TOOL_NAME {
                let stance = input
                    .get("stance")
                    .and_then(Value::as_str)
                    .ok_or_else(|| AppError::Other("judge missing stance".into()))?;
                let quote = input
                    .get("quote")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                return Ok(JudgeVerdict {
                    stance: parse_stance(stance)?,
                    quote,
                });
            }
        }
    }

    Err(AppError::Other(
        "anthropic response missing judge tool_use".into(),
    ))
}

fn parse_kind(value: &str) -> AppResult<RawClaimKind> {
    match value {
        "fact" => Ok(RawClaimKind::Fact),
        "inference" => Ok(RawClaimKind::Inference),
        "opinion" => Ok(RawClaimKind::Opinion),
        "contradiction" => Ok(RawClaimKind::Contradiction),
        other => Err(AppError::Other(format!("unknown claim kind: {other}"))),
    }
}

fn parse_stance(value: &str) -> AppResult<Stance> {
    match value {
        "supports" => Ok(Stance::Supports),
        "contradicts" => Ok(Stance::Contradicts),
        "mentions" => Ok(Stance::Mentions),
        other => Err(AppError::Other(format!("unknown stance: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_atomize_request_with_tool_choice() {
        let message = Message {
            role: "user",
            content: "input",
        };
        let request = build_atomize_request(DEFAULT_MODEL, "system", &message);
        assert_eq!(request.model, DEFAULT_MODEL);
        assert_eq!(request.max_tokens, 4096);
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.tool_choice["name"], TOOL_NAME);
    }

    #[test]
    fn parses_typical_atomize_response() {
        let body = r#"{
            "content": [
                {"type": "tool_use", "name": "submit_analysis", "input": {
                    "claims": [
                        {"text": "Karel IV. se narodil v roce 1316", "kind": "fact", "reason": "Historické datum."},
                        {"text": "byl podle mě nejlepší král", "kind": "opinion", "reason": "Subjektivní hodnocení."}
                    ],
                    "truncated": false
                }}
            ],
            "stop_reason": "tool_use"
        }"#;
        let result = parse_atomize_response(body).unwrap();
        assert_eq!(result.claims.len(), 2);
        assert!(!result.truncated);
        assert_eq!(result.claims[0].kind, RawClaimKind::Fact);
        assert_eq!(result.claims[1].kind, RawClaimKind::Opinion);
    }

    #[test]
    fn rejects_response_without_tool_use() {
        let body = r#"{"content": [{"type": "text", "text": "hello"}], "stop_reason": "end_turn"}"#;
        assert!(parse_atomize_response(body).is_err());
    }

    #[test]
    fn drops_empty_text_claims() {
        let body = r#"{
            "content": [
                {"type": "tool_use", "name": "submit_analysis", "input": {
                    "claims": [
                        {"text": "  ", "kind": "fact", "reason": "x"},
                        {"text": "Real claim", "kind": "fact", "reason": "x"}
                    ],
                    "truncated": false
                }}
            ],
            "stop_reason": "tool_use"
        }"#;
        let result = parse_atomize_response(body).unwrap();
        assert_eq!(result.claims.len(), 1);
        assert_eq!(result.claims[0].text, "Real claim");
    }

    #[test]
    fn rejects_unknown_kind() {
        let body = r#"{
            "content": [
                {"type": "tool_use", "name": "submit_analysis", "input": {
                    "claims": [{"text": "x", "kind": "lies", "reason": "y"}],
                    "truncated": false
                }}
            ],
            "stop_reason": "tool_use"
        }"#;
        assert!(parse_atomize_response(body).is_err());
    }

    #[test]
    fn parses_judge_response() {
        let body = r#"{
            "content": [
                {"type": "tool_use", "name": "submit_judgement", "input": {
                    "stance": "supports", "quote": "Wikipedia confirms 1316."
                }}
            ],
            "stop_reason": "tool_use"
        }"#;
        let verdict = parse_judge_response(body).unwrap();
        assert_eq!(verdict.stance, Stance::Supports);
        assert_eq!(verdict.quote, "Wikipedia confirms 1316.");
    }
}
