use std::{fmt::Display, io::Cursor, time::Duration};

use image::DynamicImage;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::{query, SqlitePool};

static DB: Lazy<SqlitePool> = Lazy::new(|| {
    SqlitePool::connect_lazy(&std::env::var("DATABASE_URL").expect("`DATABASE_URL` must be set"))
        .expect("Unable to connect database")
});

pub struct WaterResult {
    pub x: u32,
    pub y: u32,
    pub stage: String,
    pub timestamp: u64,
}

pub struct CheckResult {
    pub x: u32,
    pub y: u32,
    pub stage: String,
    pub timestamp: u64,
    pub image_id: i64,
    pub water_duration: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CheckingConfig {
    #[serde(default = "Default::default")]
    pub id: i64,
    pub stage: String,
    pub check_period: i64,
    pub water_period: i64,
    pub water_duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub is_manager: bool,
    pub is_watcher: bool,
}

#[derive(Debug)]
pub struct Position {
    pub id: i64,
    pub x: u32,
    pub y: u32,
}

pub async fn migrate() -> anyhow::Result<()> {
    sqlx::migrate!().run(&*DB).await?;
    Ok(())
}

pub async fn get_list_account() -> anyhow::Result<Vec<User>> {
    Ok(query!(
        r#"
select id, username, is_admin, is_manager, is_watcher
from accounts
        "#,
    )
    .fetch_all(&*DB)
    .await?
    .into_iter()
    .map(|obj| User {
        id: obj.id,
        username: obj.username,
        is_admin: obj.is_admin != 0,
        is_manager: obj.is_manager != 0,
        is_watcher: obj.is_watcher != 0,
    })
    .collect())
}

pub async fn get_account(username: &str) -> anyhow::Result<Option<User>> {
    Ok(query!(
        r#"
select id, username, is_admin, is_manager, is_watcher
from accounts
where username = ?1
        "#,
        username
    )
    .fetch_optional(&*DB)
    .await?
    .map(|obj| User {
        id: obj.id,
        username: obj.username,
        is_admin: obj.is_admin != 0,
        is_manager: obj.is_manager != 0,
        is_watcher: obj.is_watcher != 0,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountRole {
    Admin,
    Manager,
    Watcher,
    None,
}

pub async fn create_account(
    username: &str,
    password: &str,
    role: &AccountRole,
) -> anyhow::Result<bool> {
    let raw_role = match role {
        AccountRole::Admin => (1, 0, 0),
        AccountRole::Manager => (0, 1, 0),
        AccountRole::Watcher => (0, 0, 1),
        AccountRole::None => (0, 0, 0),
    };
    Ok(query!(
        r#"
insert into accounts
(username, password, is_admin, is_manager, is_watcher)
values (?1, ?2, ?3, ?4, ?5)
        "#,
        username,
        password,
        raw_role.0,
        raw_role.1,
        raw_role.2,
    )
    .execute(&*DB)
    .await
    .map(|rows| rows.rows_affected() == 1)?)
}

pub async fn check_password(username: &str, password: &str) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
select password
from accounts
where username = ?1
        "#,
        username
    )
    .fetch_optional(&*DB)
    .await
    .map(|obj| {
        if let Some(obj) = obj {
            obj.password == password
        } else {
            false
        }
    })?)
}

pub async fn remove_account(id: i64) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
delete from accounts
where id = ?1
        "#,
        id,
    )
    .execute(&*DB)
    .await
    .map(|row| row.rows_affected() == 1)?)
}

pub async fn update_account_role(
    id: i64,
    is_admin: bool,
    is_manager: bool,
    is_watcher: bool,
) -> anyhow::Result<bool> {
    let is_admin = if is_admin { 1 } else { 0 };
    let is_manager = if is_manager { 1 } else { 0 };
    let is_watcher = if is_watcher { 1 } else { 0 };

    Ok(query!(
        r#"
update accounts
set 
    is_admin = ?2,
    is_manager = ?3,
    is_watcher = ?4
where id = ?1
        "#,
        id,
        is_admin,
        is_manager,
        is_watcher,
    )
    .execute(&*DB)
    .await
    .map(|row| row.rows_affected() == 1)?)
}

pub async fn is_admin(username: &str) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
select id
from accounts
where username = ?1
and is_admin = 1
        "#,
        username
    )
    .fetch_optional(&*DB)
    .await?
    .is_some())
}

pub async fn is_manager(username: &str) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
select id
from accounts
where username = ?1
and is_manager = 1
        "#,
        username
    )
    .fetch_optional(&*DB)
    .await?
    .is_some())
}
pub async fn is_watcher(username: &str) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
select id
from accounts
where username = ?1
and is_watcher = 1
        "#,
        username
    )
    .fetch_optional(&*DB)
    .await?
    .is_some())
}
pub async fn get_list_position() -> anyhow::Result<Vec<Position>> {
    Ok(query!(
        r#"
select positions.id, x, y
from positions
where active = 1
        "#,
    )
    .fetch_all(&*DB)
    .await?
    .into_iter()
    .map(|obj| Position {
        id: obj.id,
        x: obj.x as u32,
        y: obj.y as u32,
    })
    .collect())
}

pub async fn insert_position(x: u32, y: u32) -> anyhow::Result<i64> {
    Ok(query!(
        r#"
insert into positions
    (x, y, active)
values (?1, ?2, 1)
returning id;

        "#,
        x,
        y,
    )
    .fetch_one(&*DB)
    .await?
    .id)
}

pub async fn remove_position(id: i64) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
update positions
set active = 0
where id = ?1
        "#,
        id,
    )
    .execute(&*DB)
    .await
    .map(|row| row.rows_affected() == 1)?)
}

pub async fn insert_image(image: &[u8]) -> anyhow::Result<i64> {
    Ok(query!(
        r#"
insert into images
(image)
values(
    ?1
)
returning id;
        "#,
        image,
    )
    .fetch_one(&*DB)
    .await
    .map(|obj| obj.id)?)
}

pub async fn get_image(image_id: i64) -> anyhow::Result<Vec<u8>> {
    Ok(query!(
        r#"
select image from images
where id = ?1;
        "#,
        image_id,
    )
    .fetch_one(&*DB)
    .await
    .map(|obj| obj.image.to_owned())?)
}

pub async fn insert_check(check: &CheckResult) -> anyhow::Result<i64> {
    Ok(query!(
        r#"
insert into checking_results
(position_id, stage, image_id, water_duration)
values(
    (select id as position_id from positions where x = ?1 and y = ?2),
    ?3,
    ?4,
    ?5
)
returning id;
        "#,
        check.x,
        check.y,
        check.stage,
        check.image_id,
        check.water_duration
    )
    .fetch_one(&*DB)
    .await
    .map(|obj| obj.id)?)
}

pub async fn should_check(x: u32, y: u32) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
select check_period
from checking_configs c
join (
    select 
        coalesce(stage, "unknown") as stage,
        coalesce(max(created_ts), 0) as ts
    from checking_results
    where position_id = (
        select id from positions
        where x=?1
        and y = ?2
        and active = 1
    )
) o
on o.stage = c.stage
and ts + check_period  <= unixepoch()
        "#,
        x,
        y
    )
    .fetch_optional(&*DB)
    .await?
    .is_some())
}
pub async fn shoud_water(x: u32, y: u32) -> anyhow::Result<Option<u64>> {
    Ok(query!(
        r#"
select water_duration
from checking_configs
join (
    select 
        coalesce(stage, "unknown") as stage,
        coalesce(max(created_ts), 0) as ts
    from checking_results c
    where water_duration > 0 
    and position_id = (
        select id from positions
        where active = 1
        and x = ?1
        and y = ?2
    )
) ch
on checking_configs.stage = ch.stage
and checking_configs.water_period + ts <= unixepoch()
        "#,
        x,
        y
    )
    .fetch_optional(&*DB)
    .await?
    .map(|obj| obj.water_duration as u64))
}

pub async fn water_duration(stage: &str) -> anyhow::Result<Duration> {
    Ok(query!(
        r#"
select 
    coalesce(
        water_duration,
        (
            select water_duration
            from checking_configs
            where stage = "unknown"
        )
    ) as water_duration
from checking_configs
where stage = ?1
        "#,
        stage
    )
    .fetch_one(&*DB)
    .await
    .map(|obj| Duration::from_secs(obj.water_duration as u64))?)
}

pub async fn get_last_water(x: u32, y: u32) -> anyhow::Result<CheckResult> {
    Ok(query!(
        r#"
select x, y, stage, image_id, max(checking_results.created_ts) timestamp, water_duration
from checking_results
join positions on positions.id = position_id
where x = ?1
and y = ?2
and water_duration > 0
        "#,
        x,
        y
    )
    .fetch_optional(&*DB)
    .await?
    .and_then(|obj| {
        Some(CheckResult {
            x,
            y,
            image_id: obj.image_id?,
            stage: obj.stage?.parse().unwrap(),
            timestamp: obj.timestamp? as u64,
            water_duration: Some(obj.water_duration.map(|i| i as u32)?),
        })
    })
    .unwrap_or_else(|| CheckResult {
        x,
        y,
        image_id: 0,
        stage: "unknown".to_owned(),
        timestamp: 0,
        water_duration: Some(0),
    }))
}

pub async fn get_last_check(x: u32, y: u32) -> anyhow::Result<CheckResult> {
    Ok(query!(
        r#"
select x, y, stage, image_id, max(checking_results.created_ts) timestamp, water_duration
from checking_results
join positions on positions.id = position_id
where x = ?1
and y = ?2
        "#,
        x,
        y
    )
    .fetch_optional(&*DB)
    .await?
    .and_then(|obj| {
        Some(CheckResult {
            x,
            y,
            image_id: obj.image_id?,
            stage: obj.stage?.parse().unwrap(),
            timestamp: obj.timestamp? as u64,
            water_duration: obj.water_duration.map(|i| i as u32),
        })
    })
    .unwrap_or_else(|| CheckResult {
        x,
        y,
        image_id: 0,
        stage: "unknown".to_owned(),
        timestamp: 0,
        water_duration: None,
    }))
}

pub async fn get_list_check(x: u32, y: u32) -> anyhow::Result<Vec<CheckResult>> {
    Ok(query!(
        r#"
select x, y, stage, image_id, checking_results.created_ts, water_duration
from checking_results
join positions on positions.id = position_id
where x = ?1
and y = ?2
        "#,
        x,
        y
    )
    .fetch_all(&*DB)
    .await?
    .into_iter()
    .map(|obj| CheckResult {
        x,
        y,
        image_id: obj.image_id,
        stage: obj.stage,
        timestamp: obj.created_ts as u64,
        water_duration: obj.water_duration.map(|i| i as u32),
    })
    .collect())
}

pub async fn get_checking_config_stage(stage: &str) -> anyhow::Result<CheckingConfig> {
    Ok(query!(
        r#"
select *, 1 as filter from checking_configs
where stage = ?1
union
select *, 2 as filter from checking_configs
where stage = 'unknown'
ORDER  by filter
        "#,
        stage
    )
    .fetch_one(&*DB)
    .await
    .map(|obj| CheckingConfig {
        id: obj.id,
        stage: obj.stage,
        check_period: obj.check_period,
        water_period: obj.water_period,
        water_duration: obj.water_duration,
    })?)
}
pub async fn get_list_checking_config() -> anyhow::Result<Vec<CheckingConfig>> {
    Ok(query!(
        r#"
select * from checking_configs
        "#,
    )
    .fetch_all(&*DB)
    .await?
    .into_iter()
    .map(|obj| CheckingConfig {
        id: obj.id,
        stage: obj.stage,
        check_period: obj.check_period,
        water_period: obj.water_period,
        water_duration: obj.water_duration,
    })
    .collect())
}

pub async fn update_checking_config(
    stage: &str,
    check_period: i64,
    water_period: i64,
    water_duration: i64,
) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
update checking_configs
set
    check_period = ?2,
    water_period = ?3,
    water_duration = ?4
where stage = ?1
        "#,
        stage,
        check_period,
        water_period,
        water_duration,
    )
    .execute(&*DB)
    .await?
    .rows_affected()
        == 1)
}
