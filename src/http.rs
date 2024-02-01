use async_std::sync::Mutex;
use async_std::task::{spawn, JoinHandle};
use tide::{Request, Response};

use crate::gateway::broadcast;
use crate::{control::handle_msg, message::Message};

pub async fn start_server() {
    static HTTP_TASK: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
    if let Some(handle) = HTTP_TASK.lock().await.take() {
        handle.cancel().await;
    }

    let port = crate::settings::restapi().port;
    let task = spawn(async move {
        broadcast(Message::Status("Http starting".to_owned()));
        server().listen(format!("0.0.0.0:{}", port)).await.ok();
        broadcast(Message::Status("Http stopped".to_owned()));
    });
    HTTP_TASK.lock().await.replace(task);
}

fn server() -> tide::Server<()> {
    let mut server = tide::new();

    server.at("/").serve_file("static/index.html").unwrap();
    server.at("/static").serve_dir("static/").unwrap();
    server.at("/push").post(|mut req: Request<()>| async move {
        if let Ok(msg) = req.body_json::<Message>().await {
            log::info!("{:?}", msg);
            handle_msg(msg).await;
            Ok(Response::builder(200).build())
        } else {
            Ok(Response::builder(400).build())
        }
    });
    server
        .at("/pull")
        .get(tide::sse::endpoint(|_req, sender| async move {
            let mut queue = crate::gateway::receiver();

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
        }));
    server
}
