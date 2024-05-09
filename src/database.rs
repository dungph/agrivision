use std::{fmt::Display, time::Duration};

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
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
    pub stage: String,
    pub timestamp: u64,
    pub image: Vec<u8>,
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

#[derive(Debug)]
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

pub async fn list_account() -> anyhow::Result<Vec<User>> {
    Ok(query!(
        r#"
select id, username, is_admin, is_manager, is_watcher
from account
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

pub async fn get_account(username: &str) -> anyhow::Result<User> {
    Ok(query!(
        r#"
select id, username, is_admin, is_manager, is_watcher
from account
where username = ?1
        "#,
        username
    )
    .fetch_one(&*DB)
    .await
    .map(|obj| User {
        id: obj.id,
        username: obj.username,
        is_admin: obj.is_admin != 0,
        is_manager: obj.is_manager != 0,
        is_watcher: obj.is_watcher != 0,
    })?)
}

pub async fn create_account(username: &str, password: &str) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
insert into account
(username, password, is_admin, is_manager, is_watcher)
values (?1, ?2, 0, 0, 0)
        "#,
        username,
        password
    )
    .execute(&*DB)
    .await
    .map(|rows| rows.rows_affected() == 1)?)
}

pub async fn check_password(username: &str, password: &str) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
select password
from account
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

pub async fn remove_account(username: &str, id: i64) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
delete from account
where id = ?2
and exists (
    select id
    from account
    where is_admin = 1
    and username = ?1
)
        "#,
        username,
        id,
    )
    .execute(&*DB)
    .await
    .map(|row| row.rows_affected() == 1)?)
}

pub async fn update_account_role(
    username: &str,
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
update account
set 
    is_admin = ?3,
    is_manager = ?4,
    is_watcher = ?5
where id = ?2
and exists (
    select id
    from account
    where is_admin = 1
    and username = ?1
)
        "#,
        username,
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
from account
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
from account
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
from account
where username = ?1
and is_watcher = 1
        "#,
        username
    )
    .fetch_optional(&*DB)
    .await?
    .is_some())
}
pub async fn list_position(username: &str) -> anyhow::Result<Vec<Position>> {
    Ok(query!(
        r#"
select position.id, x, y
from position
where exists (
    select id
    from account
    where username = ?1
    and (
        is_admin = 1
        or is_manager = 1
        or is_watcher = 1
    )
)
        "#,
        username
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

pub async fn insert_position(username: &str, x: u32, y: u32) -> anyhow::Result<i64> {
    Ok(query!(
        r#"
insert into position
    (x, y, active)
select ?2 as x, ?3 as y, 1 as active
where exists (
    select id
    from account
    where username = ?1
    and is_admin = 1
)
returning id;

        "#,
        username,
        x,
        y,
    )
    .fetch_one(&*DB)
    .await?
    .id)
}

pub async fn remove_position(username: &str, id: i64) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
update position
set active = 0
where id = ?2
and exists (
    select id
    from account
    where is_admin = 1
    and username = ?1
)
        "#,
        username,
        id,
    )
    .execute(&*DB)
    .await
    .map(|row| row.rows_affected() == 1)?)
}

pub async fn insert_check(check: &CheckResult) -> anyhow::Result<i64> {
    Ok(query!(
        r#"
insert into checking_result
(position_id, top, left, width, height, stage, image)
values(
    (select id as position_id from position where x = ?1 and y = ?2),
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    ?8
)
returning id;
        "#,
        check.x,
        check.y,
        check.top,
        check.left,
        check.width,
        check.height,
        check.stage,
        check.image
    )
    .fetch_one(&*DB)
    .await
    .map(|obj| obj.id)?)
}

pub async fn insert_water(check_id: i64) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
insert into checking_water
    (check_id)
values (?1);
        "#,
        check_id
    )
    .execute(&*DB)
    .await
    .map(|rows| rows.rows_affected() == 1)?)
}

pub async fn should_check(x: u32, y: u32) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
select check_period
from checking_config c
join (
    select 
        coalesce(stage, "unknown") as stage,
        coalesce(max(created_ts), 0) as ts
    from checking_result
    where position_id = (
        select id from position
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
from checking_config
join (
    select 
        coalesce(stage, "unknown") as stage,
        coalesce(max(created_ts), 0) as ts
    from checking_result c
    join checking_water w on c.id = w.check_id
    where position_id = (
        select id from position
        where active = 1
        and x = ?1
        and y = ?2
    )
) ch
on checking_config.stage = ch.stage
and checking_config.water_period + ts <= unixepoch()
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
            from checking_config
            where stage = "unknown"
        )
    ) as water_duration
from checking_config
where stage = ?1
        "#,
        stage
    )
    .fetch_one(&*DB)
    .await
    .map(|obj| Duration::from_secs(obj.water_duration as u64))?)
}

pub async fn get_last_check(x: u32, y: u32) -> anyhow::Result<Option<CheckResult>> {
    Ok(query!(
        r#"
select x, y, top, left, width, height, stage, image, max(checking_result.created_ts) timestamp
from checking_result
join position on position.id = position_id
where x = ?1
and y = ?2
        "#,
        x,
        y
    )
    .fetch_optional(&*DB)
    .await?
    .map(|obj| CheckResult {
        x,
        y,
        top: obj.top.unwrap() as u32,
        left: obj.left.unwrap() as u32,
        width: obj.width.unwrap() as u32,
        height: obj.height.unwrap() as u32,
        image: obj.image.unwrap(),
        stage: obj.stage.unwrap().parse().unwrap(),
        timestamp: obj.timestamp.unwrap() as u64,
    }))
}

pub async fn get_active_config(username: &str) -> anyhow::Result<Vec<CheckingConfig>> {
    Ok(query!(
        r#"
select * from checking_config
where exists (
    select id from account
    where username = ?1
    and is_admin = 1
)
        "#,
        username
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
    username: &str,
    stage: &str,
    check_period: i64,
    water_period: i64,
    water_duration: i64,
) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
update checking_config
set
    check_period = ?3,
    water_period = ?4,
    water_duration = ?5
where 
    stage = ?2
    and exists (
        select id from account 
        where username = ?1
        and (
            is_admin = 1
            or is_manager = 1
        )
    )
        "#,
        username,
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
