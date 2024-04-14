use std::collections::VecDeque;

use async_std::channel::{bounded, Receiver, Sender};
use tide::{Redirect, Request, Response};

use crate::system::System;

use self::msg::{IncomingMessage, OutgoingMessage};

mod msg;

#[derive(Clone)]
struct State {
    queue_out_tx: Sender<OutgoingMessage>,
    queue_out_rx: Receiver<OutgoingMessage>,
    sys: System,
}

pub async fn start_http(sys: System) -> anyhow::Result<()> {
    let (queue_out_tx, queue_out_rx) = bounded(100);

    let rx1 = sys.start_automation().await?;
    let queue_out = queue_out_tx.clone();
    async_std::task::spawn(async move {
        while let Ok(msg) = rx1.recv().await {
            queue_out.send(msg).await.ok();
        }
    });

    let state = State {
        queue_out_tx,
        queue_out_rx,
        sys,
    };

    let mut server = tide::with_state(state);

    server.at("/").get(Redirect::new("/static/index.html"));
    server.at("/static").serve_dir("static/").unwrap();
    server
        .at("/push")
        .post(|mut req: Request<State>| async move {
            let sys = req.state().sys.clone();
            let sender = req.state().queue_out_tx.clone();

            let mut msg_queue = VecDeque::new();
            match req.body_json::<IncomingMessage>().await {
                Ok(msg) => {
                    msg_queue.push_back(msg);
                }
                Err(e) => log::error!("{e}"),
            }

            while let Some(msg) = msg_queue.pop_front() {
                log::info!("Recv: {msg}");
                match msg {
                    IncomingMessage::GetListPot => {
                        let list = sys.positions().await?;
                        for pos in list {
                            sender
                                .send(OutgoingMessage::Pot { x: pos.0, y: pos.1 })
                                .await
                                .ok();
                            msg_queue
                                .push_back(IncomingMessage::GetLastCheck { x: pos.0, y: pos.1 });
                        }
                    }
                    IncomingMessage::GetAutoWater => {
                        let state = sys.auto_water().await?;
                        sender.send(OutgoingMessage::AutoWater { state }).await?;
                    }
                    IncomingMessage::GetAutoCheck => {
                        let state = sys.auto_check().await?;
                        sender.send(OutgoingMessage::AutoCheck { state }).await?;
                    }
                    IncomingMessage::SetAutoWater(state) => {
                        sys.set_auto_water(state).await?;
                        msg_queue.push_back(IncomingMessage::GetAutoWater);
                    }
                    IncomingMessage::SetAutoCheck(state) => {
                        sys.set_auto_check(state).await?;
                        msg_queue.push_back(IncomingMessage::GetAutoCheck);
                    }
                    IncomingMessage::Water { x, y } => {
                        let res = sys.water_at(x, y).await?;
                        sender.send(res.into()).await?;
                        sys.check_at(x, y).await?;
                    }
                    IncomingMessage::Check { x, y } => {
                        let res = sys.check_at(x, y).await?;
                        sender.send(res.into()).await?;
                    }
                    IncomingMessage::Shutdown => {
                        sys.poweroff().await.unwrap();
                    }
                    IncomingMessage::GetAllWater { x, y } => {
                        let msgs = sys.get_all_water_report(x, y).await?;
                        for msg in msgs.values() {
                            sender
                                .send(OutgoingMessage::Water {
                                    x,
                                    y,
                                    timestamp: *msg.timestamp(),
                                })
                                .await?;
                        }
                    }
                    IncomingMessage::GetAllCheck { x, y } => {
                        let msgs = sys.get_all_check_report(x, y).await?;
                        for msg in msgs.values() {
                            sender.send(msg.clone().into()).await?;
                        }
                    }
                    IncomingMessage::GetLastWater { x, y } => {
                        let res = sys.get_last_water_report(x, y).await?;
                        if let Some(res) = res {
                            sender.send(res.into()).await?;
                        }
                    }
                    IncomingMessage::GetLastCheck { x, y } => {
                        let res = sys.get_last_check_report(x, y).await?;
                        if let Some(res) = res {
                            sender.send(res.into()).await?;
                        }
                    }
                    IncomingMessage::Goto { x, y } => {
                        let res = sys.capture_at(x, y).await.map_err(|e| dbg!(e))?;
                        sender.send(res.into()).await?;
                    }
                    IncomingMessage::SetPot { x, y } => {
                        sys.add_positions(x, y).await?;
                        msg_queue.push_back(IncomingMessage::GetListPot);
                    }
                    IncomingMessage::RemovePot { x, y } => {
                        sys.remove_positions(x, y).await?;
                        msg_queue.push_back(IncomingMessage::GetListPot);
                    }
                }
            }
            Ok(Response::builder(200).build())
        });
    server.at("/wait").get(|req: Request<State>| async move {
        let msg = req.state().queue_out_rx.recv().await?;
        //log::info!("Sending {msg}");
        Ok(Response::builder(200)
            .body(tide::Body::from_json(&msg)?)
            .build())
    });
    server.listen("0.0.0.0:8080").await?;
    Ok(())
}
