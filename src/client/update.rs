use serde::Deserialize;
use tide::{Redirect, Request};

use crate::database;

use super::get_user;

pub async fn update_role(mut req: Request<()>) -> tide::Result {
    if let Some(user) = get_user(&req).await? {
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
        let roles = match role {
            FormRole::Admin => (true, false, false),
            FormRole::Manager => (false, true, false),
            FormRole::Watcher => (false, false, true),
            FormRole::None => (false, false, false),
        };
        database::upsert_account(database::AccountData {
            id: 0,
            username: user.username,
            password: String::new(),
            is_admin: roles.0,
            is_manager: roles.1,
            is_watcher: roles.2,
        })
        .await?;
    }
    Ok(Redirect::new("/show/manage/users").into())
}

pub async fn stage(mut req: Request<()>) -> tide::Result {
    match get_user(&req).await? {
        Some(user) if user.is_admin => {
            #[derive(Deserialize)]
            struct Form {
                stage: String,
                is_first_stage: bool,
                check_period: u32,
                water_period: u32,
                water_duration: u32,
            }
            let form: Form = req.body_form().await.map_err(|e| dbg!(e))?;
            database::upsert_stage(database::StageData {
                id: 0,
                stage: form.stage,
                first_stage: form.is_first_stage,
                check_period: form.check_period as i64,
                water_duration: form.water_duration as i64,
                water_period: form.water_period as i64,
            })
            .await
            .map_err(|e| dbg!(e))?;
        }
        _ => (),
    }
    Ok(Redirect::new("/show/manage/stages").into())
}
