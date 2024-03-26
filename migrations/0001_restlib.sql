create table reader (
	id serial primary key,
	firstname varchar(100),
	surname varchar(100),
	read_books integer[] 
);

create table book (
	id serial primary key,
	name varchar(100)
);
