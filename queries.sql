create table videos (
  id uuid default uuid_generate_v4() primary key,
  title text not null,
  description text,
  video_url text not null,
  thumbnail_url text,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null,
  likes integer default 0,
  views integer default 0
);

create function increment_views(video_id uuid) returns void as $$
begin
  update videos set views = views + 1 where id = video_id;
end;
$$ language plpgsql;

create function toggle_like(video_id uuid) returns void as $$
begin
  update videos set likes = likes + 1 where id = video_id;
end;
$$ language plpgsql;
