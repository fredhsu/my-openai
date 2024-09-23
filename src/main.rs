use std::collections::HashMap;
use std::env;

#[derive(serde::Serialize, Debug)]
struct Message <'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug)]
struct Simple <'a>{
    system: &'a str,
    user: &'a str,
    model: &'a str,
}
impl Simple <'_>{
    fn create_chat_completion(&self) -> CreateChatCompletion {
        let systemMessage = Message{role:"system", content: self.system};
        let userMessage = Message{role:"user", content: self.user};
        let messages = vec![systemMessage, userMessage];
        CreateChatCompletion{
            messages,
            model : self.model,
            temperature: None,
            stream: None,
        }
    }
}

#[derive(serde::Serialize, Debug)]
struct CreateChatCompletion <'a>{
    messages: Vec<Message<'a>>,
    model: &'a str,
    temperature: Option<f32>,
    stream: Option<bool>,
}

#[derive(serde::Serialize, Debug)]
struct ChatCompletion<'a>{
    id: &'a str,
    object: &'a str,
    created: u32,
    model: &'a str,
    system_fingerprint: &'a str,
    choices: Vec<ChatCompletionChoice>,
    service_tier: Option<String>,
    usage: ChatUsage,
}
#[derive(serde::Serialize, Debug)]
struct ChatUsage{
    completion_tokens: u32,
    prompt_tokens: u32,
    total_tokens: u32,
    completion_tokens_details: CompletionTokenDetails,
}
#[derive(serde::Serialize, Debug)]
struct CompletionTokenDetails{
    reasoning_tokens: u32,
}

#[derive(serde::Serialize, Debug)]
struct ChatCompletionChoice{
    finish_reason: String,
    index: u32,
    message: ChatCompletionMessage,
}
#[derive(serde::Serialize, Debug)]
struct ChatCompletionMessage{
    refusal: Option<String>,
    role: String,
}


#[derive(Debug)]
struct Client {
    api_key: String,
}

const OPENAI_API_URL:&str = "https://api.openai.com/v1";

impl Client {
    fn new(api_key: String) -> Self {
        Client {
            api_key
        }
    }
    async fn call(&self, path: &str, payload: CreateChatCompletion<'_>) -> Result<(),Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        
        let resp = client.post(OPENAI_API_URL.to_string() + path)
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

    let simple = Simple{
        system: "You are a helpful assistant.",
        user: &format!("Say hello to {name}!"),
        model,
    };
    let cc = simple.create_chat_completion();
    let resp = client.call("/chat/completions", cc).await?;
    dbg!(resp);


    Ok(())
}
