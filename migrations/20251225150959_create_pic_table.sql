-- Add migration script here
create table pic
(
    id         serial primary key,
    doc_id     int       not null references doc (id),
    url        text      not null unique,
    status     int, -- 0: new, 1: downloaded, 2: error
    created_at timestamp not null default now(),
    updated_at timestamp not null default now()
);