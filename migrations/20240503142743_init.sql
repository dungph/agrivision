create table if not exists position (
    id          integer not null primary key,
    created_ts  integer not null default (unixepoch()),
    active      integer not null,
    x           integer not null,
    y           integer not null
);

create table if not exists checking_result (
    id          integer not null primary key,
    created_ts  integer not null default (unixepoch()),
    position_id integer not null,
    
    top         integer not null,
    left        integer not null,
    width       integer not null,
    height      integer not null,
    
    stage       text not null,
    image       blob not null
);

create table if not exists checking_water (
    id          integer not null primary key,
    check_id    integer not null,
    water_duration integer not null
);

create table if not exists checking_config (
    id          integer not null primary key,
    stage       text not null,
    check_period integer not null,
    water_period integer not null,
    water_duration integer not null
);

create table if not exists account (
    id          integer not null primary key,
    created_ts  integer not null default (unixepoch()),
    is_admin    integer not null,
    is_manager  integer not null,
    is_watcher  integer not null,
    username    text    not null,
    password    text    not null
);

create table if not exists partial_watcher (
    id          integer not null primary key,
    key         text not null,
    created_ts  integer not null default (unixepoch()),
    position_id integer not null,
    from_ts     integer not null,
    to_ts       integer not null
);


insert into account
    (username, password, is_admin, is_manager, is_watcher)
values
    ("admin", "admin", 1, 1, 1);

insert into checking_config
    (stage, check_period, water_period, water_duration)
values 
    ("unknown", 100, 300, 2),
    ("empty", 100, 300, 2),
    ("young", 100, 300, 2),
    ("ready", 100, 300, 2),
    ("old", 100, 300, 2);
