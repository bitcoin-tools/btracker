use chrono::NaiveDate;
use csv::{ReaderBuilder, WriterBuilder};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

// Analytics constants
const MOVING_AVERAGE_DAYS: usize = 1400;

// Input and output constants
const INPUT_DATA_PATH_STR: &str = "./resources/data/historical_data.csv";
const OUTPUT_DIRECTORY: &str = "output/";
const OUTPUT_CSV_FILENAME: &str = "clean_data_with_analytics.csv";
const OUTPUT_IMAGE_FILENAME: &str = "200_week_moving_average.png";

// Chart constants
const CHART_COLOR_BACKGROUND: RGBColor = WHITE;
const CHART_COLOR_PRICE_SERIES: RGBColor = BLUE;
const CHART_COLOR_WMA_SERIES: RGBColor = RED;
const CHART_FONT: (&str, u32) = ("sans-serif", 20);
const CHART_TITLE: &str = "Price and 200-WMA";

// Image dimensions
const OUTPUT_IMAGE_WIDTH: u32 = 1024;
const OUTPUT_IMAGE_HEIGHT: u32 = 600;
// TODO: try others like 1024x768, 800x600, 640x480, 320x240, 1280x1024, 1920x1080
const OUTPUT_IMAGE_DIMENSIONS: (u32, u32) = (OUTPUT_IMAGE_WIDTH, OUTPUT_IMAGE_HEIGHT);

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

impl RawData {
    fn new(path: &Path) -> Result<Vec<RawData>, Box<dyn Error>> {
        let mut reader = ReaderBuilder::new().delimiter(b'|').from_path(path)?;
        let raw_data: Vec<RawData> = reader.deserialize().collect::<Result<_, _>>()?;
        Ok(raw_data)
    }
}

#[derive(Debug, Clone)]
struct CleanData {
    date: NaiveDate,
    close: f32,
}

impl CleanData {
    fn new(raw_data: &[RawData]) -> Result<Vec<CleanData>, Box<dyn Error>> {
        let mut clean_data: Vec<CleanData> = Vec::new();
        for row in raw_data {
            let date_str = format!("{} {} {}", row.month, row.day, row.year);
            let date = NaiveDate::parse_from_str(&date_str, "%b %d %Y")?;
            let close_str = row.close.replace(",", "");
            let close: f32 = close_str.parse()?;
            clean_data.push(CleanData { date, close });
        }
        Ok(clean_data)
    }
}

#[derive(Debug, Serialize)]
struct CleanDataWithAnalytics {
    date: NaiveDate,
    close: f32,
    two_hundred_wma: f32,
}

impl CleanDataWithAnalytics {
    fn new(clean_data: &[CleanData], moving_average_size: usize) -> Vec<CleanDataWithAnalytics> {
        let moving_averages = Self::calculate_moving_averages(clean_data, moving_average_size);
        clean_data
            .iter()
            .enumerate()
            .map(|(i, row)| CleanDataWithAnalytics {
                date: row.date,
                close: row.close,
                two_hundred_wma: moving_averages[i],
            })
            .collect()
    }

    fn calculate_moving_averages(clean_data: &[CleanData], moving_average_size: usize) -> Vec<f32> {
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
        moving_averages
    }

    fn save_to_csv(data: &[CleanDataWithAnalytics], path: &Path) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new().from_path(path)?;
        for record in data {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    fn min_close(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter().map(|d| d.close).fold(f32::INFINITY, f32::min)
    }

    fn min_wma(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.two_hundred_wma)
            .fold(f32::INFINITY, f32::min)
    }

    fn max_close(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.close)
            .fold(f32::NEG_INFINITY, f32::max)
    }

    fn max_wma(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.two_hundred_wma)
            .fold(f32::NEG_INFINITY, f32::max)
    }

    fn min_value(data: &[CleanDataWithAnalytics]) -> f32 {
        f32::min(Self::min_close(data), Self::min_wma(data))
    }

    fn max_value(data: &[CleanDataWithAnalytics]) -> f32 {
        f32::max(Self::max_close(data), Self::max_wma(data))
    }

    fn min_date(data: &[CleanDataWithAnalytics]) -> NaiveDate {
        data.iter()
            .map(|d| d.date)
            .fold(NaiveDate::MAX, NaiveDate::min)
    }

    fn max_date(data: &[CleanDataWithAnalytics]) -> NaiveDate {
        data.iter()
            .map(|d| d.date)
            .fold(NaiveDate::MIN, NaiveDate::max)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let raw_data_path: &Path = Path::new(INPUT_DATA_PATH_STR);
    let raw_data = RawData::new(raw_data_path)?;
    let clean_data = CleanData::new(&raw_data)?;
    let clean_data_with_analytics = CleanDataWithAnalytics::new(&clean_data, MOVING_AVERAGE_DAYS);

    println!("Loaded {} rows of data", clean_data_with_analytics.len());
    for i in 0..4 {
        println!(
            "Row +{} of clean data: {:?}",
            i + 1,
            clean_data_with_analytics[i]
        );
    }
    for i in 0..4 {
        println!(
            "Row -{} of clean data: {:?}",
            i + 1,
            clean_data_with_analytics[clean_data_with_analytics.len() - i - 1]
        );
    }

    std::fs::create_dir_all(OUTPUT_DIRECTORY)?;
    let output_csv_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_CSV_FILENAME);
    CleanDataWithAnalytics::save_to_csv(&clean_data_with_analytics, &output_csv_path)?;

    // Calculate the max and min values for both dimensions of the chart
    let min_date = CleanDataWithAnalytics::min_date(&clean_data_with_analytics);
    let max_date = CleanDataWithAnalytics::max_date(&clean_data_with_analytics);
    let min_value = CleanDataWithAnalytics::min_value(&clean_data_with_analytics);
    let max_value = CleanDataWithAnalytics::max_value(&clean_data_with_analytics);

    // Build the drawing area
    let output_image_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_IMAGE_FILENAME);
    let root = BitMapBackend::new(&output_image_path, OUTPUT_IMAGE_DIMENSIONS).into_drawing_area();
    root.fill(&CHART_COLOR_BACKGROUND)?;

    let chart_caption = format!("{} from {} to {}", CHART_TITLE, min_date, max_date);

    let mut chart = ChartBuilder::on(&root)
        .caption(chart_caption, CHART_FONT.into_font())
        .build_cartesian_2d(min_date..max_date, min_value..max_value)?;

    chart.draw_series(LineSeries::new(
        clean_data_with_analytics.iter().map(|d| (d.date, d.close)),
        &CHART_COLOR_PRICE_SERIES,
    ))?;

    chart.draw_series(LineSeries::new(
        clean_data_with_analytics
            .iter()
            .map(|d| (d.date, d.two_hundred_wma)),
        &CHART_COLOR_WMA_SERIES,
    ))?;

    root.present()?;

    Ok(())
}
