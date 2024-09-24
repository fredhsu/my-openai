use std::env;

#[derive(serde::Serialize, Debug)]
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
    fn create_chat_completion(&self) -> CreateChatCompletion {
        let system_message = Message {
            role: "system".to_string(),
            content: self.system.clone(),
        };
        let user_message = Message {
            role: "user".to_string(),
            content: self.user.clone(),
        };
        let messages = vec![system_message, user_message];
        CreateChatCompletion {
            messages,
            model: self.model.clone(),
            temperature: None,
            stream: None,
        }
    }
}

#[derive(serde::Serialize, Debug)]
struct CreateChatCompletion {
    messages: Vec<Message>,
    model: String,
    temperature: Option<f32>,
    stream: Option<bool>,
}

#[derive(serde::Serialize, Debug)]
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
#[derive(serde::Serialize, Debug)]
struct ChatUsage {
    completion_tokens: u32,
    prompt_tokens: u32,
    total_tokens: u32,
    completion_tokens_details: CompletionTokenDetails,
}
#[derive(serde::Serialize, Debug)]
struct CompletionTokenDetails {
    reasoning_tokens: u32,
}

#[derive(serde::Serialize, Debug)]
struct ChatCompletionChoice {
    finish_reason: String,
    index: u32,
    message: ChatCompletionMessage,
}
#[derive(serde::Serialize, Debug)]
struct ChatCompletionMessage {
    refusal: Option<String>,
    role: String,
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
    async fn call(
        &self,
        path: &str,
        payload: CreateChatCompletion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let resp = client
            .post(OPENAI_API_URL.to_string() + path)
            .bearer_auth(self.api_key.clone())
            .json(&payload)
            .send()
            .await?
            //.json::<HashMap<String, String>>()
            .text()
            .await?;

        println!("{resp:#?}");
        Ok(())
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
    let cc = simple.create_chat_completion();
    let resp = client.call("/chat/completions", cc).await?;
    dbg!(resp);

    Ok(())
}
