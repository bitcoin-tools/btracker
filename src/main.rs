use serde::Deserialize;
use csv::ReaderBuilder;
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

    println!("The current time is: {:?}", chrono::Local::now());
    println!("Loaded {} rows", data.len());
    println!("First row Month: {}", data[0].month);
    println!("First row Day: {}", data[0].day);
    println!("First row Year: {}", data[0].year);
    println!("First row Close: {}", data[0].close);
    println!("First row: {:?}", data[0]);

    Ok(())
}
