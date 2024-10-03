use anyhow::{Context, Result};
use bytes::Bytes;
use core::str;
use futures_util::{stream, Stream, StreamExt, TryStreamExt};
use qdrant_client::qdrant::{query, QueryPointsBuilder, QueryResponse};
use qdrant_client::qdrant::{PointId, Query};
use qdrant_client::Qdrant;
use serde::{Deserialize, Serialize};
use std::{env, vec};

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
struct StreamingChunk {
    data: ChatCompletionChunk,
}
#[derive(Serialize, Deserialize, Debug)]
struct ChatCompletionChunk {
    id: String,
    choices: Vec<ChatCompletionChunkChoice>,
    created: u32,
    model: String,
    service_tier: Option<String>,
    system_fingerprint: String,
    object: String,
    usage: Option<ChatUsage>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ChatCompletionChunkChoice {
    delta: Delta,
    logprobs: Option<LogProbs>,
    finish_reason: Option<String>,
    index: u32,
}
#[derive(Serialize, Deserialize, Debug)]
struct Delta {
    content: Option<String>,
    tool_calls: Option<ToolCall>,
    refusal: Option<String>,
    role: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct LogProbs {
    // TODO: put proper structs
    content: Vec<String>,
    refusal: Vec<String>,
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
    organization: Option<String>,
    project: Option<String>,
}

const OPENAI_API_URL: &str = "https://api.openai.com/v1";

impl Client {
    fn new(api_key: String) -> Self {
        Client {
            api_key,
            organization: None,
            project: None,
        }
    }
    // TODO: refactor to use generic function
    async fn create_embedding(
        &self,
        payload: &EmbeddingRequest,
    ) -> Result<EmbeddingResponse, Box<dyn std::error::Error>> {
        let path = "/embeddings";
        let client = reqwest::Client::new();

        let resp = client
            .post(OPENAI_API_URL.to_string() + path)
            .bearer_auth(self.api_key.clone())
            .json(&payload)
            .send()
            .await?
            .json::<EmbeddingResponse>()
            .await?;

        Ok(resp)
    }

    // TODO: make generic function for requests
    async fn create_request(&self, path: &str, payload: &EmbeddingRequest) {}
    async fn create_chat_completion(
        &self,
        path: &str,
        payload: &ChatCompletionRequest,
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

    async fn create_stream_chat_completion(&self, path: &str, payload: ChatCompletionRequest) {
        fn parse_chat_completion_chunk(chunk: &str) -> Option<ChatCompletionChunk> {
            chunk
                .strip_prefix("data: ")
                .and_then(|d| serde_json::from_str::<ChatCompletionChunk>(d).ok())
        }
        // reference: https://platform.openai.com/docs/api-reference/streaming#chat/create-stream
        // https://html.spec.whatwg.org/multipage/server-sent-events.html#server-sent-events
        let client = reqwest::Client::new();

        let mut stream = client
            .post(OPENAI_API_URL.to_string() + path)
            .bearer_auth(self.api_key.clone())
            .json(&payload)
            .send()
            .await
            .unwrap()
            .bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item.unwrap();
            let chunk_strings = str::from_utf8(&chunk).unwrap();
            let chunks: Vec<&str> = chunk_strings.split("\n").collect();
            // println!("{chunks:#?}");
            let cc_chunks: Vec<ChatCompletionChunk> = chunks
                .iter()
                .filter(|&c| !c.is_empty() && !c.ends_with("[DONE]"))
                .filter_map(|c| parse_chat_completion_chunk(c))
                .collect();
            // let cc_chunks = chunks
            //     .iter()
            //     .filter_map(|c| {
            //         if c.is_empty() {
            //             return None;
            //         }
            //         if c.ends_with("[DONE]") {
            //             return None;
            //         }
            //         if let Some(d) = c.strip_prefix("data: ") {
            //             Some(serde_json::from_str::<ChatCompletionChunk>(d).unwrap())
            //         } else {
            //             None
            //         }
            //     })
            //     // .collect::<Vec<Option<ChatCompletionChunk>>>();
            //     .collect::<Vec<ChatCompletionChunk>>();
            //
            println!("{cc_chunks:#?}");
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}
impl EmbeddingRequest {
    fn new(input: &str, model: &str) -> Self {
        EmbeddingRequest {
            input: input.to_string(),
            model: model.to_string(),
            encoding_format: None,
            dimensions: None,
            user: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingResponse {
    object: Option<String>,
    data: Vec<Embedding>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
struct Embedding {
    index: u32,
    embedding: Vec<f32>,
    object: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    total_tokens: u32,
}

async fn query_vectorstore(query: &str) -> QueryResponse {
    // Use OpenAI embeddings to create embeddings vector for query
    let client = Qdrant::from_url("http://localhost:6333").build().unwrap();
    client
        .query(QueryPointsBuilder::new("dc1").query(Query::new_nearest(vec![0.1, 0.24])))
        .await
        .unwrap()
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
    let mut cc = simple.create_chat_completion_request();

    cc.stream = Some(true);
    client
        .create_stream_chat_completion("/chat/completions", cc)
        .await;

    let e = EmbeddingRequest::new(
        "The food was delicous and the waiter ...",
        "text-embedding-ada-002",
    );

    let embedding = client.create_embedding(&e).await?;
    println!("{embedding:#?}");
    Ok(())
}
