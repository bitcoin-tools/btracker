use chrono::NaiveDate;
use csv::ReaderBuilder;
use plotters::prelude::*;
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

#[derive(Debug, Clone)]
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

    let moving_average_size = 1400;
    let mut moving_averages: Vec<f32> = Vec::new();

    for i in 0..clean_data.len() {
        if i == clean_data.len() - 1 {
            moving_averages.push(0.0 as f32);
            break;
        }

        let mut sum = 0.0;
        let j_start = i + 1;
        let j_end;
        let j_size;

        if i < clean_data.len() - moving_average_size {
            j_end = j_start + moving_average_size;
            j_size = moving_average_size;
        } else {
            j_end = clean_data.len();
            j_size = j_end - j_start;
        }

        for j in j_start..j_end {
            sum += clean_data[j].close;
        }
        moving_averages.push(sum / j_size as f32);    
    }

    println!("The current time is: {:?}", chrono::Local::now());
    println!("Loaded {} rows of data", clean_data.len());
    println!("First row Date: {}", clean_data[0].date);
    println!("First row Close: {}", clean_data[0].close);
    println!("Row 1 of clean data: {:?}", clean_data[0]);
    println!("Row 2 of clean data: {:?}", clean_data[1]);
    println!("Row 3 of clean data: {:?}", clean_data[2]);
    println!("Row 1 of moving averages:  {}", moving_averages[0]);
    println!("Row 2 of moving averages:  {}", moving_averages[1]);
    println!("Row 3 of moving averages:  {}", moving_averages[2]);

    println!("Row -4 of clean data:      {}", clean_data[clean_data.len() - 4].close);
    println!("Row -3 of clean data:      {}", clean_data[clean_data.len() - 3].close);
    println!("Row -2 of clean data:      {}", clean_data[clean_data.len() - 2].close);
    println!("Row -1 of clean data:      {}", clean_data[clean_data.len() - 1].close);
    println!("Row -4 of moving averages: {}", moving_averages[moving_averages.len() - 4]);
    println!("Row -3 of moving averages: {}", moving_averages[moving_averages.len() - 3]);
    println!("Row -2 of moving averages: {}", moving_averages[moving_averages.len() - 2]);
    println!("Row -1 of moving averages: {}", moving_averages[moving_averages.len() - 1]);

    // TODO: try 1024x768, 800x600, 640x480, 320x240, 1280x1024, 1920x1080
    let root = BitMapBackend::new("output/200wma.png", (600, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_close = clean_data.iter().map(|d| d.close).fold(f32::INFINITY, f32::min);
    let max_close = clean_data.iter().map(|d| d.close).fold(f32::NEG_INFINITY, f32::max);
    let min_date = clean_data.iter().map(|d| d.date).fold(NaiveDate::MAX, NaiveDate::min);
    let max_date = clean_data.iter().map(|d| d.date).fold(NaiveDate::MIN, NaiveDate::max);
    println!("Min close: {}", min_close);
    println!("Max close: {}", max_close);
    println!("Min date: {}", min_date);
    println!("Max date: {}", max_date);

    let chart_caption = format!("Price from {} to {}", min_date, max_date);

    let mut chart = ChartBuilder::on(&root)
        .caption(chart_caption, ("sans-serif", 20).into_font())
        .build_cartesian_2d(min_date..max_date, min_close..max_close)?;

    chart.draw_series(LineSeries::new(
        clean_data.iter().map(|d| (d.date, d.close)),
        &BLUE,
    ))?;

    root.present()?;

    Ok(())
}
