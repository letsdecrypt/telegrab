-- Add migration script here
create table pic
(
    id         serial primary key,
    doc_id     int         not null references doc (id),
    url        text        not null unique,
    seq        int         not null,
    status     smallint             default 0, -- 0: new, 1: downloaded, 2: error
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);