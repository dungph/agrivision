use serde::Deserialize;
use tide::{Redirect, Request};

use crate::database;

use super::get_user;

pub async fn update_role(mut req: Request<()>) -> tide::Result {
    match get_user(&req).await? {
        Some(user) if user.is_admin => {
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
            match role {
                FormRole::Admin => {
                    database::update_account_role(id, true, false, false).await?;
                }
                FormRole::Manager => {
                    database::update_account_role(id, false, true, false).await?;
                }
                FormRole::Watcher => {
                    database::update_account_role(id, false, false, true).await?;
                }
                FormRole::None => {
                    database::update_account_role(id, false, false, false).await?;
                }
            }
        }
        _ => (),
    }
    Ok(Redirect::new("/show/manage/users").into())
}

pub async fn config_checking(mut req: Request<()>) -> tide::Result {
    if get_user(&req).await?.is_some() {
        #[derive(Deserialize)]
        struct Form {
            stage: String,
            check_period: u64,
            water_period: u64,
            water_duration: u64,
        }
        let form: Form = req.body_form().await.map_err(|e| dbg!(e))?;
        database::update_checking_config(
            form.stage.as_str(),
            form.check_period as i64,
            form.water_period as i64,
            form.water_duration as i64,
        )
        .await
        .map_err(|e| dbg!(e))?;
    }
    Ok(Redirect::new("/show/config/checking").into())
}
