use once_cell::sync::Lazy;
use sqlx::{query, query_as, SqlitePool};

static DB: Lazy<SqlitePool> = Lazy::new(|| {
    SqlitePool::connect_lazy(&std::env::var("DATABASE_URL").expect("`DATABASE_URL` must be set"))
        .expect("Unable to connect database")
});

pub async fn migrate() -> anyhow::Result<()> {
    sqlx::migrate!().run(&*DB).await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct PositionData {
    pub id: i64,
    pub active: bool,
    pub x: i64,
    pub y: i64,
}
#[derive(Debug, Clone)]
pub struct ImageData {
    pub id: i64,
    pub image: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct StageData {
    pub id: i64,
    pub stage: String,
    pub first_stage: bool,
    pub check_period: i64,
    pub water_period: i64,
    pub water_duration: i64,
}
#[derive(Debug, Clone)]
pub struct CheckData {
    pub id: i64,
    pub created_ts: i64,
    pub position_id: i64,
    pub image_id: i64,
    pub stage_id: i64,
    pub watered: bool,
}
#[derive(Debug, Clone)]
pub struct AccountData {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
    pub is_manager: bool,
    pub is_watcher: bool,
}
pub async fn query_position(
    id: Option<i64>,
    xy: Option<(u32, u32)>,
) -> anyhow::Result<Vec<PositionData>> {
    let x = xy.map(|(x, _y)| x);
    let y = xy.map(|(_x, y)| y);

    Ok(query_as!(
        PositionData,
        r#"
select * from positions
where (?1 is null and ?2 is null and ?3 is null)
or (?1 is null and x = ?2 and y = ?3)
or (id = ?1 and ?2 is null and ?3 is null)
        "#,
        id,
        x,
        y
    )
    .fetch_all(&*DB)
    .await?)
}

pub async fn query_account(
    id: Option<i64>,
    username: Option<&str>,
) -> anyhow::Result<Vec<AccountData>> {
    Ok(query_as!(
        AccountData,
        r#"
select * from accounts
where (?1 is null and ?2 is null)
or (?1 is null and username = ?2)
or (id = ?1 and username is null)
        "#,
        id,
        username
    )
    .fetch_all(&*DB)
    .await?)
}
pub async fn query_image(id: i64) -> anyhow::Result<ImageData> {
    Ok(query_as!(
        ImageData,
        r#"
select * from images
where (id=?1)
        "#,
        id
    )
    .fetch_one(&*DB)
    .await?)
}
pub async fn query_check(id: i64) -> anyhow::Result<CheckData> {
    Ok(query_as!(
        CheckData,
        r#"
select * from checks
where (id=?1)
        "#,
        id
    )
    .fetch_one(&*DB)
    .await?)
}
pub async fn query_checks(
    position_id: Option<i64>,
    watered: bool,
) -> anyhow::Result<Vec<CheckData>> {
    Ok(query_as!(
        CheckData,
        r#"
select * from checks
where (?2 = false or watered = true)
and (?1 is null or position_id = ?1)
order by (created_ts) desc;
        "#,
        position_id,
        watered
    )
    .fetch_all(&*DB)
    .await?)
}

pub async fn query_last_checks(
    position_id: Option<i64>,
    watered: bool,
) -> anyhow::Result<Vec<CheckData>> {
    Ok(query!(
        r#"
select 
    id,
    position_id,
    max(created_ts) as created_ts,
    stage_id,
    image_id,
    watered
from checks
where (?2 = false or watered = true)
and (?1 is null or position_id = ?1)
group by (position_id)
        "#,
        position_id,
        watered
    )
    .fetch_all(&*DB)
    .await?
    .into_iter()
    .filter_map(|obj| {
        Some(CheckData {
            id: obj.id?,
            position_id: obj.position_id?,
            created_ts: obj.created_ts as i64,
            stage_id: obj.stage_id?,
            image_id: obj.image_id?,
            watered: obj.watered?,
        })
    })
    .collect())
}

pub async fn query_stages(id: Option<i64>, stage: Option<&str>) -> anyhow::Result<Vec<StageData>> {
    Ok(query_as!(
        StageData,
        r#"
select * from stages
where (?1 is null and ?2 is null)
or (?1 is null and stage = ?2)
or (id = ?1 and ?2 is null)
        "#,
        id,
        stage
    )
    .fetch_all(&*DB)
    .await?)
}

pub async fn query_images(id: Option<i64>) -> anyhow::Result<Vec<ImageData>> {
    Ok(query_as!(
        ImageData,
        r#"
select * from images
where ?1 is null
or id = ?1
        "#,
        id,
    )
    .fetch_all(&*DB)
    .await?)
}

pub async fn upsert_position(x: u32, y: u32) -> anyhow::Result<i64> {
    Ok(query!(
        r#"
insert into positions (active, x, y)
values(true, ?1, ?2)
on conflict (x, y) 
do update
set active = true
returning id
        "#,
        x,
        y
    )
    .fetch_one(&*DB)
    .await?
    .id)
}

pub async fn upsert_account(account: AccountData) -> anyhow::Result<i64> {
    Ok(query!(
        r#"
insert into accounts (username, password, is_admin, is_manager, is_watcher)
values(?1, ?2, ?3, ?4, ?5)
on conflict (username) 
do update
set is_admin = ?3,
    is_manager = ?4,
    is_watcher = ?5
returning id
        "#,
        account.username,
        account.password,
        account.is_admin,
        account.is_manager,
        account.is_watcher,
    )
    .fetch_one(&*DB)
    .await?
    .id)
}

pub async fn upsert_check(check: CheckData) -> anyhow::Result<i64> {
    Ok(query!(
        r#"
insert into checks (position_id, stage_id, image_id, watered, created_ts)
values(?1, ?2, ?3, ?4, ?5)
on conflict(created_ts)
do update
set 
    stage_id = ?2,
    image_id = ?3,
    watered = ?4
returning id
        "#,
        check.position_id,
        check.stage_id,
        check.image_id,
        check.watered,
        check.created_ts,
    )
    .fetch_one(&*DB)
    .await?
    .id)
}

pub async fn upsert_stage(stage: StageData) -> anyhow::Result<StageData> {
    Ok(query_as!(
        StageData,
        r#"
insert into stages (stage, first_stage, check_period, water_period, water_duration)
values(?1, ?2, ?3, ?4, ?5)
on conflict (stage) 
do update
set first_stage = ?2,
    check_period = ?3,
    water_period = ?4,
    water_duration = ?5
returning 
    id,
    stage,
    first_stage,
    check_period,
    water_period,
    water_duration
        "#,
        stage.stage,
        stage.first_stage,
        stage.check_period,
        stage.water_period,
        stage.water_duration,
    )
    .fetch_one(&*DB)
    .await?)
}

pub async fn insert_image(image: &[u8]) -> anyhow::Result<i64> {
    Ok(query!(
        r#"
insert into images (image)
values(?1)
returning id
        "#,
        image,
    )
    .fetch_one(&*DB)
    .await?
    .id)
}
pub async fn update_image(id: i64, image: &[u8]) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
update images
set image = ?2
where id = ?1
        "#,
        id,
        image,
    )
    .execute(&*DB)
    .await
    .map(|r| r.rows_affected() == 1)?)
}
pub async fn delete_position(id: i64) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
delete from positions
where id = ?1
        "#,
        id,
    )
    .execute(&*DB)
    .await?
    .rows_affected()
        == 1)
}

pub async fn delete_account(id: i64) -> anyhow::Result<bool> {
    Ok(query!(
        r#"
delete from accounts
where id = ?1
        "#,
        id,
    )
    .execute(&*DB)
    .await?
    .rows_affected()
        == 1)
}
