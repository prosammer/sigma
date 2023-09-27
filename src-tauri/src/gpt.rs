use anyhow::{Error, Result};
use async_openai::Client;
use async_openai::types::{ChatCompletionFunctionsArgs, ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role};
use serde_json::json;

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


pub fn create_chat_completion_request_msg(content: String, role: Role) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessageArgs::default()
        .content(content)
        .role(role)
        .build()
        .unwrap()
}

