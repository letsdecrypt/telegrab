-- Add migration script here
create table cbz
(
    id         serial primary key,
    doc_id     int references doc (id) on delete set null,
    path       text      not null unique,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
)