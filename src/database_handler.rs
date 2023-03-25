use rusqlite::ffi::Error;
use serde::Serialize;
use tokio_rusqlite::Connection as Connection;
use rusqlite::{Result, params, ErrorCode};
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc, Weekday, Date, NaiveDateTime, Duration};
use chrono_tz::Asia::Seoul;
use chrono_tz::Tz;
use log::{error, info, warn};

pub enum DBError {
    LibError(rusqlite::ErrorCode),
    TitleAlreadyUsed,
    TitleNotFound,
}

pub enum DBSuccess {
    NewUrl,
    ExistUrl,
}

pub async fn initialize(conn: &Connection) -> Result<()> {
    conn.call(|conn| {
        conn.execute(
        "CREATE TABLE IF NOT EXISTS url (
                id  INTEGER PRIMARY KEY, 
                url TEXT
            )", 
            params![]
        ).unwrap();
        conn.execute(
        "CREATE TABLE IF NOT EXISTS title (
                id      INTEGER PRIMARY KEY, 
                title   TEXT
            )", 
            params![]
        ).unwrap();
        conn.execute(
        "CREATE TABLE IF NOT EXISTS url_title (
                url_id      INTEGER REFERENCES url(id) ON UPDATE CASCADE ON DELETE CASCADE,
                title_id    INTEGER REFERENCES title(id) ON UPDATE CASCADE ON DELETE CASCADE,
                PRIMARY KEY(url_id, title_id)
            )", 
            params![]
        ).unwrap();
    }).await;

    Ok(())
}


// 현재 여러 url이 같은 정답 갖는 경우 비허용
// 추후 중복 허용 필요가능성 있음

pub async fn add_title_with_url(conn: &Connection, url: String, title: String) -> Result<DBSuccess, DBError> {
    conn.call(move |conn| {
        let _url_id = conn.query_row("SELECT id FROM url WHERE url = (?1)", params![url], |row| row.get::<usize, u64>(0));
        let _title_id = conn.query_row("SELECT id FROM title WHERE title = (?1)", params![title], |row| row.get::<usize, u64>(0));
        match _title_id {
            Ok(_) => Err(DBError::TitleAlreadyUsed),
            Err(_) => {
                let tx = conn.transaction().unwrap();
                tx.execute("INSERT INTO title (title) VALUES (?1)", params![title]).unwrap();
                let title_id = tx.query_row("SELECT last_insert_rowid()", params![], |row| row.get::<usize, u64>(0)).unwrap();
                let (url_type, url_id) = match _url_id {
                    Ok(url_id) => (0, url_id),
                    Err(_) => {
                        tx.execute("INSERT INTO url (url) VALUES (?1)", params![url]).unwrap();
                        (1, tx.query_row("SELECT last_insert_rowid()", params![], |row| row.get::<usize, u64>(0)).unwrap())
                    }
                };
                tx.execute("INSERT INTO url_title (url_id, title_id) VALUES (?1, ?2)", params![url_id, title_id]).unwrap();
                tx.commit().unwrap();
                if url_type == 1 {
                    Ok(DBSuccess::NewUrl)
                } else {
                    Ok(DBSuccess::ExistUrl)
                }
            }
        }
    }).await
}

// alias id 검색 -> 있으면 실패
// title id 검색 -> 없으면 실패
// 있으면 url_title에서 url_id 찾기
// url_id 와 alias 연결
pub async fn add_alias_with_title(conn: &Connection, title: String, alias: String) -> Result<(), DBError> {
    conn.call(move |conn| {
        let _alias_id = conn.query_row("SELECT id FROM title WHERE title = (?1)", params![alias], |row| row.get::<usize, u64>(0));
        let _title_id = conn.query_row("SELECT id FROM title WHERE title = (?1)", params![title], |row| row.get::<usize, u64>(0));
        match _alias_id {
            Ok(_) => return Err(DBError::TitleAlreadyUsed),
            Err(_) => {
                let tx = conn.transaction().unwrap();
                let title_id = match _title_id {
                    Ok(title_id) => title_id,
                    Err(_) => return Err(DBError::TitleNotFound)
                };
                let url_id = tx.query_row("SELECT url_id FROM url_title WHERE title_id = (?1)", params![title_id], |row| row.get::<usize, u64>(0)).unwrap();
                tx.execute("INSERT INTO title (title) VALUES (?1)", params![alias]).unwrap();
                let alias_id = tx.query_row("SELECT last_insert_rowid()", params![], |row| row.get::<usize, u64>(0)).unwrap();
                tx.execute("INSERT INTO url_title (url_id, title_id) VALUES (?1, ?2)", params![url_id, alias_id]).unwrap();
                tx.commit().unwrap();
                Ok(())
            }
        }
    }).await
}

pub async fn remove_title(conn: &Connection, title: String) -> Result<(), DBError> {
    conn.call(move |conn| {
        let tx = conn.transaction().unwrap();
        let title_id = tx.query_row("SELECT id FROM title WHERE title = (?1)", params![title], |row| row.get::<usize, u64>(0)).unwrap();
        tx.execute("DELETE FROM url_title WHERE title_id = (?1)", params![title_id]).unwrap();
        tx.execute("DELETE FROM title WHERE id = (?1)", params![title_id]).unwrap();
        tx.commit().unwrap();
        Ok(())
    }).await
}