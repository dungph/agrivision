use askama_tide::into_response;
use serde::Deserialize;
use tide::{Redirect, Request};

use crate::database;

use self::show::{Main, MainData};

mod action;
mod camera;
mod create;
mod delete;
mod show;
mod update;

async fn get_user(req: &Request<()>) -> anyhow::Result<Option<database::AccountData>> {
    let session = req.session();
    if let Some(user) = session.get::<String>("user") {
        Ok(database::query_account(None, Some(&user)).await?.pop())
    } else {
        Ok(None)
    }
}

pub async fn start_http() -> anyhow::Result<()> {
    let mut server = tide::new();

    server.with(tide::sessions::SessionMiddleware::new(
        tide::sessions::MemoryStore::new(),
        &[0; 32],
    ));
    server.at("/").get(index);
    server.at("/show/dashboard").get(show::dashboard);
    server.at("/show/position").get(show::position);
    server.at("/show/manage/stages").get(show::manage_stages);
    server.at("/show/manage/users").get(show::manage_users);
    server.at("/show/manage/positions").get(show::manage_poss);

    server.at("/action/water").get(action::water);
    server.at("/action/check").get(action::check);
    server.at("/action/recheck").get(action::recheck);
    server.at("/action/goto").get(action::goto);

    server.at("/login").post(login);
    server.at("/logout").all(logout);

    server.at("/create/account").post(create::create_account);
    server.at("/create/position").post(create::create_positions);

    server.at("/update/stage").post(update::stage);
    server.at("/update/role").post(update::update_role);

    server.at("/delete/position").post(delete::position);
    server.at("/delete/account").post(delete::account);

    server.at("/camera/stream").get(camera::stream);
    server.at("/camera/snapshot").get(camera::snapshot);
    server.at("/camera/image").get(camera::image);

    server.listen("0.0.0.0:8080").await.map_err(|e| dbg!(e))?;
    Ok(())
}
async fn index(req: Request<()>) -> tide::Result {
    if let Some(user) = get_user(&req).await? {
        if user.is_admin {
            Ok(Redirect::new("/show/manage/positions").into())
        } else {
            Ok(Redirect::new("/show/dashboard").into())
        }
    } else {
        Ok(into_response(&Main {
            data: MainData::Login,
            current_user: None,
        }))
    }
}
async fn logout(mut req: Request<()>) -> tide::Result {
    let session = req.session_mut();
    session.remove("user");
    Ok(into_response(&Main {
        data: MainData::Login,
        current_user: None,
    }))
}

async fn login(mut req: Request<()>) -> tide::Result {
    #[derive(Deserialize)]
    struct Form {
        username: String,
        password: String,
    }
    let Form { username, password } = req.body_form().await.map_err(|e| dbg!(e))?;
    let session = req.session_mut();
    Ok(
        if let Some(account) = database::query_account(None, Some(&username)).await?.pop() {
            if account.password == password {
                session.insert("user", username).unwrap();
            }
            Redirect::new("/").into()
        } else {
            into_response(&Main {
                data: MainData::Login,
                current_user: None,
            })
        },
    ) as tide::Result
}
