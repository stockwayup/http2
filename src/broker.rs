use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, Notify, RwLock};

pub struct Broker {
    state: State,
}

struct State {
    subscribers: RwLock<HashMap<String, Sender<Event>>>,
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
    sender: Sender<Event>,
}

#[derive(Clone, Debug)]
pub struct Event {
    id: String,
    pub data: Vec<u8>,
    pub code: String,
}

impl Event {
    pub fn new(id: String, data: Vec<u8>, code: String) -> Self {
        Self { id, data, code }
    }
}

impl Broker {
    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    pub async fn run(&self, notify: Arc<Notify>) {
        let mut sub_receiver = self.state.sub_ch.subscribe();
        let mut unsub_receiver = self.state.unsub_ch.subscribe();
        let mut pub_receiver: broadcast::Receiver<Event> = self.state.pub_ch.subscribe();

        loop {
            tokio::select! {
                sub = sub_receiver.recv() => {
                    let s = sub.expect("sub channel closed");

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
                    let e = event.expect("pub channel closed");

                    let map = self.state.subscribers.write().await;

                    match map.get(&e.id) {
                        Some(sender) => sender.send(e).await,
                        None => Ok(()),
                    };

                    drop(map)
                }
                _ = notify.notified() => {
                    log::info!("router received shutdown signal");
                    break
                },
            }
        }

        // send all remaining responses
        if !pub_receiver.is_empty() {
            while !pub_receiver.is_empty() {
                tokio::select! {
                    event = pub_receiver.recv() => {
                        let e = event.expect("pub channel closed");

                        let map = self.state.subscribers.write().await;

                        match map.get(&e.id) {
                            Some(sender) => sender.send(e).await,
                            None => Ok(()),
                        };

                        drop(map)
                    }
                }
            }
        }
    }

    pub fn subscribe(&self, id: String) -> Receiver<Event> {
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
