#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use std::{env, io};

use async_openai::{
    types::{CreateCompletionRequestArgs},
    Client,
};

fn main() {
    dotenv().ok();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_completion])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


#[tauri::command]
async fn get_completion(name: &str) -> Result<String, String> {
    let openai_api_key = env::var("OPENAI_API_KEY").map_err(|err| err.to_string())?;
    let client = Client::new();

    let request = CreateCompletionRequestArgs::default()
        .model("text-davinci-003")
        .prompt("Tell me a joke about the universe")
        .max_tokens(40_u16)
        .build()
        .map_err(|err| err.to_string())?;

    let response = client.completions().create(request).await.map_err(|err| err.to_string())?;

    Ok(format!("Hello, {}!", response.choices[0].text))
}

