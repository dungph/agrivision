use serde::Deserialize;
use tide::{Request, Response};

use crate::{client::get_user, database, system};

pub async fn water_stage(req: Request<()>) -> tide::Result {
    if let Ok(stage) = req.param("stage") {
        let user = get_user(&req).await?;

        match user {
            Some(user) if user.is_manager => {
                let list_positions = database::get_list_position().await?;
                for pos in list_positions {
                    let last_check = database::get_last_check(pos.x, pos.y).await?;
                    if last_check.stage == stage {
                        system::check_at(pos.x, pos.y, true).await?;
                    }
                }
                Ok(Response::new(200))
            }
            _ => Ok(Response::new(403)),
        }
    } else {
        Ok(Response::new(404))
    }
}

pub async fn check_stage(req: Request<()>) -> tide::Result {
    if let Ok(stage) = req.param("stage") {
        let user = get_user(&req).await?;

        match user {
            Some(user) if user.is_manager => {
                let list_positions = database::get_list_position().await?;
                for pos in list_positions {
                    let last_check = database::get_last_check(pos.x, pos.y).await?;
                    if last_check.stage == stage {
                        system::check_at(pos.x, pos.y, false).await?;
                    }
                }
                Ok(Response::new(200))
            }
            _ => Ok(Response::new(403)),
        }
    } else {
        Ok(Response::new(404))
    }
}

pub async fn water(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    #[derive(Deserialize)]
    struct Query {
        x: u32,
        y: u32,
    }
    match user {
        Some(user) if user.is_admin || user.is_manager => {
            if let Ok(Query { x, y }) = req.query() {
                system::check_at(x, y, true).await?;
                Ok(Response::new(200))
            } else {
                Ok(Response::new(404))
            }
        }
        _ => Ok(Response::new(403)),
    }
}
pub async fn check(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    #[derive(Deserialize)]
    struct Query {
        x: u32,
        y: u32,
    }
    match user {
        Some(user) if user.is_admin || user.is_manager => {
            if let Ok(Query { x, y }) = req.query() {
                system::check_at(x, y, false).await?;
                Ok(Response::new(200))
            } else {
                Ok(Response::new(404))
            }
        }
        _ => Ok(Response::new(403)),
    }
}
pub async fn goto(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    #[derive(Deserialize)]
    struct Query {
        x: u32,
        y: u32,
    }
    match user {
        Some(user) if user.is_admin => {
            if let Ok(Query { x, y }) = req.query() {
                let img = system::capture_raw_at(x, y).await?;
                let response = tide::Response::builder(200)
                    .header("Access-Control-Allow-Origin", "*")
                    .content_type("image/jpeg")
                    .body(tide::Body::from_bytes(img))
                    .build();
                Ok(response)
            } else {
                Ok(Response::new(404))
            }
        }
        _ => Ok(Response::new(403)),
    }
}
