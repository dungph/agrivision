use std::io::Cursor;

use askama::Template;
use askama_tide::into_response;
use base64::Engine;
use serde::Deserialize;
use tide::{Redirect, Request};

use crate::{
    database::{self, CheckResult, CheckingConfig},
    system,
};

#[derive(Debug, Template)]
#[template(path = "login.html")]
struct Login {
    failed: bool,
}

#[derive(Debug)]
struct CardDetail {
    crop_top: u32,
    crop_left: u32,
    crop_right: u32,
    crop_bottom: u32,
}

#[derive(Debug)]
struct Card {
    id: i64,
    x: u32,
    y: u32,
    stage: String,
    image: String,
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
    image: Option<String>,
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
            let positions = database::list_position(&username).await?;
            let mut cards = Vec::<Card>::new();
            for pos in positions {
                let last_check = database::get_last_check(pos.x, pos.y).await?;
                cards.push(Card {
                    id: pos.id,
                    x: pos.x,
                    y: pos.y,
                    stage: last_check.stage,
                    image: base64::prelude::BASE64_STANDARD.encode(&last_check.image),
                });
            }

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

            #[derive(Deserialize)]
            enum Cmd {
                HideDetail,
                ShowGeneral,
                ShowListPos,
                ShowCheckingConfig,
                ShowPot(i64),
                ShowUsers,
                ShowPosition(String),
            }

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
                            image: None,
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
                    Cmd::ShowPosition(s) => {
                        dbg!(&s);
                        if let Some((xs, ys)) = s.split_once(' ') {
                            if let Ok(x) = xs.parse() {
                                if let Ok(y) = ys.parse() {
                                    let capture = system::capture_at(x, y).await?;
                                    let mut buf = Vec::new();
                                    capture
                                        .image
                                        .write_to(
                                            &mut Cursor::new(&mut buf),
                                            image::ImageFormat::Jpeg,
                                        )
                                        .unwrap();
                                    let img = base64::prelude::BASE64_STANDARD.encode(buf);
                                    context.is_list_pos = true;
                                    return Ok(into_response(&DetailPositions {
                                        context,
                                        positions: database::list_position(&username).await?,
                                        image: Some(img),
                                    }));
                                }
                            }
                        }

                        into_response(&DashboardOnly { context })
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

    server.at("/goto").get(|req: Request<()>| async move {
        #[derive(Deserialize)]
        struct Form {
            x: u32,
            y: u32,
        }

        let Form { x, y } = req.query()?;
        let query = format!("/?ShowPosition={}+{}", x, y);
        Ok(Redirect::new(query))
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
    server.listen("0.0.0.0:8080").await.map_err(|e| dbg!(e))?;
    Ok(())
}
