use chrono::NaiveDate;
use csv::{ReaderBuilder, WriterBuilder};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize)]
struct CleanDataWithAnalytics {
    date: NaiveDate,
    close: f32,
    two_hundred_wma: f32,
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

    let mut clean_data_with_analytics: Vec<CleanDataWithAnalytics> = Vec::new();
    for (i, row) in clean_data.iter().enumerate() {
        clean_data_with_analytics.push(CleanDataWithAnalytics {
            date: row.date,
            close: row.close,
            two_hundred_wma: moving_averages[i],
        });
    }

    println!("Loaded {} rows of data", clean_data_with_analytics.len());
    for i in 0..4 {
        println!("Row +{} of clean data: {:?}", i+1, clean_data_with_analytics[i]);
    }
    for i in 0..4 {
        println!("Row -{} of clean data: {:?}", i+1, clean_data_with_analytics[clean_data_with_analytics.len() - i - 1]);
    }

    const OUTPUT_DIRECTORY: &str = "output/";
    const OUTPUT_CSV_FILENAME: &str = "clean_data_with_analytics.csv";
    let output_csv_path: String = format!("{}{}", OUTPUT_DIRECTORY, OUTPUT_CSV_FILENAME);
    const OUTPUT_IMAGE_FILENAME: &str = "200wma.png";
    let output_image_path: String = format!("{}{}", OUTPUT_DIRECTORY, OUTPUT_IMAGE_FILENAME);

    // if the path OUTPUT_IMAGE_DESTINATION doesn't exist as a directory, create it
    std::fs::create_dir_all(OUTPUT_DIRECTORY)?;

    // Save the clean_data_with_analytics to a CSV file
    let mut writer = WriterBuilder::new().from_path(output_csv_path)?;
    for record in &clean_data_with_analytics {
        writer.serialize(record)?;
    }
    writer.flush()?;

    // TODO: try 1024x768, 800x600, 640x480, 320x240, 1280x1024, 1920x1080
    const OUTPUT_IMAGE_WIDTH: u32 = 800;
    const OUTPUT_IMAGE_HEIGHT: u32 = 600;
    const OUTPUT_IMAGE_DIMENSIONS: (u32, u32) = (OUTPUT_IMAGE_WIDTH, OUTPUT_IMAGE_HEIGHT);

    // Build the drawing area (root) for the chart
    let root = BitMapBackend::new(&output_image_path, OUTPUT_IMAGE_DIMENSIONS).into_drawing_area();
    root.fill(&WHITE)?;

    // Calculate the max and min values for both domensions of the chart
    // TODO: dimensions should be the min of close or two_hundred_wma
    let min_close = clean_data_with_analytics.iter().map(|d| d.close).fold(f32::INFINITY, f32::min);
    let min_wma = clean_data_with_analytics.iter().map(|d| d.two_hundred_wma).fold(f32::INFINITY, f32::min);
    let min_value = f32::min(min_close, min_wma);

    let max_close = clean_data_with_analytics.iter().map(|d| d.close).fold(f32::NEG_INFINITY, f32::max);
    let max_wma = clean_data_with_analytics.iter().map(|d| d.two_hundred_wma).fold(f32::NEG_INFINITY, f32::max);
    let max_value = f32::max(max_close, max_wma);
    
    let min_date = clean_data_with_analytics.iter().map(|d| d.date).fold(NaiveDate::MAX, NaiveDate::min);
    let max_date = clean_data_with_analytics.iter().map(|d| d.date).fold(NaiveDate::MIN, NaiveDate::max);

    println!("Min value: {}", min_value);
    println!("Max value: {}", max_value);
    println!("Min date:  {}", min_date);
    println!("Max date:  {}", max_date);

    let chart_caption = format!("Price and 200-WMA from {} to {}", min_date, max_date);

    let mut chart = ChartBuilder::on(&root)
        .caption(chart_caption, ("sans-serif", 20).into_font())
        .build_cartesian_2d(min_date..max_date, min_close..max_close)?;

    chart.draw_series(LineSeries::new(
        clean_data_with_analytics.iter().map(|d| (d.date, d.close)),
        &BLUE,
    ))?;

    chart.draw_series(LineSeries::new(
        clean_data_with_analytics.iter().map(|d| (d.date, d.two_hundred_wma)),
        &RED,
    ))?;

    root.present()?;

    Ok(())
}
