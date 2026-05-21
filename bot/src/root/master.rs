use std::sync::Arc;

use parking_lot::Mutex;
use triangle::{
  Client, ClientOptions,
  classes::ribbon,
  types::{
    events::recv,
    social::{Detail, Status},
  },
  utils::api::core::ApiError,
};

use crate::{
  game::{Bot, Target},
  lib::{
    env::env,
    events::{events, msgs},
  },
};

pub struct Master {
  client: Client,
  children: Arc<Mutex<Vec<Arc<Bot>>>>,
}

impl Master {
  pub async fn new() -> Result<Self, ApiError> {
    let c = Master {
      client: Client::new(ClientOptions {
        game: None,
        ribbon: Some(ribbon::OptionalParams {
          options: Some(ribbon::Options {
            logging: ribbon::LoggingLevel::Error,
            ..Default::default()
          }),
          ..Default::default()
        }),
        social: None,
        token: triangle::Credentials::Token(env().token.clone()),
        user_agent: None,
      })
      .await?,
      children: Arc::new(Mutex::new(Vec::new())),
    };

    c.init().await;

    Ok(c)
  }

  async fn init(&self) {
    self
      .client
      .social
      .set_status(Status::Online, Detail::Menus)
      .await;

    let c = self.children.clone();
    let cc = self.client.clone();

    self.client.on::<recv::social::Invite>(async move |invite| {
      match Bot::new(Target::Join(invite.roomid.clone())).await {
        Ok(bot) => {
          c.lock().push(bot);
        }
        Err(e) => {
          let message = e.to_string();
          cc.social
            .dm(invite.sender, format!("Failed to join room: {}", message))
            .await
            .ok();
        }
      };
    });

    let mut client = self.client.clone();

    self.client.on::<recv::client::Dead>(async move |_| {
      loop {
        if client.reconnect().await.is_ok() {
          break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
      }
    });

    // events.on("shutdown", async () => {
    //   this.#client.social.status("offline");
    //   await this.destroy();
    // });
    let client = self.client.clone();

    events()
      .on::<msgs::Shutdown>(move |_| {
        let client = client.clone();
        async move {
          client
            .social
            .set_status(Status::Offline, Detail::Menus)
            .await;
          client.destroy().await;
        }
      })
      .await;
  }
}
