use std::{
    str,
    error::Error,
    fs,
    net::TcpStream,
    io::{prelude::*, BufReader}
};

pub struct Reader {
    pub id: i32,
    pub firstname: String,
    pub surname: String,
    pub read_books: Vec<i32>
}

impl Reader {
    pub async fn create(
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

    pub async fn delete(reader: Reader, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!("delete from reader 
                      where id = $1", reader.id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

pub struct Book {
    pub id: i32,
    pub name: String 
}

impl Book {
    pub async fn new(
        name: String, pool: &sqlx::PgPool
    ) -> Result<Book, sqlx::Error> {

        sqlx::query!("insert into book (id, name)
                         values (
                             (
                                select case when max(id) is null then 0
                                            else max(id)
                                       end
                                from book
                             ) + 1,
                             $1
                      )", name).execute(pool).await?;

        let book = sqlx::query_as!(Book, 
                r#"select id, 
                          name as "name!"
                   from book 
                   group by id 
                   having id = max(id)"#)
            .fetch_one(pool)
            .await?;

        Ok(book)
    }

    pub async fn delete(book: Book, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!("delete from book 
                      where id = $1", book.id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

pub async fn handle_connection(mut stream: TcpStream, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let (request_line, body) = handle_stream(&mut stream)?;
   
    let (status, file) = route_request(request_line, body, pool).await;

    let contents = fs::read_to_string(file).unwrap();
    let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", status, contents.len(), contents);
    stream.write_all(response.as_bytes()).unwrap();

    Ok(())
}

fn handle_stream(stream: &mut TcpStream) -> Result<(String, String), Box<dyn Error>> {
    let buf_reader = BufReader::new(stream.try_clone().unwrap());
    let request = get_request_contents(buf_reader)?;
    let mut lines = request.lines();

    let request_line = lines.next().unwrap().to_string();
    let body = lines.last().unwrap().to_string();

    Ok((request_line, body))
}

async fn route_request(request_line: String, body: String, pool: &sqlx::PgPool) -> (&'static str, &'static str) {
    match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),

        "GET /signup HTTP/1.1" => ("HTTP/1.1 200 OK", "signup.html"),

        "POST /readers HTTP/1.1" => {
            let (firstname, surname) = parse_post_data(body);
            let path = "index.html";

            match Reader::create(firstname.to_string(), surname.to_string(), pool).await {
                Ok(_) => ("HTTP/1.1 201 CREATED", path),
                Err(e) => {
                    println!("{:?}", e);
                    ("HTTP/1.1 409 CONFLICT", path)
                }
            }
        },

        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    }
}

fn parse_post_data(data: String) -> (String, String) {
    let mut cred: Vec<String> = data.split("&").map(|s| {
        s.split("=").last().unwrap().to_string()
    }).collect();

    let surname = cred.pop().unwrap();
    let firstname = cred.pop().unwrap();

    (firstname, surname)
}

fn get_request_contents(mut buf_reader: BufReader<TcpStream>) -> Result<String, Box<dyn Error>> {
    let request = str::from_utf8(buf_reader.fill_buf()?)?.to_string();
    buf_reader.consume(request.len());
    Ok(request)
}

