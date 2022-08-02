use std::collections::HashMap;

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, RwLock};

pub struct Router {
    state: State,
}

struct State {
    subscribers: RwLock<HashMap<String, Sender<Vec<u8>>>>,
    sub_ch: broadcast::Sender<Sub>,
    unsub_ch: broadcast::Sender<String>,
    pub_ch: broadcast::Sender<Event>,
}

impl State {
    pub fn new() -> Self {
        let (sub_tx, _) = broadcast::channel(128);
        let (unsub_tx, _) = broadcast::channel(128);
        let (pub_tx, _) = broadcast::channel(128);

        Self {
            subscribers: RwLock::new(Default::default()),
            sub_ch: sub_tx,
            unsub_ch: unsub_tx,
            pub_ch: pub_tx,
        }
    }
}

#[derive(Clone, Debug)]
struct Sub {
    id: String,
    sender: Sender<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct Event {
    id: String,
    data: Vec<u8>,
}

impl Event {
    pub fn new(id: String, data: Vec<u8>) -> Self {
        Self { id, data }
    }
}

impl Router {
    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    pub async fn run(&self) {
        let mut sub_receiver = self.state.sub_ch.subscribe();
        let mut unsub_receiver = self.state.unsub_ch.subscribe();
        let mut pub_receiver = self.state.pub_ch.subscribe();

        loop {
            tokio::select! {
                sub = sub_receiver.recv() => {
                    let s = sub.unwrap();

                    let mut map = self.state.subscribers.write().await;

                    map.insert(s.id.to_string(), s.sender);

                    drop(map)
                }
                id = unsub_receiver.recv() => {
                    let mut map = self.state.subscribers.write().await;

                    map.remove(&id.unwrap());

                    drop(map)
                }
                event = pub_receiver.recv() => {
                    let e = event.unwrap();

                    let map = self.state.subscribers.write().await;

                    match map.get(&e.id) {
                        Some(sender) => sender.send(e.data).await,
                        None => Ok(()),
                    };

                    drop(map)
                }
            }
        }
    }

    pub fn subscribe(&self, id: String) -> Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::channel(128);

        self.state.sub_ch.send(Sub { id, sender: tx }).unwrap();

        rx
    }

    pub fn unsubscribe(&self, id: String) {
        self.state.unsub_ch.send(id).unwrap();
    }

    pub fn publish(&self, e: Event) {
        self.state.pub_ch.send(e).unwrap();
    }
}
