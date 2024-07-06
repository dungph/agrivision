create table if not exists positions (
    id          integer not null primary key,
    active      boolean not null,
    x           unsigned integer not null,
    y           unsigned integer not null,
    unique(x, y)
);

create table if not exists images (
    id          integer not null primary key,
    image       blob not null
);

create table if not exists stages (
    id          integer not null primary key,
    stage       text not null unique,
    first_stage boolean not null,
    check_period unsigned integer not null,
    water_period unsigned integer not null,
    water_duration unsigned integer not null
);

create table if not exists checks (
    id          integer not null primary key,
    created_ts  unsigned integer not null,
    position_id unsigned integer not null,
    image_id    unsigned integer not null,
    stage_id    integer not null,
    watered     boolean not null,
    unique(created_ts)
);

create table if not exists accounts (
    id          integer not null primary key,
    username    text    not null unique,
    password    text    not null,
    is_admin    boolean not null,
    is_manager  boolean not null,
    is_watcher  boolean not null
);

insert into accounts
    (username, password, is_admin, is_manager, is_watcher)
values
    ("admin", "admin", true, true, true);

insert into stages
    (stage, first_stage, check_period, water_period, water_duration)
values 
    ("unknown", false, 1000, 9000, 1),
    ("empty",   false, 100,  3000, 2),
    ("young",   true,  100,  3000, 2),
    ("ready",   false, 100,  3000, 2),
    ("old",     false, 100,  3000, 2);
