use std::collections::HashMap;
use std::env;
use anyhow::{Error, Result};
use async_openai::Client;
use async_openai::types::{ChatCompletionFunctionsArgs, ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, Role};
use bytes::Bytes;
use serde_json::json;
use crate::whisper::create_chat_completion_request_msg;

pub async fn get_gpt_response(messages: Vec<ChatCompletionRequestMessage>) -> Result<ChatCompletionRequestMessage, Error> {
    let client = Client::new();

    let function = ChatCompletionFunctionsArgs::default()
        .name("leave_conversation")
        .description("The GPT AI can choose to call this function to leave the conversation whenever it appears finished, or if the user is unintelligible more than 3 times in a row.")
        .parameters(json!({"type": "object", "properties": {}}))
        .build()?;

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .max_tokens(120_u16)
        .messages(messages.clone())
        .functions(vec![function])
        .build()?;

    let resp = client.chat().create(request).await?;

    let resp_message = resp.choices.get(0).unwrap().message.clone();

    if let Some(function_call) = resp_message.function_call {
        if function_call.name == "leave_conversation" {
            return Ok(create_chat_completion_request_msg(
                "Goodbye!".to_string(),
                Role::System));
        }
    }

    let bot_string = resp_message.content.as_ref().unwrap().clone();

    let new_bot_message = create_chat_completion_request_msg(
        bot_string,
        Role::Assistant);

    return Ok(new_bot_message);
}

pub async fn text_to_speech(voice_id: &str, text: String) -> Result<Bytes, Error> {
    let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{}/stream", voice_id);

    let mut data = HashMap::new();
    data.insert("text", text);

    let api_key = env::var("ELEVEN_LABS_API_KEY").expect("ELEVEN_LABS_API_KEY must be set");


    let client = reqwest::Client::new();
    let res = client.post(&url)
        .header("Accept", "audio/mpeg")
        .header("Content-Type", "application/json")
        .header("xi-api-key", api_key)
        .json(&data)
        .send()
        .await?;

    return Ok(res.bytes().await?);
}
