use std::{collections::BTreeMap, time::Duration};

use askama::Template;
use askama_tide::into_response;
use async_std::task::spawn;
use serde::Deserialize;
use tide::Request;

use crate::{
    client::get_user,
    database::{self, CheckingConfig},
    system,
};

#[derive(Template)]
#[template(path = "main.html")]
pub struct Main {
    pub data: MainData,
    pub current_user: Option<database::User>,
    pub error: Option<String>,
    pub info: Option<String>,
}

#[derive(variation::Variation)]
pub enum MainData {
    Login,
    UserManagement(DetailUsers),
    PositionManagement(DetailPositions),
    CheckingConfig(DetailCheckConfig),
    Dashboard(Dashboard),
}

#[derive(Debug, Clone)]
struct Card {
    id: i64,
    x: u32,
    y: u32,
    image_id: i64,
    stage: String,
}

pub struct DetailPositions {
    positions: Vec<database::Position>,
}

pub struct DetailUsers {
    users: Vec<database::User>,
}
pub struct DetailCheckConfig {
    stages: Vec<CheckingConfig>,
}

pub struct Dashboard {
    cards: Vec<Card>,
    current_card: Option<Card>,
    history: Vec<CheckInfo>,
    detail: Vec<DashboardDetail>,
}

struct DashboardDetail {
    stage: String,
    quantity: u32,
}
struct CheckInfo {
    position_id: u32,
    image_id: i64,
    stage: String,
    datetime: String,
}

async fn list_card() -> anyhow::Result<Vec<Card>> {
    let positions = database::get_list_position().await.map_err(|e| dbg!(e))?;
    let mut cards = Vec::<Card>::new();
    for pos in positions {
        let last_check = database::get_last_check(pos.x, pos.y)
            .await
            .map_err(|e| dbg!(e))?;
        cards.push(Card {
            id: pos.id,
            x: pos.x,
            y: pos.y,
            stage: last_check.stage,
            image_id: last_check.image_id,
        });
    }
    Ok(cards)
}
pub async fn dashboard(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await.map_err(|e| dbg!(e))?;

    #[derive(Deserialize)]
    struct Query {
        id: u32,
    }
    let mut info = None;

    let data = match user {
        Some(ref u) if u.is_manager || u.is_watcher => {
            let list = list_card().await.map_err(|e| dbg!(e))?;
            let mut current_card = None;
            let mut history = Vec::new();
            if let Ok(Query { id }) = req.query() {
                if let Some(card) = list.iter().find(|c| c.id == id as i64) {
                    current_card.replace(card.clone());
                    let mut list = database::get_list_check(card.x, card.y).await?;
                    list.sort_unstable_by(|a, b| b.timestamp.cmp(&a.timestamp));

                    history = list
                        .into_iter()
                        .map(|r| CheckInfo {
                            position_id: id,
                            image_id: r.image_id,
                            stage: r.stage,
                            datetime: {
                                let dt: time::OffsetDateTime = (std::time::UNIX_EPOCH
                                    + Duration::from_secs(r.timestamp))
                                .into();
                                dt.to_string()
                            },
                        })
                        .collect();
                }
            }
            let mut detail = BTreeMap::new();

            for card in list.iter() {
                *detail.entry(&card.stage).or_insert(0) += 1;
            }

            MainData::Dashboard(Dashboard {
                current_card,
                history,
                detail: detail
                    .into_iter()
                    .map(|(s, q)| DashboardDetail {
                        stage: s.clone(),
                        quantity: q,
                    })
                    .collect(),
                cards: list,
            })
        }
        Some(_) => {
            info.replace("Please login as manager or watcher".to_owned());
            MainData::Login
        }
        None => MainData::Login,
    };
    Ok(into_response(&Main {
        data,
        current_user: user,
        info,
        error: None,
    }))
}
pub async fn manage_users(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    let mut info = None;
    let data = match user {
        Some(ref user) if user.is_admin => MainData::UserManagement(DetailUsers {
            users: database::get_list_account().await?,
        }),
        _ => {
            info.replace("Please login as admin".to_owned());
            MainData::Login
        }
    };
    Ok(into_response(&Main {
        data,
        current_user: user,
        info,
        error: None,
    }))
}
pub async fn manage_positions(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    #[derive(Deserialize)]
    struct Query {
        x: u32,
        y: u32,
    }

    let mut info = None;

    let data = match user {
        Some(ref user) if user.is_admin => {
            if let Ok(Query { x, y }) = req.query() {
                spawn(system::capture_raw_at(x, y));
            }
            MainData::PositionManagement(DetailPositions {
                positions: database::get_list_position().await?,
            })
        }
        _ => {
            info.replace("Please login as admin".to_owned());
            MainData::Login
        }
    };
    Ok(into_response(&Main {
        data,
        current_user: user,
        info,
        error: None,
    }))
}

pub async fn config_checking(req: Request<()>) -> tide::Result {
    let user = get_user(&req).await?;

    let mut info = None;
    let data = match user {
        Some(ref user) if user.is_manager => MainData::CheckingConfig(DetailCheckConfig {
            stages: database::get_list_checking_config().await?,
        }),
        _ => {
            info.replace("Please login as manager".to_owned());
            MainData::Login
        }
    };
    Ok(into_response(&Main {
        data,
        current_user: user,
        info,
        error: None,
    }))
}
