use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use futures::future::join_all;
use serde_json::Value;
use tokio::sync::RwLock;

pub use triangle::utils::events::Event;

pub type ListenerId = u64;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

struct Listener {
  id: ListenerId,
  event: &'static str,
  cb: Arc<dyn Fn(Value) -> BoxFuture + Send + Sync>,
  once: bool,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct Events {
  listeners: Arc<RwLock<Vec<Listener>>>,
}

impl Default for Events {
  fn default() -> Self {
    Self::new()
  }
}

impl Events {
  pub fn new() -> Self {
    Self {
      listeners: Arc::new(RwLock::new(Vec::new())),
    }
  }

  pub async fn on<T, F, Fut>(&self, cb: F) -> ListenerId
  where
    T: Event,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.add_listener::<T, F, Fut>(cb, false).await
  }

  pub async fn off(&self, id: ListenerId) {
    self.listeners.write().await.retain(|l| l.id != id);
  }

  pub async fn once<T, F, Fut>(&self, cb: F) -> ListenerId
  where
    T: Event,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.add_listener::<T, F, Fut>(cb, true).await
  }

  async fn add_listener<T, F, Fut>(&self, cb: F, once: bool) -> ListenerId
  where
    T: Event,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let cb = Arc::new(move |val: Value| -> BoxFuture {
      match serde_json::from_value::<T>(val) {
        Ok(event) => Box::pin(cb(event)),
        Err(e) => Box::pin(async move {
          tracing::error!("Failed to parse event {}: {}", T::NAME, e);
        }),
      }
    });
    self.listeners.write().await.push(Listener {
      id,
      event: T::NAME,
      cb,
      once,
    });
    id
  }

  pub async fn emit<T: Event>(&self, event: T) {
    let data = match serde_json::to_value(&event) {
      Ok(v) => v,
      Err(e) => {
        tracing::error!("Failed to serialize event {}: {}", T::NAME, e);
        return;
      }
    };

    let mut once_ids = Vec::new();
    let futures: Vec<BoxFuture> = {
      let listeners = self.listeners.read().await;
      listeners
        .iter()
        .filter(|l| l.event == T::NAME)
        .map(|l| {
          if l.once {
            once_ids.push(l.id);
          }
          (l.cb)(data.clone())
        })
        .collect()
    };

    join_all(futures).await;

    if !once_ids.is_empty() {
      self
        .listeners
        .write()
        .await
        .retain(|l| !once_ids.contains(&l.id));
    }
  }
}

static EVENTS: OnceLock<Events> = OnceLock::new();

pub mod msgs {
	triangle::event!(shutdown => Shutdown);
}

pub fn events() -> &'static Events {
  EVENTS.get_or_init(Events::new)
}
pub fn events() -> &'static Events {
  EVENTS.get_or_init(|| Events::new())
}
