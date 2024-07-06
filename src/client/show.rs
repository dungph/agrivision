use std::collections::BTreeMap;

use askama::Template;
use askama_tide::into_response;
use async_std::task::spawn;
use serde::Deserialize;
use tide::{Redirect, Request};

use crate::{
    client::get_user,
    database::{self, AccountData, CheckData, StageData},
    system,
};

#[derive(Template)]
#[template(path = "main.html")]
pub struct Main {
    pub data: MainData,
    pub current_user: Option<AccountData>,
}

#[derive(variation::Variation)]
pub enum MainData {
    Login,
    UserManagement(DetailUsers),
    PositionManagement(DetailPositions),
    StageManagement(DetailStageConfig),
    Dashboard(Dashboard),
    Position(DetailPosition),
}

pub struct DetailPosition {
    current_card: (CheckData, CheckData, StageData),
    history: Vec<(CheckData, StageData)>,
}

pub struct DetailPositions {
    positions: Vec<database::PositionData>,
}

pub struct DetailUsers {
    users: Vec<AccountData>,
}
pub struct DetailStageConfig {
    stages: Vec<StageData>,
}

pub struct Dashboard {
    cards: Vec<(
        database::PositionData,
        database::CheckData,
        database::StageData,
    )>,
    detail: Vec<DashboardDetail>,
}

struct DashboardDetail {
    stage: String,
    quantity: u32,
}

pub async fn dashboard(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await.map_err(|e| dbg!(e))?;

    let data = match user {
        Some(ref u) if u.is_manager || u.is_watcher => {
            let mut infos = Vec::new();
            let positions = database::query_position(None, None).await?;
            for position in positions {
                if let Some(last_check) = database::query_last_checks(Some(position.id), false)
                    .await?
                    .pop()
                {
                    if let Some(last_check_stage) =
                        database::query_stages(Some(last_check.stage_id), None)
                            .await?
                            .pop()
                    {
                        infos.push((position, last_check, last_check_stage));
                    }
                }
            }
            let mut detail = BTreeMap::new();
            for (_pos, _check, stage) in infos.iter() {
                *detail.entry(&stage.stage).or_insert(0) += 1;
            }

            MainData::Dashboard(Dashboard {
                detail: detail
                    .into_iter()
                    .map(|(s, q)| DashboardDetail {
                        stage: s.clone(),
                        quantity: q,
                    })
                    .collect(),
                cards: infos,
            })
        }
        Some(_) => {
            //info.replace("Please login as manager or watcher".to_owned());
            MainData::Login
        }
        None => MainData::Login,
    };
    Ok(into_response(&Main {
        data,
        current_user: user,
    }))
}

pub async fn position(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await.map_err(|e| dbg!(e))?;

    #[derive(Deserialize)]
    struct Query {
        id: i64,
    }

    let data = match user {
        Some(ref u) if u.is_manager || u.is_watcher => {
            if let Ok(Query { id }) = req.query() {
                let mut infos = Vec::new();
                let checks = database::query_checks(Some(id), false).await?;
                for check in checks {
                    if let Some(stage) = database::query_stages(Some(check.stage_id), None)
                        .await?
                        .pop()
                    {
                        infos.push((check, stage));
                    }
                }

                let last_water = database::query_last_checks(Some(id), true).await?.pop();
                let current: Option<(
                    database::CheckData,
                    database::CheckData,
                    database::StageData,
                )> = infos
                    .iter()
                    .reduce(|acc, v| {
                        if v.0.created_ts > acc.0.created_ts {
                            v
                        } else {
                            acc
                        }
                    })
                    .cloned()
                    .and_then(|current| Some((current.0, last_water?, current.1)));

                if let Some(current_check) = current {
                    MainData::Position(DetailPosition {
                        current_card: current_check,
                        history: infos,
                    })
                } else {
                    return Ok(Redirect::new("/show/dashboard").into());
                }
            } else {
                return Ok(Redirect::new("/show/dashboard").into());
            }
        }
        Some(_) => {
            //info.replace("Please login as manager or watcher".to_owned());
            MainData::Login
        }
        None => MainData::Login,
    };
    Ok(into_response(&Main {
        data,
        current_user: user,
    }))
}
pub async fn manage_users(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    let data = match user {
        Some(ref user) if user.is_admin => MainData::UserManagement(DetailUsers {
            users: database::query_account(None, None).await?,
        }),
        _ => {
            //info.replace("Please login as admin".to_owned());
            MainData::Login
        }
    };
    Ok(into_response(&Main {
        data,
        current_user: user,
    }))
}
pub async fn manage_poss(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    #[derive(Deserialize)]
    struct Query {
        x: u32,
        y: u32,
    }

    let data = match user {
        Some(ref user) if user.is_admin => {
            if let Ok(Query { x, y }) = req.query() {
                spawn(system::capture_raw_at(x, y));
            }
            MainData::PositionManagement(DetailPositions {
                positions: database::query_position(None, None).await?,
            })
        }
        _ => {
            //info.replace("Please login as admin".to_owned());
            MainData::Login
        }
    };
    Ok(into_response(&Main {
        data,
        current_user: user,
    }))
}

pub async fn manage_stages(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    let data = match user {
        Some(ref user) if user.is_manager => MainData::StageManagement(DetailStageConfig {
            stages: database::query_stages(None, None).await?,
        }),
        _ => {
            //info.replace("Please login as manager".to_owned());
            MainData::Login
        }
    };
    Ok(into_response(&Main {
        data,
        current_user: user,
    }))
}
