use crate::{
    credentials,
    domain::{Engine, EngineKind, WorkingContext},
    error::{KeelError, Result},
};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: MessageResponse,
}

fn messages(context: &WorkingContext, prompt: &str) -> Vec<Message> {
    let sources = context
        .included
        .iter()
        .map(|item| {
            format!(
                "SOURCE: {}\nWHY INCLUDED: {}\n{}",
                item.name, item.reason, item.excerpt
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let system = format!(
        "You are the Engine operating inside Keel. Follow the objective and constraints. Use only supplied sources for project-specific facts. If sources are absent or insufficient, say so plainly. Never imply that omitted material was considered.\n\nOBJECTIVE:\n{}\n\nCONSTRAINTS:\n{}\n\nDECISIONS:\n{}\n\nINCLUDED SOURCES:\n{}",
        context.objective,
        context.constraints.join("\n"),
        context.decisions.join("\n"),
        sources
    );
    vec![
        Message {
            role: "system".into(),
            content: system,
        },
        Message {
            role: "user".into(),
            content: prompt.into(),
        },
    ]
}

pub async fn generate(engine: &Engine, context: &WorkingContext, prompt: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    let messages = messages(context, prompt);
    match engine.kind {
        EngineKind::SixtwelveMlx => generate_sixtwelve(engine, context, prompt).await,
        EngineKind::Ollama => {
            let url = format!("{}/api/chat", engine.endpoint.trim_end_matches('/'));
            let response = client
                .post(url)
                .json(&OllamaRequest {
                    model: engine.model.clone(),
                    messages,
                    stream: false,
                })
                .send()
                .await
                .map_err(|e| KeelError::Engine(e.to_string()))?;
            if !response.status().is_success() {
                return Err(KeelError::Engine(format!(
                    "Ollama returned {}",
                    response.status()
                )));
            }
            response
                .json::<OllamaResponse>()
                .await
                .map(|value| value.message.content)
                .map_err(|e| KeelError::Engine(e.to_string()))
        }
        EngineKind::OpenAiCompatible => {
            let url = format!("{}/chat/completions", engine.endpoint.trim_end_matches('/'));
            let mut request = client.post(url).json(&OpenAiRequest {
                model: engine.model.clone(),
                messages,
                stream: false,
            });
            if let Some(secret) = credentials::get(&engine.id.to_string())? {
                request = request.bearer_auth(secret);
            }
            let response = request
                .send()
                .await
                .map_err(|e| KeelError::Engine(e.to_string()))?;
            if !response.status().is_success() {
                return Err(KeelError::Engine(format!(
                    "Provider returned {}",
                    response.status()
                )));
            }
            response
                .json::<OpenAiResponse>()
                .await
                .map_err(|e| KeelError::Engine(e.to_string()))?
                .choices
                .into_iter()
                .next()
                .map(|choice| choice.message.content)
                .ok_or_else(|| KeelError::Engine("Provider returned no answer".into()))
        }
    }
}

pub async fn check(engine: &Engine) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    match engine.kind {
        EngineKind::SixtwelveMlx => {
            let root = std::path::Path::new(&engine.endpoint);
            let cli = root.join(".venv/bin/sixai");
            let model = match engine.model.as_str() {
                "writer" => root.join("models/bases/writer/qwen2.5-3b-instruct-4bit/config.json"),
                "code-7b" => {
                    root.join("models/bases/code/qwen2.5-coder-7b-instruct-4bit/config.json")
                }
                _ => return Err(KeelError::Engine("Unknown SIXTWELVE model profile".into())),
            };
            if cli.is_file() && model.is_file() {
                Ok(format!("Local {} model ready", engine.model))
            } else {
                Err(KeelError::Engine(
                    "Local SIXTWELVE runtime or model files are missing".into(),
                ))
            }
        }
        EngineKind::Ollama => {
            let url = format!("{}/api/tags", engine.endpoint.trim_end_matches('/'));
            let response = client.get(url).send().await.map_err(|_| {
                KeelError::Engine("Ollama is installed or configured but is not responding".into())
            })?;
            if response.status().is_success() {
                Ok("Engine ready".into())
            } else {
                Err(KeelError::Engine(format!(
                    "Ollama returned {}",
                    response.status()
                )))
            }
        }
        EngineKind::OpenAiCompatible => {
            Ok("Provider configured. A request will still require the send gate.".into())
        }
    }
}

#[derive(Deserialize)]
struct OllamaTags {
    models: Vec<OllamaTag>,
}

#[derive(Deserialize)]
struct OllamaTag {
    name: String,
    remote_host: Option<String>,
}

/// Local Ollama models only; cloud-hosted tags would send text off the machine.
pub async fn detect_ollama(endpoint: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    let tags = client
        .get(format!("{}/api/tags", endpoint.trim_end_matches('/')))
        .send()
        .await
        .map_err(|_| KeelError::Engine("Ollama is not running on this Mac".into()))?
        .json::<OllamaTags>()
        .await
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    Ok(local_models(tags))
}

fn local_models(tags: OllamaTags) -> Vec<String> {
    tags.models
        .into_iter()
        .filter(|tag| tag.remote_host.is_none() && !tag.name.contains("cloud"))
        .map(|tag| tag.name)
        .collect()
}

async fn generate_sixtwelve(
    engine: &Engine,
    context: &WorkingContext,
    prompt: &str,
) -> Result<String> {
    let root = std::path::Path::new(&engine.endpoint);
    let cli = root.join(".venv/bin/sixai");
    if !cli.is_file() {
        return Err(KeelError::Engine(format!(
            "SIXTWELVE runtime not found at {}",
            cli.display()
        )));
    }
    let rendered = messages(context, prompt)
        .into_iter()
        .map(|message| format!("{}:\n{}", message.role.to_uppercase(), message.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    let mut command = Command::new(cli);
    command.current_dir(root);
    match engine.model.as_str() {
        "writer" => {
            command.args([
                "context",
                "chat",
                "writer",
                &rendered,
                "-n",
                "512",
                "-k",
                "5",
                "--context-budget",
                "9000",
                "--root",
            ]);
        }
        "code-7b" => {
            command.args([
                "code", "chat", &rendered, "--base", "7b", "-n", "900", "--root",
            ]);
        }
        profile => {
            return Err(KeelError::Engine(format!(
                "Unknown SIXTWELVE profile: {profile}"
            )));
        }
    }
    command.arg(root);
    let output = command
        .output()
        .await
        .map_err(|error| KeelError::Engine(error.to_string()))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(KeelError::Engine(if detail.is_empty() {
            "Local model process failed".into()
        } else {
            detail
        }));
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Err(KeelError::Engine("Local model returned no answer".into()))
    } else {
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection_excludes_cloud_models() {
        let tags: OllamaTags = serde_json::from_str(
            r#"{"models":[{"name":"llama3.2:3b"},{"name":"gpt-oss:120b-cloud"},{"name":"kimi","remote_host":"https://ollama.com"}]}"#,
        )
        .unwrap();
        assert_eq!(local_models(tags), ["llama3.2:3b"]);
    }
}
