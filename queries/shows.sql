create table shows(
    id uuid default uuid_generate_v4() primary key,
    title text not null,
    description text not null,
    release_date date not null,
    thumbnail_url text not null,
    episode_count integer not null,
    genre text not null check (genre in ('Revenge', 'Billionare', 'Asian', 'Romance')),
    rating float not null,
    status text not null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now()
);

-- Enable Row Level Security (RLS) on shows table
alter table shows enable row level security;
-- Create an index on the genre column for faster querying
create index idx_shows_genre on shows(genre);
create index idx_shows_rating on shows(rating);

create index idx_shows_genre_rating on shows(genre, rating);

create or replace function get_shows_by_genre(genre_param text)
returns setof shows as $$
begin
    return query
    select * from shows
    where genre = genre_param
    order by rating desc, release_date desc;
end;
$$ language plpgsql;
