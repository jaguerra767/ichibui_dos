use rusqlite::{params, Connection};
use std::error::Error;

pub enum DataAction {
    Dispensed,
    RanOut,
    Refilled,
}
impl ToString for DataAction {
    fn to_string(&self) -> String {
        match self {
            DataAction::Dispensed => "Dispensed",
            DataAction::RanOut => "Ran Out",
            DataAction::Refilled => "Refilled",
        }
        .to_string()
    }
}
pub struct Data {
    database: Connection,
}
impl Data {
    pub fn new(database: Connection) -> Self {
        Self { database }
    }

    pub fn connect(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        // Create the table if it doesn't exist
        self.database.execute(
            "CREATE TABLE IF NOT EXISTS dispense_logs (
            id INTEGER PRIMARY KEY,
            timestamp TEXT NOT NULL,
            data TEXT NOT NULL,
            ingredient INTEGER
        )",
            [],
        )?;

        // Count the number of rows in the table
        let row_count: i64 =
            self.database
                .query_row("SELECT COUNT(*) FROM dispense_logs", [], |row| row.get(0))?;

        Ok(row_count)
    }

    pub fn log(
        &self,
        action: DataAction,
        ingredient: Option<usize>,
    ) -> rusqlite::Result<(), Box<dyn Error + Send + Sync>> {
        let curr_time = chrono::Utc::now().to_string();
        self.database.execute(
            "INSERT INTO dispense_logs (timestamp, data, ingredient) VALUES (?1, ?2, ?3)",
            params![curr_time, action.to_string(), ingredient],
        )?;
        Ok(())
    }
}
