use async_broadcast::{Receiver, Sender};
use once_cell::sync::Lazy;

use crate::message::Message;

static QUEUE_OUT: Lazy<(Sender<Message>, Receiver<Message>)> = Lazy::new(|| {
    let (mut tx, rx) = async_broadcast::broadcast(10);
    tx.set_overflow(true);
    (tx, rx)
});

pub fn receiver() -> Receiver<Message> {
    QUEUE_OUT.1.clone()
}
pub fn broadcast(msg: Message) {
    if let Message::Error(e) = &msg {
        log::error!("{:?}", e);
    }
    let _ = QUEUE_OUT.0.try_broadcast(msg);
}
