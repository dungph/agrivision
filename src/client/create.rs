use async_std::task::spawn;
use serde::Deserialize;
use tide::{Redirect, Request};

use crate::{client::get_user, database, system};

pub async fn create_account(mut req: Request<()>) -> tide::Result {
    #[derive(Deserialize)]
    struct Form {
        username: String,
        password: String,
        role: database::AccountRole,
    }
    let Form {
        username,
        password,
        role,
    } = req.body_form().await?;

    match get_user(&req).await? {
        Some(u) if u.is_admin => {
            database::create_account(&username, &password, &role).await?;
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
    if get_user(&req).await?.is_some() {
        database::insert_position(x, y).await.map_err(|e| dbg!(e))?;
        spawn(async move {
            system::check_at(x, y, false).await.ok();
        });
    }
    Ok(Redirect::new("/show/manage/positions").into())
}
