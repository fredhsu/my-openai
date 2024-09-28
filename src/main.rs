use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug)]
struct Simple {
    system: String,
    user: String,
    model: String,
}
impl Simple {
    fn create_chat_completion_request(&self) -> ChatCompletionRequest {
        let system_message = Message {
            role: "system".to_string(),
            content: self.system.clone(),
        };
        let user_message = Message {
            role: "user".to_string(),
            content: self.user.clone(),
        };
        let messages = vec![system_message, user_message];
        ChatCompletionRequest {
            messages,
            model: self.model.clone(),
            temperature: None,
            stream: None,
        }
    }
}

#[derive(Serialize, Debug)]
struct ChatCompletionRequest {
    messages: Vec<Message>,
    model: String,
    temperature: Option<f32>,
    stream: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatCompletion {
    id: String,
    object: String,
    created: u32,
    model: String,
    system_fingerprint: String,
    choices: Vec<ChatCompletionChoice>,
    service_tier: Option<String>,
    usage: ChatUsage,
}
#[derive(Serialize, Deserialize, Debug)]
struct ChatUsage {
    completion_tokens: u32,
    prompt_tokens: u32,
    total_tokens: u32,
    completion_tokens_details: CompletionTokenDetails,
}
#[derive(Serialize, Deserialize, Debug)]
struct CompletionTokenDetails {
    reasoning_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatCompletionChoice {
    finish_reason: String,
    index: u32,
    message: ChatCompletionMessage,
}
#[derive(Serialize, Deserialize, Debug)]
struct ChatCompletionMessage {
    content: Option<String>,
    refusal: Option<String>,
    role: String,
    tool_calls: Option<ToolCall>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolCallFunction,
    role: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ToolCallFunction {
    name: String,
    arguments: String,
}

#[derive(Debug)]
struct Client {
    api_key: String,
}

const OPENAI_API_URL: &str = "https://api.openai.com/v1";

impl Client {
    fn new(api_key: String) -> Self {
        Client { api_key }
    }
    async fn create_chat_completion(
        &self,
        path: &str,
        payload: ChatCompletionRequest,
    ) -> Result<ChatCompletion, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let resp = client
            .post(OPENAI_API_URL.to_string() + path)
            .bearer_auth(self.api_key.clone())
            .json(&payload)
            .send()
            .await?
            //.json::<HashMap<String, String>>()
            .json::<ChatCompletion>()
            .await?;

        Ok(resp)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = "gpt-4o-mini";
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = Client::new(api_key);
    let name = "Fred";

    let simple = Simple {
        system: "You are a helpful assistant.".to_string(),
        user: format!("Say hello to {name}!"),
        model: model.to_string(),
    };
    let cc = simple.create_chat_completion_request();
    let resp = client
        .create_chat_completion("/chat/completions", cc)
        .await?;
    dbg!(resp);

    Ok(())
}
