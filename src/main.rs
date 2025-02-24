use serde::Deserialize;
use csv::ReaderBuilder;
use std::error::Error;
use chrono::NaiveDate;

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

fn main() -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'|')
        .from_path("./resources/data/historical_data.csv")?;
    let mut data: Vec<RawData> = Vec::new();

    for result in reader.deserialize() {
        let record: RawData = result?;
        data.push(record);
    }

    let test_date_str = format!("{} {} {}", data[0].month, data[0].day, data[0].year);
    let date = NaiveDate::parse_from_str(&test_date_str, "%b %d %Y")?;

    let test_close_str = data[0].close.replace(",", "");
    let close: f32 = test_close_str.parse()?;

    println!("The current time is: {:?}", chrono::Local::now());
    println!("Loaded {} rows", data.len());    
    println!("First row Date: {}", date);
    println!("First row Close: {}", close);

    Ok(())
}
