use std::{
    error::Error,
    net::TcpListener,
    env
};
use restlib::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = sqlx::postgres::PgPool::connect(env::var("DATABASE_URL").unwrap().as_str()).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, &pool).await?;
    }

    Ok(())
}

