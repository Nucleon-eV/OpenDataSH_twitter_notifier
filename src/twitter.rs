use std::io::{stdin, stdout, Write};

use egg_mode::Token;
use tokio::runtime::current_thread::block_on_all;

use crate::config::Config;

#[derive(Debug)]
enum LoginStatus {
    LoggedOut,
    LoggedIn,
}

#[derive(Debug)]
pub struct Twitter {
    config: Config,
    token: Option<Token>,
    user_id: Option<u64>,
    screen_name: Option<String>,
    logged_in: LoginStatus,
}

impl Twitter {
    pub fn new(config: Config) -> Self {
        Twitter {
            config,
            token: None,
            user_id: None,
            screen_name: None,
            logged_in: LoginStatus::LoggedOut,
        }
    }

    pub fn login(&mut self) {
        let tokens = &self.config.tokens;

        // Prepare twitter auth
        let con_token =
            egg_mode::KeyPair::new(tokens.consumer_key.clone(), tokens.consumer_secret.clone());
        let request_token = block_on_all(egg_mode::request_token(&con_token, "oob")).unwrap();
        let auth_url = egg_mode::authorize_url(&request_token);

        // Print auth URL
        info!("Please Login using this URL: {}", auth_url);

        // Get user Input
        let mut verifier = String::new();
        print!("Please enter the auth verifier: ");
        let _ = stdout().flush();

        stdin()
            .read_line(&mut verifier)
            .expect("Did not enter a correct string");

        if let Some('\n') = verifier.chars().next_back() {
            verifier.pop();
        }
        if let Some('\r') = verifier.chars().next_back() {
            verifier.pop();
        }

        let (token, user_id, screen_name) =
            block_on_all(egg_mode::access_token(con_token, &request_token, verifier)).unwrap();
        self.token = Some(token);
        self.user_id = Some(user_id);
        self.screen_name = Some(screen_name);
        self.logged_in = LoginStatus::LoggedIn;
    }

    // TODO add the sending part

    // TODO add a way to listen on dms

    //let post = block_on_all(DraftTweet::new("Bot Test!").send(&token)).unwrap();

    // TODO broken
    /*let mut conversations = egg_mode::direct::conversations(&token);
    conversations = block_on_all(conversations.newest()).unwrap();

    for (id, convo) in &conversations.conversations {
        let user = block_on_all(egg_mode::user::show(id, &token)).unwrap();
        info!("Conversation with @{}", user.screen_name);
        for msg in convo {
            info!("<@{}> {}", msg.sender_screen_name, msg.text);
        }
    }*/
}
