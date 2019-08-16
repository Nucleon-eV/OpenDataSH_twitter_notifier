use std::collections::HashSet;
use std::io::{stdin, stdout, Write};

use egg_mode::Token;
use egg_mode::tweet::DraftTweet;
use unicode_segmentation::UnicodeSegmentation;

use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum LoginStatus {
    LoggedOut,
    LoggedIn,
}

impl Default for LoginStatus {
    fn default() -> Self {
        LoginStatus::LoggedOut
    }
}

#[derive(Debug, Clone)]
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

    pub async fn login(&mut self) -> Result<(), egg_mode::error::Error> {
        let config = &self.config;
        let tokens = &config.tokens;

        // Prepare twitter auth
        let con_token =
            egg_mode::KeyPair::new(tokens.consumer_key.clone(), tokens.consumer_secret.clone());
        let request_token = egg_mode::request_token(&con_token, "oob").await?;
        let auth_url = egg_mode::authorize_url(&request_token);

        // Print auth URL
        info!("Please Login using this URL: {}", auth_url);

        // Get user Input
        let mut verifier = String::new();
        info!("Please enter the auth verifier: ");
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

        debug!("got input");

        let (token, user_id, screen_name) =
            egg_mode::access_token(con_token, &request_token, verifier).await?;
        debug!("got auth stuff");

        self.token = Some(token);
        self.user_id = Some(user_id);
        self.screen_name = Some(screen_name);
        self.logged_in = LoginStatus::LoggedIn;
        debug!("{:?}",self.status());
        Ok(())
    }
    pub fn status(&self) -> &LoginStatus {
        return &self.logged_in;
    }

    pub async fn post_changed_datasets(
        &self,
        added_datasets: HashSet<String>,
        removed_datasets: HashSet<String>,
    ) -> Result<(), egg_mode::error::Error> {
        if self.status().to_owned() == LoginStatus::LoggedOut {
            error!("NOT LOGGED IN TWITTER");
            return Ok(());
        }
        // TODO watch the key limit
        if !added_datasets.is_empty() {
            let added_text: Vec<String> =
                added_datasets.iter().map(|x| format!("- {}", x)).collect();
            let prefix = "Neue Datasets:\n{}\n#opendata #sh #datasets #open";
            let characters_prefix: Vec<&str> =
                UnicodeSegmentation::graphemes(prefix, true).collect::<Vec<&str>>();

            //for datasets in added_text

            let tweet_text = format!("{}{}", prefix, added_text.join("\n"));
            let tweet = DraftTweet::new(tweet_text);
            tweet
                .send(Option::as_ref(&self.token).unwrap())
                .await
                .expect("Failed to send tweet");
            return Ok(());
        }

        if !removed_datasets.is_empty() {
            let removed_text: Vec<String> = removed_datasets
                .iter()
                .map(|x| format!("- {}", x))
                .collect();

            let tweet_text = format!(
                "Entfernte Datasets:\n{}\n#opendata #sh #datasets #open",
                removed_text.join("\n")
            );
            let tweet = DraftTweet::new(tweet_text);
            tweet
                .send(Option::as_ref(&self.token).unwrap())
                .await
                .expect("Failed to send tweet");
            return Ok(());
        }
        return Ok(());
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
