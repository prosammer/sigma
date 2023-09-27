use anyhow::Result;
use tts::*;

#[cfg(target_os = "macos")]
use cocoa_foundation::base::id;
#[cfg(target_os = "macos")]
use cocoa_foundation::foundation::NSDefaultRunLoopMode;
#[cfg(target_os = "macos")]
use cocoa_foundation::foundation::NSRunLoop;
#[cfg(target_os = "macos")]
use objc::class;
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use std::sync::mpsc;
use tts::*;
use tauri::AppHandle;
use crate::stores::get_from_store;

pub fn speak_string<S: AsRef<str>>(text: S) -> Result<()> {
    let mut tts = Tts::default()?;
    let (tx, rx) = mpsc::channel();

    tts.on_utterance_end(Some(Box::new(move |_| {
        tx.send(()).unwrap();
    })))?;

    // tts.speak accepts only &str
    tts.speak(text.as_ref(), false)?;
    // TODO: Try commenting this out to see if I need it - tauri might include NSRunLoop
    #[cfg(target_os = "macos")]
    {
        let run_loop: id = unsafe { NSRunLoop::currentRunLoop() };
        unsafe {
            let date: id = msg_send![class!(NSDate), distantFuture];
            let _: () = msg_send![run_loop, runMode:NSDefaultRunLoopMode beforeDate:date];
        }
    }
    rx.recv().unwrap();
    Ok(())
}

pub async fn initial_speech(handle: AppHandle) {
    println!("Starting initial_speech");
    let user_first_name = get_from_store(handle, "userFirstName");
    let initial_speech = match user_first_name {
        Some(s) => format!("Good morning {}!", s),
        None => "Good morning!".to_string(),
    };
    speak_string(&initial_speech);
    println!("Finished initial_speech");
}
