use chrono::NaiveDate;
use csv::{ReaderBuilder, WriterBuilder};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::write;
use std::path::Path;

// Analytics constants
const MOVING_AVERAGE_DAYS: usize = 1400;

// Input and output constants
const INPUT_DATA_PATH_STR: &str = "./resources/data/historical_data.csv";
const OUTPUT_DIRECTORY: &str = "output/";
const OUTPUT_CSV_FILENAME: &str = "clean_data_with_analytics.csv";
const OUTPUT_HTML_FILENAME: &str = "index.html";
const OUTPUT_LINEAR_IMAGE_FILENAME: &str = "200_week_moving_average_linear.png";
const OUTPUT_LOG_IMAGE_FILENAME: &str = "200_week_moving_average_log.png";

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
const OUTPUT_IMAGES_DIMENSIONS: (u32, u32) = (OUTPUT_IMAGE_WIDTH, OUTPUT_IMAGE_HEIGHT);

#[derive(Debug, Deserialize)]
struct RawData {
    #[serde(rename = "Month")]
    month: String,
    #[serde(rename = "Day")]
    day: String,
    #[serde(rename = "Year")]
    year: String,
    #[serde(rename = "Open")]
    open: String,
    #[serde(rename = "High")]
    high: String,
    #[serde(rename = "Low")]
    low: String,
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
    open: f32,
    high: f32,
    low: f32,
    close: f32,
}

impl CleanData {
    fn new(raw_data: &[RawData]) -> Result<Vec<CleanData>, Box<dyn Error>> {
        raw_data
            .iter()
            .map(|row| {
                let date_str = format!("{} {} {}", row.month, row.day, row.year);
                let date = NaiveDate::parse_from_str(&date_str, "%b %d %Y")?;
                let open: f32 = row.open.replace(",", "").parse()?;
                let high: f32 = row.high.replace(",", "").parse()?;
                let low: f32 = row.low.replace(",", "").parse()?;
                let close: f32 = row.close.replace(",", "").parse()?;
                Ok(CleanData {
                    date,
                    open,
                    high,
                    low,
                    close,
                })
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
struct CleanDataWithAnalytics {
    date: NaiveDate,
    open: f32,
    high: f32,
    low: f32,
    close: f32,
    open_two_hundred_wma: f32,
    high_two_hundred_wma: f32,
    low_two_hundred_wma: f32,
    close_two_hundred_wma: f32,
}

impl CleanDataWithAnalytics {
    fn new(clean_data: &[CleanData], moving_average_size: usize) -> Vec<CleanDataWithAnalytics> {
        let moving_averages = MovingAverages::new(clean_data, moving_average_size);
        clean_data
            .iter()
            .enumerate()
            .map(|(i, row)| CleanDataWithAnalytics {
                date: row.date,
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                open_two_hundred_wma: moving_averages[i].open,
                high_two_hundred_wma: moving_averages[i].high,
                low_two_hundred_wma: moving_averages[i].low,
                close_two_hundred_wma: moving_averages[i].close,
            })
            .collect()
    }

    fn save_to_csv(data: &[CleanDataWithAnalytics], path: &Path) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new().from_path(path)?;
        data.iter()
            .try_for_each(|record| writer.serialize(record))?;
        writer.flush()?;
        Ok(())
    }

    fn min_close(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter().map(|d| d.close).fold(f32::INFINITY, f32::min)
    }

    fn min_close_wma(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.close_two_hundred_wma)
            .fold(f32::INFINITY, f32::min)
    }

    fn max_close(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.close)
            .fold(f32::NEG_INFINITY, f32::max)
    }

    fn max_close_wma(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.close_two_hundred_wma)
            .fold(f32::NEG_INFINITY, f32::max)
    }

    fn min_value(data: &[CleanDataWithAnalytics]) -> f32 {
        f32::min(Self::min_close(data), Self::min_close_wma(data))
    }

    fn max_value(data: &[CleanDataWithAnalytics]) -> f32 {
        f32::max(Self::max_close(data), Self::max_close_wma(data))
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

#[derive(Debug)]
struct MovingAverages {
    open: f32,
    high: f32,
    low: f32,
    close: f32,
}

impl MovingAverages {
    fn new(clean_data: &[CleanData], moving_average_size: usize) -> Vec<MovingAverages> {
        let mut moving_averages: Vec<MovingAverages> = Vec::new();
        for i in 0..clean_data.len() {
            if i == clean_data.len() - 1 {
                moving_averages.push(MovingAverages {
                    open: 0.0,
                    high: 0.0,
                    low: 0.0,
                    close: 0.0,
                });
                break;
            }

            let mut sum_open = 0.0;
            let mut sum_high = 0.0;
            let mut sum_low = 0.0;
            let mut sum_close = 0.0;
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

            for row in clean_data.iter().take(j_end).skip(j_start) {
                sum_open += row.open;
                sum_high += row.high;
                sum_low += row.low;
                sum_close += row.close;
            }

            moving_averages.push(MovingAverages {
                open: format!("{:.2}", sum_open / j_size as f32).parse().unwrap(),
                high: format!("{:.2}", sum_high / j_size as f32).parse().unwrap(),
                low: format!("{:.2}", sum_low / j_size as f32).parse().unwrap(),
                close: format!("{:.2}", sum_close / j_size as f32).parse().unwrap(),
            });
        }
        moving_averages
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let raw_data_path: &Path = Path::new(INPUT_DATA_PATH_STR);
    let raw_data = RawData::new(raw_data_path)?;
    let clean_data = CleanData::new(&raw_data)?;
    let clean_data_with_analytics = CleanDataWithAnalytics::new(&clean_data, MOVING_AVERAGE_DAYS);

    println!("Loaded {} rows of data", clean_data_with_analytics.len());
    clean_data_with_analytics
        .iter()
        .take(4)
        .enumerate()
        .for_each(|(i, row)| {
            println!("Row +{} of clean data: {:?}", i, row);
        });
    clean_data_with_analytics
        .iter()
        .rev()
        .take(4)
        .rev()
        .enumerate()
        .for_each(|(i, row)| {
            println!("Row -{} of clean data: {:?}", 4 - i, row);
        });

    std::fs::create_dir_all(OUTPUT_DIRECTORY)?;
    let output_csv_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_CSV_FILENAME);
    CleanDataWithAnalytics::save_to_csv(&clean_data_with_analytics, &output_csv_path)?;

    // Calculate the max and min values for both dimensions of the chart
    let min_date = CleanDataWithAnalytics::min_date(&clean_data_with_analytics);
    let max_date = CleanDataWithAnalytics::max_date(&clean_data_with_analytics);
    let min_value = CleanDataWithAnalytics::min_value(&clean_data_with_analytics);
    let max_value = CleanDataWithAnalytics::max_value(&clean_data_with_analytics);

    // Build the drawing area for the linear graph
    let output_linear_image_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_LINEAR_IMAGE_FILENAME);
    let root_linear =
        BitMapBackend::new(&output_linear_image_path, OUTPUT_IMAGES_DIMENSIONS).into_drawing_area();
    root_linear.fill(&CHART_COLOR_BACKGROUND)?;

    let chart_caption_linear = format!("Linear scale from {} to {}", min_date, max_date);

    let mut chart_linear = ChartBuilder::on(&root_linear)
        .caption(chart_caption_linear, CHART_FONT.into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min_date..max_date, min_value..max_value)?;

    chart_linear
        .configure_mesh()
        .x_label_formatter(&|date| date.format("%b %Y").to_string())
        .x_max_light_lines(0)
        .y_label_formatter(&|price| format!("{:.0}", price))
        .y_max_light_lines(10)
        .set_all_tick_mark_size(4)
        .draw()?;

    chart_linear.draw_series(LineSeries::new(
        clean_data_with_analytics.iter().map(|d| (d.date, d.close)),
        &CHART_COLOR_PRICE_SERIES,
    ))?;

    chart_linear.draw_series(LineSeries::new(
        clean_data_with_analytics
            .iter()
            .map(|d| (d.date, d.close_two_hundred_wma)),
        &CHART_COLOR_WMA_SERIES,
    ))?;

    root_linear.present()?;

    // Build the drawing area for the linear graph
    let output_log_image_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_LOG_IMAGE_FILENAME);
    let root_log =
        BitMapBackend::new(&output_log_image_path, OUTPUT_IMAGES_DIMENSIONS).into_drawing_area();
    root_log.fill(&CHART_COLOR_BACKGROUND)?;

    let chart_caption_log = format!("Log scale from {} to {}", min_date, max_date);

    let mut chart_log = ChartBuilder::on(&root_log)
        .caption(chart_caption_log, CHART_FONT.into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min_date..max_date, (min_value..max_value).log_scale())?;

    chart_log
        .configure_mesh()
        .x_label_formatter(&|date| date.format("%b %Y").to_string())
        .x_max_light_lines(0)
        .y_label_formatter(&|price| format!("{:.0}", price))
        .set_all_tick_mark_size(4)
        .draw()?;

    chart_log.draw_series(LineSeries::new(
        clean_data_with_analytics.iter().map(|d| (d.date, d.close)),
        &CHART_COLOR_PRICE_SERIES,
    ))?;

    chart_log.draw_series(LineSeries::new(
        clean_data_with_analytics
            .iter()
            .map(|d| (d.date, d.close_two_hundred_wma)),
        &CHART_COLOR_WMA_SERIES,
    ))?;

    root_log.present()?;

    // Generate HTML table rows
    let table_rows: String = clean_data_with_analytics
        .iter()
        .map(|d| {
            format!(
                "<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>",
                d.date,
                d.open,
                d.high,
                d.low,
                d.close,
                d.open_two_hundred_wma,
                d.high_two_hundred_wma,
                d.low_two_hundred_wma,
                d.close_two_hundred_wma
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Generate HTML output
    let output_html_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_HTML_FILENAME);
    let html_content = format!(
        "<html>
            <head>
                <title>{}</title>
            </head>
            <body>
                <h1>{}</h1>
                <a href="https://github.com/bitcoin-tools/btracker">Link to the btracker repo</a>
                <br><br><br>
                <img src='{}' style='border: 1px solid black;' alt='Linear Chart'>
                <br><br><br>
                <img src='{}' style='border: 1px solid black;' alt='Log Chart'>
                <br><br><br>
                <table border='1'>
                    <tr>
                        <th>Date</th>
                        <th>Open</th>
                        <th>High</th>
                        <th>Low</th>
                        <th>Close</th>
                        <th>Open 200-WMA</th>
                        <th>High 200-WMA</th>
                        <th>Low 200-WMA</th>
                        <th>Close 200-WMA</th>
                    </tr>
                    {}
                </table>
            </body>
        </html>",
        CHART_TITLE,
        CHART_TITLE,
        OUTPUT_LINEAR_IMAGE_FILENAME,
        OUTPUT_LOG_IMAGE_FILENAME,
        table_rows
    );
    write(output_html_path, html_content)?;

    Ok(())
}
