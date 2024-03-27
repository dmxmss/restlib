use sqlx::Row;

pub struct Reader {
    pub id: i32,
    pub firstname: String,
    pub surname: String,
    pub read_books: Vec<i32>
}

impl Reader {
    pub async fn new(
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

