use async_std::task::spawn;
use serde::Deserialize;
use tide::{Redirect, Request};

use crate::{client::get_user, database, system};

pub async fn create_account(mut req: Request<()>) -> tide::Result {
    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum AccountRole {
        Admin,
        Manager,
        Watcher,
        None,
    }

    #[derive(Deserialize)]
    struct Form {
        username: String,
        password: String,
        role: AccountRole,
    }
    let Form {
        username,
        password,
        role,
    } = req.body_form().await?;

    match get_user(&req).await? {
        Some(admin) if admin.is_admin => {
            let roles = match role {
                AccountRole::Admin => (true, false, false),
                AccountRole::Manager => (false, true, false),
                AccountRole::Watcher => (false, false, true),
                AccountRole::None => (false, false, false),
            };
            database::upsert_account(database::AccountData {
                id: 0,
                username,
                password,
                is_admin: roles.0,
                is_manager: roles.1,
                is_watcher: roles.2,
            })
            .await?;
        }
        _ => (),
    }
    Ok(Redirect::new("/show/manage/users").into())
}

pub async fn create_positions(mut req: Request<()>) -> tide::Result {
    #[derive(Deserialize)]
    struct Form {
        x: u32,
        y: u32,
    }
    let Form { x, y } = req.body_form().await.map_err(|e| dbg!(e))?;
    match get_user(&req).await? {
        Some(user) if user.is_admin => {
            let id = database::upsert_position(x, y).await?;
            spawn(async move {
                system::check_at(id, false).await.ok();
            });
        }
        _ => (),
    }
    Ok(Redirect::new("/show/manage/positions").into())
}
