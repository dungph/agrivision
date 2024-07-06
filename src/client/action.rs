use serde::Deserialize;
use tide::{Request, Response};

use crate::{client::get_user, system};

pub async fn water(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    #[derive(Deserialize)]
    struct Query {
        id: i64,
    }
    match user {
        Some(user) if user.is_admin || user.is_manager => {
            if let Ok(Query { id }) = req.query() {
                async_std::task::spawn(async move {
                    system::check_at(id, true).await?;
                    Ok(()) as anyhow::Result<()>
                });
                Ok(Response::new(200))
            } else {
                Ok(Response::new(404))
            }
        }
        _ => Ok(Response::new(403)),
    }
}
pub async fn recheck(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    #[derive(Deserialize)]
    struct Query {
        id: i64,
    }
    match user {
        Some(user) if user.is_admin || user.is_manager => {
            if let Ok(Query { id }) = req.query() {
                async_std::task::spawn(async move {
                    system::recheck_id(id).await?;
                    Ok(()) as anyhow::Result<()>
                });
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
        id: i64,
    }
    match user {
        Some(user) if user.is_admin || user.is_manager => {
            if let Ok(Query { id }) = req.query() {
                async_std::task::spawn(async move {
                    system::check_at(id, false).await?;
                    Ok(()) as anyhow::Result<()>
                });
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
                let img = async_std::task::spawn(async move {
                    let img = system::capture_raw_at(x, y).await?;
                    Ok(img) as anyhow::Result<_>
                })
                .await?;
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
