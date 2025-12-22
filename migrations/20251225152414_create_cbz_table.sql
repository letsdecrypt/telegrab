-- Add migration script here
create table cbz
(
    id         serial primary key,
    doc_id     int references doc (id),
    path       text      not null unique,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now()
)