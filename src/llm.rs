use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct OpenAiResponse {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    message: Message,
}

pub fn get_chat_completion(api_key: &str, user_message: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();

    let request_body = OpenAiRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_message.to_string(),
            },
        ],
    };

    let body_json = serde_json::to_string(&request_body).unwrap();

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .body(body_json)
        .send()?;

    if !response.status().is_success() {
        return Err("Error response from open ai".into());
    }

    let response_body: OpenAiResponse = serde_json::from_str(&response.text()?)?;

    if let Some(choice) = response_body.choices.into_iter().next() {
        Ok(choice.message.content)
    } else {
        Err("No choices found in response".into())
    }
}
