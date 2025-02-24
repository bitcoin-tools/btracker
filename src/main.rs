use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct StockData {
    #[serde(rename = "Month")]
    month: String,
    #[serde(rename = "Day")]
    day: String,
    #[serde(rename = "Year")]
    year: String,
    #[serde(rename = "Close")]
    close: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'|')
        .from_path("./resources/data/historical_data.csv")?;
    let mut data: Vec<StockData> = Vec::new();

    for result in reader.deserialize() {
        let record: StockData = result?;
        data.push(record);
    }

    let test_date_str = format!("{} {} {}", data[0].month, data[0].day, data[0].year);
    let date = NaiveDate::parse_from_str(&test_date_str, "%b %d %Y")?;
    println!("The current time is: {:?}", chrono::Local::now());
    println!("Loaded {} rows", data.len());    
    println!("First row Date: {}", date);
    println!("First row: {:?}", data[0]);
    Ok(())
}
