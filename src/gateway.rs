use std::net::SocketAddr;

use async_std::channel::{bounded, Receiver, Sender};
use tide::{Redirect, Request, Response};

use crate::message::{InMsg, OutMsg};

pub struct Gateway {
    queue_in: Receiver<InMsg>,
    queue_out: Sender<OutMsg>,
    queue_in_myself: Sender<InMsg>,
}

impl Gateway {
    pub fn with_socket(socket: SocketAddr) -> Self {
        let queue_in = bounded::<InMsg>(1000);
        let queue_out = bounded::<OutMsg>(1000);
        let queue_in_myself = queue_in.0.clone();
        let mut server = tide::with_state((queue_in.0, queue_out.1));

        server.at("/").get(Redirect::new("/static/index.html"));
        server.at("/static").serve_dir("static/").unwrap();
        server.at("/static/out").serve_dir("out/").unwrap();
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
        server.at("/pull").get(tide::sse::endpoint(
            |req: Request<(_, Receiver<_>)>, sender| async move {
                let queue = req.state().1.clone();

                loop {
                    match queue.recv().await {
                        Ok(msg) => {
                            sender
                                .send("message", serde_json::to_string(&msg)?, None)
                                .await?;
                        }
                        Err(_) => {
                            log::warn!("Queue overflowed");
                        }
                    }
                }
            },
        ));
        async_std::task::spawn(async move {
            server.listen(socket).await.ok();
        });
        Self {
            queue_in: queue_in.1,
            queue_out: queue_out.0,
            queue_in_myself,
        }
    }
    pub async fn send(&self, msg: OutMsg) {
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

//static QUEUE: Lazy<(Sender<Message>, Receiver<Message>)> = Lazy::new(|| bounded(100));
//
//pub fn send_out(msg: Message) {
//    QUEUE.0.try_send(msg).ok();
//}
//
//pub fn start_http() -> anyhow::Result<()> {
//    let mut server = tide::new();
//
//    server.at("/").serve_file("static/index.html").unwrap();
//    server.at("/static").serve_dir("static/").unwrap();
//    server.at("/push").post(|mut req: Request<_>| async move {
//        if let Ok(msg) = req.body_json::<Message>().await {
//            crate::control::handle_msg(msg).await;
//            Ok(Response::builder(200).build())
//        } else {
//            Ok(Response::builder(400).build())
//        }
//    });
//    server
//        .at("/pull")
//        .get(tide::sse::endpoint(|req: Request<_>, sender| async move {
//            let queue = QUEUE.1.clone();
//
//            loop {
//                match queue.recv().await {
//                    Ok(msg) => {
//                        sender
//                            .send("message", serde_json::to_string(&msg)?, None)
//                            .await?;
//                    }
//                    Err(_) => {
//                        log::warn!("Queue overflowed");
//                    }
//                }
//            }
//        }));
//    async_std::task::spawn(async {
//        server.listen(*crate::CONF.system().listen_socket()).await;
//    });
//    Ok(())
//}
