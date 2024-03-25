use std::net::SocketAddr;

use async_std::channel::{bounded, Receiver, Sender};
use tide::{Redirect, Request, Response};

use crate::message::{InMsg, OutMsg};

pub trait Gateway {
    async fn recv(&self) -> anyhow::Result<InMsg>;
    async fn send(&self, out: OutMsg) -> anyhow::Result<()>;
}

pub struct HttpGateway {
    queue_in: Receiver<InMsg>,
    queue_out: Sender<OutMsg<String>>,
    queue_in_myself: Sender<InMsg>,
}

impl HttpGateway {
    pub fn with_socket(socket: SocketAddr) -> Self {
        let queue_in = bounded::<InMsg>(1000);
        let queue_out = bounded::<OutMsg<String>>(1000);
        let queue_in_myself = queue_in.0.clone();
        let mut server = tide::with_state((queue_in.0, queue_out.1));

        server.at("/").get(Redirect::new("/static/index.html"));
        server.at("/static").serve_dir("static/").unwrap();
        server
            .at("/push")
            .post(|mut req: Request<(Sender<InMsg>, _)>| async move {
                if let Ok(msg) = req.body_json::<InMsg>().await {
                    let sender = req.state().0.clone();
                    sender.send(msg).await.ok();
                    Ok(Response::builder(200).build())
                } else {
                    Ok(Response::builder(400).build())
                }
            });
        server
            .at("/wait")
            .get(|req: Request<(_, Receiver<OutMsg<String>>)>| async move {
                let msg = req.state().1.recv().await?;
                Ok(Response::builder(200)
                    .body(tide::Body::from_json(&msg)?)
                    .build())
            });
        async_std::task::spawn(server.listen(socket));
        Self {
            queue_in: queue_in.1,
            queue_out: queue_out.0,
            queue_in_myself,
        }
    }
    pub async fn send(&self, msg: OutMsg<String>) {
        log::info!("Sending {:?}", msg);
        self.queue_out.send(msg).await.ok();
    }
    pub async fn send_myself(&self, msg: InMsg) {
        log::info!("Sending myself {:?}", msg);
        self.queue_in_myself.send(msg).await.ok();
    }
    pub async fn recv(&self) -> InMsg {
        let msg = self.queue_in.recv().await.unwrap();
        log::info!("Receiving {:?}", msg);
        msg
    }
    pub fn has_msg(&self) -> bool {
        !self.queue_in.is_empty()
    }
}
