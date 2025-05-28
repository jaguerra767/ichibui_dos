use rusqlite::{params, Connection};
use std::error::Error;

#[derive(Debug)]
pub enum DataAction {
    DispensedSmall,
    DispensedRegular,
    Cleaning,
    Emptying,
    RanOut,
    Refilled,
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

    pub fn get_bowl_count(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let row_count: i64 =
            self.database
                .query_row("SELECT COUNT(*) FROM dispense_logs WHERE  action IN ('Dispensed', 'DispensedSmall', 'DispensedRegular')", [], |row| row.get(0))?;

        Ok(row_count)
    }

    pub fn log(
        &self,
        action: &DataAction,
        ingredient: Option<usize>,
    ) -> rusqlite::Result<(), Box<dyn Error + Send + Sync>> {
        let curr_time = chrono::Utc::now().to_string();
        let action = format!("{:?}", action);
        self.database.execute(
            "INSERT INTO dispense_logs (timestamp, data, ingredient) VALUES (?1, ?2, ?3)",
            params![curr_time, action, ingredient],
        )?;
        Ok(())
    }
}
