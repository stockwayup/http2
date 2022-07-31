use std::collections::HashMap;

use tokio::sync::mpsc::{self, Receiver, Sender};

pub struct Router {
    subscribers: HashMap<String, Sender<Vec<u8>>>,
    sub_sender: Sender<Sub>,
    sub_receiver: Receiver<Sub>,
    unsub_sender: Sender<String>,
    unsub_receiver: Receiver<String>,
    pub_sender: Sender<Event>,
    pub_receiver: Receiver<Event>,
}

#[derive(Debug)]
struct Sub {
    id: String,
    sender: Sender<Vec<u8>>,
}

#[derive(Debug)]
pub struct Event {
    id: String,
    data: Vec<u8>,
}

impl Router {
    pub fn new() -> Self {
        let (sub_tx, sub_rx) = mpsc::channel(128);
        let (unsub_tx, unsub_rx) = mpsc::channel(128);
        let (pub_tx, pub_rx) = mpsc::channel(128);

        Self {
            subscribers: Default::default(),
            sub_sender: sub_tx,
            sub_receiver: sub_rx,
            unsub_sender: unsub_tx,
            unsub_receiver: unsub_rx,
            pub_sender: pub_tx,
            pub_receiver: pub_rx,
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                sub = self.sub_receiver.recv() => {
                    let s = sub.unwrap();

                    self.subscribers.insert(s.id.to_string(), s.sender);
                }
                id = self.unsub_receiver.recv() => {
                    self.subscribers.remove(&id.unwrap());
                }
                event = self.pub_receiver.recv() => {
                    let e = event.unwrap();

                    match self.subscribers.get(&e.id) {
                        Some(sender) => sender.blocking_send(e.data),
                        None => Ok(()),
                    };
                }
            }
        }
    }

    pub fn subscribe(&self, id: String) -> Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::channel(128);

        self.sub_sender
            .blocking_send(Sub { id, sender: tx })
            .unwrap();

        rx
    }

    pub fn unsubscribe(&self, id: String) {
        self.unsub_sender.blocking_send(id).unwrap();
    }

    pub fn publish(&self, e: Event) {
        self.pub_sender.blocking_send(e).unwrap();
    }
}
