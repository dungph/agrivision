use serde::Deserialize;
use tide::{Redirect, Request};

use crate::database;

use super::get_user;

pub async fn position(mut req: Request<()>) -> tide::Result {
    if get_user(&req).await?.is_some() {
        #[derive(Deserialize)]
        struct Form {
            id: i64,
        }
        let Form { id } = req.body_form().await.map_err(|e| dbg!(e))?;
        database::remove_position(id).await.map_err(|e| dbg!(e))?;
    }
    Ok(Redirect::new("/show/manage/positions").into())
}
pub async fn account(mut req: Request<()>) -> tide::Result {
    if get_user(&req).await?.is_some() {
        #[derive(Deserialize)]
        struct Form {
            id: i64,
        }
        let Form { id } = req.body_form().await.map_err(|e| dbg!(e))?;
        database::remove_account(id).await.map_err(|e| dbg!(e))?;
    }
    Ok(Redirect::new("/show/manage/users").into())
}
