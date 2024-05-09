use askama::Template;
use askama_tide::into_response;
use serde::Deserialize;
use tide::{Redirect, Request};

use crate::database::{self, CheckingConfig};

#[derive(Deserialize)]
enum Cmd {
    HideDetail,
    ShowGeneral,
    ShowListPos,
    ShowCheckingConfig,
    ShowPot(i64),
    ShowUsers,
}

#[derive(Debug, Template)]
#[template(path = "login.html")]
struct Login {
    failed: bool,
}

#[derive(Debug)]
struct Card {
    id: i64,
    top: u32,
    left: u32,
    width: u32,
    height: u32,
    stage: String,
    image_id: u32,
}

#[derive(Debug)]
struct Context {
    user: database::User,
    is_general: bool,
    is_list_pos: bool,
    is_checking_config: bool,
    is_pot: bool,
    is_users: bool,

    show_detail: bool,
    cards: Vec<Card>,
}

#[derive(Debug, Template)]
#[template(path = "dashboard.html")]
struct DashboardOnly {
    context: Context,
}

#[derive(Debug, Template)]
#[template(path = "detail-general.html")]
struct DetailGeneral {
    context: Context,
}

#[derive(Debug, Template)]
#[template(path = "detail-checking-config.html")]
struct DetailCheckingConfig {
    stages: Vec<CheckingConfig>,
    context: Context,
}

#[derive(Debug, Template)]
#[template(path = "detail-pot.html")]
struct DetailPot {
    pot_id: i64,
    context: Context,
}

#[derive(Debug, Template)]
#[template(path = "detail-positions.html")]
struct DetailPositions {
    context: Context,
    positions: Vec<database::Position>,
}

#[derive(Debug, Template)]
#[template(path = "detail-users.html")]
struct DetailUsers {
    context: Context,
    users: Vec<database::User>,
}

fn get_user(req: &Request<()>) -> Option<String> {
    let session = req.session();
    session.get("user")
}

pub async fn start_http() -> anyhow::Result<()> {
    let mut server = tide::new();

    server.with(tide::sessions::SessionMiddleware::new(
        tide::sessions::MemoryStore::new(),
        &[0; 32],
    ));
    server.at("/").get(|req: Request<()>| async move {
        Ok(if let Some(username) = get_user(&req) {
            let user = database::get_account(&username).await?;
            let pos = database::list_position(&username).await?;
            let cards = pos
                .into_iter()
                .map(|database::Position { id, x, y }| Card {
                    id,
                    top: y + 10,
                    left: x + 10,
                    height: 200,
                    width: 200,
                    stage: "unknown".to_owned(),
                    image_id: 0,
                })
                .collect::<Vec<Card>>();

            let mut context = Context {
                user,
                is_pot: false,
                is_general: false,
                is_list_pos: false,
                is_checking_config: false,
                is_users: false,
                show_detail: true,
                cards,
            };
            if let Ok(cmd) = req.query::<Cmd>() {
                match cmd {
                    Cmd::HideDetail => {
                        context.show_detail = false;
                        into_response(&DashboardOnly { context })
                    }
                    Cmd::ShowGeneral => {
                        context.is_general = true;
                        into_response(&DetailGeneral { context })
                    }
                    Cmd::ShowListPos => {
                        context.is_list_pos = true;
                        into_response(&DetailPositions {
                            context,
                            positions: database::list_position(&username).await?,
                        })
                    }
                    Cmd::ShowCheckingConfig => {
                        context.is_checking_config = true;
                        into_response(&DetailCheckingConfig {
                            stages: database::get_active_config(&username).await?,
                            context,
                        })
                    }
                    Cmd::ShowPot(id) => {
                        context.is_pot = true;
                        into_response(&DetailPot {
                            pot_id: id,
                            context,
                        })
                    }
                    Cmd::ShowUsers => {
                        context.is_users = true;
                        into_response(&DetailUsers {
                            context,
                            users: database::list_account().await?,
                        })
                    }
                }
            } else {
                context.show_detail = false;
                into_response(&DashboardOnly { context })
            }
        } else {
            into_response(&Login { failed: false })
        })
    });

    server.at("/logout").all(|mut req: Request<()>| async move {
        let session = req.session_mut();
        session.remove("user");
        Ok(Redirect::new("/"))
    });

    server.at("/login").post(|mut req: Request<()>| async move {
        #[derive(Deserialize)]
        struct Form {
            username: String,
            password: String,
        }
        let Form { username, password } = req.body_form().await?;
        let session = req.session_mut();
        Ok(if database::check_password(&username, &password).await? {
            session.insert("user", username).unwrap();
            Redirect::new("/").into()
        } else {
            into_response(&Login { failed: false })
        }) as tide::Result
    });
    server
        .at("/create/user")
        .post(|mut req: Request<()>| async move {
            #[derive(Deserialize)]
            struct Form {
                username: String,
                password: String,
            }
            let Form { username, password } = req.body_form().await?;
            database::create_account(&username, &password).await?;
            Ok(Redirect::new("/").into()) as tide::Result
        });
    server
        .at("/create/position")
        .post(|mut req: Request<()>| async move {
            #[derive(Deserialize)]
            struct Form {
                x: u32,
                y: u32,
            }
            let Form { x, y } = req.body_form().await.map_err(|e| dbg!(e))?;
            if let Some(user) = get_user(&req) {
                database::insert_position(&user, x, y)
                    .await
                    .map_err(|e| dbg!(e))?;
            }
            Ok(Redirect::new("/"))
        });
    server
        .at("/update/checking_config")
        .post(|mut req: Request<()>| async move {
            if let Some(username) = get_user(&req) {
                #[derive(Deserialize)]
                struct Form {
                    stage: String,
                    check_period: u64,
                    water_period: u64,
                    water_duration: u64,
                }
                let form: Form = req.body_form().await.map_err(|e| dbg!(e))?;
                database::update_checking_config(
                    &username,
                    form.stage.as_str(),
                    form.check_period as i64,
                    form.water_period as i64,
                    form.water_duration as i64,
                )
                .await
                .map_err(|e| dbg!(e))?;
            }
            Ok(Redirect::new("/"))
        });
    server
        .at("/delete/position")
        .post(|mut req: Request<()>| async move {
            if let Some(user) = get_user(&req) {
                #[derive(Deserialize)]
                struct Form {
                    id: i64,
                }
                let Form { id } = req.body_form().await.map_err(|e| dbg!(e))?;
                database::remove_position(&user, id)
                    .await
                    .map_err(|e| dbg!(e))?;
            }
            Ok(Redirect::new("/"))
        });
    server
        .at("/delete/account")
        .post(|mut req: Request<()>| async move {
            if let Some(user) = get_user(&req) {
                #[derive(Deserialize)]
                struct Form {
                    id: i64,
                }
                let Form { id } = req.body_form().await.map_err(|e| dbg!(e))?;
                database::remove_account(&user, id)
                    .await
                    .map_err(|e| dbg!(e))?;
            }
            Ok(Redirect::new("/"))
        });
    server
        .at("/update/account_role")
        .post(|mut req: Request<()>| async move {
            if let Some(user) = get_user(&req) {
                #[derive(Deserialize)]
                struct Form {
                    id: i64,
                    role: FormRole,
                }
                #[derive(Deserialize)]
                #[serde(rename_all = "snake_case")]
                enum FormRole {
                    Admin,
                    Manager,
                    Watcher,
                    None,
                }
                let Form { id, role } = req.body_form().await.map_err(|e| dbg!(e))?;
                match role {
                    FormRole::Admin => {
                        database::update_account_role(&user, id, true, true, true).await?;
                    }
                    FormRole::Manager => {
                        database::update_account_role(&user, id, false, true, true).await?;
                    }
                    FormRole::Watcher => {
                        database::update_account_role(&user, id, false, false, true).await?;
                    }
                    FormRole::None => {
                        database::update_account_role(&user, id, false, false, false).await?;
                    }
                }
            }
            Ok(Redirect::new("/"))
        });
    //server.at("/push").post(|mut req: Request<()>| async move {
    //    let sender = QUEUE.0.clone();

    //    let mut msg_queue = VecDeque::new();
    //    match req.body_json::<IncomingMessage>().await {
    //        Ok(msg) => {
    //            msg_queue.push_back(msg);
    //        }
    //        Err(e) => log::error!("{e}"),
    //    }

    //    while let Some(msg) = msg_queue.pop_front() {
    //        log::info!("Recv: {msg}");
    //        match msg {
    //            IncomingMessage::GetListPot => {
    //                let list = database::list_position().await;
    //                for pos in list {
    //                    sender
    //                        .send(OutgoingMessage::Pot { x: pos.0, y: pos.1 })
    //                        .await
    //                        .ok();
    //                    msg_queue.push_back(IncomingMessage::GetLastCheck { x: pos.0, y: pos.1 });
    //                }
    //            }
    //            IncomingMessage::GetAutoWater => {
    //                let state = sys.auto_water().await.map_err(|e| dbg!(e))?;
    //                sender
    //                    .send(OutgoingMessage::AutoWater { state })
    //                    .await
    //                    .map_err(|e| dbg!(e))?;
    //            }
    //            IncomingMessage::GetAutoCheck => {
    //                let state = sys.auto_check().await.map_err(|e| dbg!(e))?;
    //                sender
    //                    .send(OutgoingMessage::AutoCheck { state })
    //                    .await
    //                    .map_err(|e| dbg!(e))?;
    //            }
    //            IncomingMessage::SetAutoWater(state) => {
    //                sys.set_auto_water(state).await.map_err(|e| dbg!(e))?;
    //                msg_queue.push_back(IncomingMessage::GetAutoWater);
    //            }
    //            IncomingMessage::SetAutoCheck(state) => {
    //                sys.set_auto_check(state).await.map_err(|e| dbg!(e))?;
    //                msg_queue.push_back(IncomingMessage::GetAutoCheck);
    //            }
    //            IncomingMessage::Water { x, y } => {
    //                let res = sys.check_at(x, y, true).await.map_err(|e| dbg!(e))?;
    //                sender.send(res.into()).await.map_err(|e| dbg!(e))?;
    //            }
    //            IncomingMessage::Check { x, y } => {
    //                let res = sys.check_at(x, y, false).await.map_err(|e| dbg!(e))?;
    //                sender.send(res.into()).await.map_err(|e| dbg!(e))?;
    //            }
    //            IncomingMessage::Shutdown => {
    //                sys.poweroff().await.unwrap();
    //            }
    //            IncomingMessage::GetAllWater { x, y } => {
    //                //let msgs = sys.get_all_water_report(x, y).await.map(|e|dbg!(e))?;
    //                //for msg in msgs.values() {
    //                //    sender
    //                //        .send(OutgoingMessage::Water {
    //                //            x,
    //                //            y,
    //                //            timestamp: *msg.timestamp(),
    //                //        })
    //                //        .await.map(|e|dbg!(e))?;
    //                //}
    //            }
    //            IncomingMessage::GetAllCheck { x, y } => {
    //                //let msgs = sys.get_all_check(x, y).await.map(|e|dbg!(e))?;
    //                //for msg in msgs.values() {
    //                //    sender.send(msg.clone().into()).await.map(|e|dbg!(e))?;
    //                //}
    //            }
    //            IncomingMessage::GetLastWater { x, y } => {
    //                let res = sys.get_last_check(x, y, true).await.map_err(|e| dbg!(e))?;
    //                sender.send(res.into()).await.map_err(|e| dbg!(e))?;
    //            }
    //            IncomingMessage::GetLastCheck { x, y } => {
    //                let res = sys.get_last_check(x, y, false).await.map_err(|e| dbg!(e))?;
    //                sender.send(res.into()).await.map_err(|e| dbg!(e))?;
    //            }
    //            IncomingMessage::Goto { x, y } => {
    //                let res = sys.capture_at(x, y).await.map_err(|e| dbg!(e)).unwrap();
    //                sender.send(res.into()).await.map_err(|e| dbg!(e))?;
    //            }
    //            IncomingMessage::SetPot { x, y } => {
    //                sys.add_positions(x, y).await.map_err(|e| dbg!(e))?;
    //                msg_queue.push_back(IncomingMessage::GetListPot);
    //            }
    //            IncomingMessage::RemovePot { x, y } => {
    //                sys.remove_positions(x, y).await.map_err(|e| dbg!(e))?;
    //                msg_queue.push_back(IncomingMessage::GetListPot);
    //            }
    //        }
    //    }
    //    Ok(Response::builder(200).build())
    //});
    //server.at("/wait").get(|req: Request<State>| async move {
    //    let msg = req.state().queue_out_rx.recv().await.map_err(|e| dbg!(e))?;
    //    //log::info!("Sending {msg}");
    //    Ok(Response::builder(200)
    //        .body(tide::Body::from_json(&msg).map_err(|e| dbg!(e))?)
    //        .build())
    //});
    server.listen("0.0.0.0:8080").await.map_err(|e| dbg!(e))?;
    Ok(())
}
