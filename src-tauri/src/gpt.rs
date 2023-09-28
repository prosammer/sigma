use anyhow::{Error, Result};
use async_openai::Client;
use async_openai::types::{ChatCompletionFunctionsArgs, ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role};
use serde_json::json;
use tauri::AppHandle;
use crate::stores::get_from_store;

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


pub async fn messages_setup(handle: AppHandle) -> Vec<ChatCompletionRequestMessage> {
    let system_message_content = "You are an AI personal routine trainer. You greet the user in the morning, then go through the user-provided morning routine checklist and ensure that the user completes each task on the list in order. Make sure to keep your tone positive, but it is vital that the user completes each task - do not allow them to 'skip' tasks. The user uses speech-to-text to communicate, so some of their messages may be incorrect - if some text seems out of place, please ignore it. If the users sentence makes no sense in the context, tell them you don't understand and ask them to repeat themselves. If you receive any text like [SILENCE] or [MUSIC] please respond with - I didn't catch that. The following message is the prompt the user provided - their morning checklist. Call the leave_conversation function when the user has completed their morning routine, or whenever the AI would normally say goodbye";
    let system_message = create_chat_completion_request_msg(system_message_content.to_string(), Role::System);

    let user_prompt_content = get_from_store(handle, "userPrompt").unwrap_or("".to_string());
    let user_prompt_message = create_chat_completion_request_msg(user_prompt_content, Role::System);

    return vec![system_message, user_prompt_message]
}

