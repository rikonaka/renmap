use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use pistol::scan::TcpUdpScans;
use rusqlite::Connection;
use std::fs;

const SQLITE_DEFAULT_SAVE_PATH: &str = ".renmap.db";

#[derive(Debug)]
pub struct ScanInfo {
    id: u32,
    target_addr: String,
    target_port: String,
    scan_time: DateTime<Local>,
    scans: TcpUdpScans,
}

pub struct SqliteDB {
    conn: Connection,
}

impl SqliteDB {
    pub fn init_db(memory: bool) -> Result<SqliteDB> {
        if memory {
            let conn = Connection::open_in_memory()?;
            Ok(SqliteDB { conn })
        } else {
            let conn = Connection::open(SQLITE_DEFAULT_SAVE_PATH)?;
            Ok(SqliteDB { conn })
        }
    }
    /// Drop all datas saved in local .db file.
    pub fn drop_all() -> Result<()> {
        if std::path::Path::new(SQLITE_DEFAULT_SAVE_PATH).exists() {
            fs::remove_file(SQLITE_DEFAULT_SAVE_PATH)?;
        }
        Ok(())
    }
}
