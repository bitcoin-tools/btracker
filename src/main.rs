use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct RawData {
    #[serde(rename = "Month")]
    month: String,
    #[serde(rename = "Day")]
    day: String,
    #[serde(rename = "Year")]
    year: String,
    #[serde(rename = "Close")]
    close: String,
}

#[derive(Debug)]
struct CleanData {
    date: NaiveDate,
    close: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'|')
        .from_path("./resources/data/historical_data.csv")?;
    let mut raw_data: Vec<RawData> = Vec::new();

    for result in reader.deserialize() {
        let record: RawData = result?;
        raw_data.push(record);
    }

    let mut clean_data: Vec<CleanData> = Vec::new();
    for row in &raw_data {
        let date_str = format!("{} {} {}", row.month, row.day, row.year);
        let date = NaiveDate::parse_from_str(&date_str, "%b %d %Y")?;
        let close_str = row.close.replace(",", "");
        let close: f32 = close_str.parse()?;
        clean_data.push(CleanData { date, close });
    }

    println!("The current time is: {:?}", chrono::Local::now());
    println!("Loaded {} rows of raw data", data.len());
    println!("Loaded {} rows of clean data", clean_data.len());
    println!("First row Date: {}", clean_data[0].date);
    println!("First row Close: {}", clean_data[0].close);
    println!("First row: {:?}", clean_data[0]);

    Ok(())
}
