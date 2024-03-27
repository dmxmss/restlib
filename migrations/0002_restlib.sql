alter table reader
alter column firstname set not null,
alter column surname set not null,
add unique (firstname);

alter table book
alter column name set not null,
add unique (name);
