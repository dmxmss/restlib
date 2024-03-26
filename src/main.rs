use std::{
    fs,
    env,
    net::{TcpListener, TcpStream},
    io::{prelude::*, BufReader}
};
use::sqlx::Row;

struct Reader {
    id: i32,
    firstname: String,
    surname: String,
    read_books: Vec<i32>
}

impl Reader {
    async fn new(
        firstname: String, surname: String, pool: &sqlx::PgPool
    ) -> Result<Reader, sqlx::Error> {

        sqlx::query!("insert into reader (id, firstname, surname, read_books)
                         values (
                             (
                                select case when max(id) is null then 0
                                            else max(id)
                                       end
                                from reader
                             ) + 1,
                             $1,
                             $2,
                             array[]::integer[]
                      )", firstname, surname).execute(pool).await?;

        let reader = sqlx::query_as!(Reader, 
                r#"select id, 
                          firstname as "firstname!", 
                          surname as "surname!", 
                          read_books as "read_books!" 
                   from reader 
                   group by id 
                   having id = max(id)"#)
            .fetch_one(pool)
            .await?;

        Ok(reader)
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = sqlx::postgres::PgPool::connect(env::var("DATABASE_URL").unwrap().as_str()).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let reader = Reader::new(String::from("John"), String::from("Doe"), &pool).await?; 

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    
    let (status, file) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(file).unwrap();
    let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", status, contents.len(), contents);
    stream.write_all(response.as_bytes()).unwrap();
}
