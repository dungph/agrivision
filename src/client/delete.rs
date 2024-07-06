use serde::Deserialize;
use tide::{Redirect, Request};

use crate::database;

use super::get_user;

pub async fn position(mut req: Request<()>) -> tide::Result {
    match get_user(&req).await? {
        Some(user) if user.is_admin => {
            #[derive(Deserialize)]
            struct Form {
                id: i64,
            }
            let Form { id } = req.body_form().await.map_err(|e| dbg!(e))?;
            database::delete_position(id).await?;
        }
        _ => (),
    }
    Ok(Redirect::new("/show/manage/positions").into())
}
pub async fn account(mut req: Request<()>) -> tide::Result {
    match get_user(&req).await? {
        Some(user) if user.is_admin => {
            #[derive(Deserialize)]
            struct Form {
                id: i64,
            }
            let Form { id } = req.body_form().await.map_err(|e| dbg!(e))?;
            database::delete_account(id).await?;
        }
        _ => (),
    }
    Ok(Redirect::new("/show/manage/users").into())
}
