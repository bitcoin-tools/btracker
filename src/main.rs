use crate::full_palette::ORANGE;
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
const INPUT_FAVICON_PATH_STR: &str = "resources/favicon.png";
const OUTPUT_DIRECTORY: &str = "output/";
const OUTPUT_CSV_FILENAME: &str = "clean_data_with_analytics.csv";
const OUTPUT_FAVICON_FILENAME: &str = "favicon.png";
const OUTPUT_HTML_FILENAME: &str = "index.html";
const OUTPUT_LINEAR_IMAGE_FILENAME: &str = "200_week_moving_average_linear.png";
const OUTPUT_LOG_IMAGE_FILENAME: &str = "200_week_moving_average_log.png";

// Chart colors and fonts
const CHART_COLOR_BACKGROUND: RGBColor = WHITE;
const CHART_COLOR_PRICE_SERIES: RGBColor = ORANGE;
const CHART_COLOR_WMA_SERIES: RGBColor = BLUE;
const CHART_COLOR_LEGEND_BORDER: RGBColor = BLACK;
const CHART_COLOR_LEGEND_BACKGROUND: RGBColor = WHITE;
const CHART_FONT_LEGEND: (&str, u32) = ("sans-serif", 20);
const CHART_FONT_TITLE: (&str, u32) = ("sans-serif", 32);

// Chart content
const CHART_TITLE: &str = "Price and 200-WMA";
const CHART_LEGEND_PRICE_SERIES_LABEL: &str = "Daily Price";
const CHART_LEGEND_WMA_SERIES_LABEL: &str = "200-WMA";

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
    values: CleanValues,
}

impl CleanData {
    fn new(raw_data: &[RawData]) -> Result<Vec<CleanData>, Box<dyn Error>> {
        raw_data
            .iter()
            .map(|row| {
                let date_str = format!("{} {} {}", row.month, row.day, row.year);
                let date = NaiveDate::parse_from_str(&date_str, "%b %d %Y")?;
                let values = CleanValues::new(row)?;
                Ok(CleanData { date, values })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
struct CleanValues {
    open: f32,
    high: f32,
    low: f32,
    close: f32,
}

impl CleanValues {
    fn new(raw_data: &RawData) -> Result<Self, Box<dyn Error>> {
        let open: f32 = raw_data.open.replace(",", "").parse()?;
        let high: f32 = raw_data.high.replace(",", "").parse()?;
        let low: f32 = raw_data.low.replace(",", "").parse()?;
        let close: f32 = raw_data.close.replace(",", "").parse()?;
        Ok(CleanValues {
            open,
            high,
            low,
            close,
        })
    }
}

#[derive(Debug, Serialize)]
struct CleanDataWithAnalytics {
    date: NaiveDate,
    values: CleanValues,
    moving_averages: MovingAverages,
}

impl CleanDataWithAnalytics {
    fn new(clean_data: &[CleanData], moving_average_size: usize) -> Vec<CleanDataWithAnalytics> {
        let moving_averages = MovingAverages::new(clean_data, moving_average_size);
        clean_data
            .iter()
            .enumerate()
            .map(|(i, row)| CleanDataWithAnalytics {
                date: row.date,
                values: row.values.clone(),
                moving_averages: moving_averages[i].clone(),
            })
            .collect()
    }

    fn save_to_csv(data: &[CleanDataWithAnalytics], path: &Path) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new().from_path(path)?;

        writer.write_record([
            "Date",
            "Open",
            "High",
            "Low",
            "Close",
            "200_WMA_Open",
            "200_WMA_High",
            "200_WMA_Low",
            "200_WMA_Close",
        ])?;

        data.iter().try_for_each(|row| {
            writer.write_record(&[
                row.date.to_string(),
                format!("{:.2}", row.values.open),
                format!("{:.2}", row.values.high),
                format!("{:.2}", row.values.low),
                format!("{:.2}", row.values.close),
                format!("{:.2}", row.moving_averages.open),
                format!("{:.2}", row.moving_averages.high),
                format!("{:.2}", row.moving_averages.low),
                format!("{:.2}", row.moving_averages.close),
            ])
        })?;

        writer.flush()?;
        Ok(())
    }

    fn min_close(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.values.close)
            .fold(f32::INFINITY, f32::min)
    }

    fn min_close_wma(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.moving_averages.close)
            .fold(f32::INFINITY, f32::min)
    }

    fn max_close(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.values.close)
            .fold(f32::NEG_INFINITY, f32::max)
    }

    fn max_close_wma(data: &[CleanDataWithAnalytics]) -> f32 {
        data.iter()
            .map(|d| d.moving_averages.close)
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

#[derive(Debug, Clone, Serialize)]
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
            let mut sum_open = 0.0;
            let mut sum_high = 0.0;
            let mut sum_low = 0.0;
            let mut sum_close = 0.0;

            // The size is the number of days for the weekly moving average calculation.
            //   As a proxy for 200 weeks, we use 1400 days for the simple moving average.
            //   The dates are in reverse chronological order with the newest first.
            //   For most of the data, we will use a size of 1400 days. But for the last
            //   1400 days, we will use the actual number of days available in the data.
            //   The j_start and j_end variables are used to calculate the sum of the
            //   prices for the moving average. The j_ notation refers to for loop syntax.
            //   The j_start is the index of the first row for the moving average.
            //   The j_end is the index of the row after the last row for the moving average.
            //   Meaning for most of the data, j_end is the same as j_start + 1400.
            //   The j_size is the number of rows to include in the moving average.
            //   j_start is includive, so the first row to average is j_start.
            //   j_end is exclusive, so the last row to average is j_end - 1.
            let j_start = i;
            let j_size = usize::min(moving_average_size, clean_data.len() - i);
            let j_end = i + j_size;

            for row in clean_data.iter().take(j_end).skip(j_start) {
                sum_open += row.values.open;
                sum_high += row.values.high;
                sum_low += row.values.low;
                sum_close += row.values.close;
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

    let input_favicon_path = Path::new(INPUT_FAVICON_PATH_STR);
    let output_favicon_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_FAVICON_FILENAME);
    std::fs::copy(input_favicon_path, output_favicon_path)?;

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

    let chart_caption_linear = format!("Linear scale from {min_date} to {max_date}");

    let mut chart_linear = ChartBuilder::on(&root_linear)
        .caption(chart_caption_linear, CHART_FONT_TITLE)
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

    chart_linear
        .draw_series(LineSeries::new(
            clean_data_with_analytics
                .iter()
                .map(|d| (d.date, d.values.close)),
            &CHART_COLOR_PRICE_SERIES,
        ))?
        .label(CHART_LEGEND_PRICE_SERIES_LABEL)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], CHART_COLOR_PRICE_SERIES));

    chart_linear
        .draw_series(
            LineSeries::new(
                clean_data_with_analytics
                    .iter()
                    .map(|d| (d.date, d.moving_averages.close)),
                &CHART_COLOR_WMA_SERIES,
            )
            .point_size(2), // Makes the line thicker
        )?
        .label(CHART_LEGEND_WMA_SERIES_LABEL)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], CHART_COLOR_WMA_SERIES));

    chart_linear
        .configure_series_labels()
        .background_style(CHART_COLOR_LEGEND_BACKGROUND.mix(0.8))
        .border_style(CHART_COLOR_LEGEND_BORDER)
        .label_font(CHART_FONT_LEGEND)
        .position(SeriesLabelPosition::LowerRight)
        .draw()?;

    root_linear.present()?;

    // Build the drawing area for the log graph
    let output_log_image_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_LOG_IMAGE_FILENAME);
    let root_log =
        BitMapBackend::new(&output_log_image_path, OUTPUT_IMAGES_DIMENSIONS).into_drawing_area();
    root_log.fill(&CHART_COLOR_BACKGROUND)?;

    let chart_caption_log = format!("Log scale from {min_date} to {max_date}");

    let mut chart_log = ChartBuilder::on(&root_log)
        .caption(chart_caption_log, CHART_FONT_TITLE)
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min_date..max_date, (min_value..max_value).log_scale())?;

    chart_log
        .configure_mesh()
        .x_label_formatter(&|date| date.format("%b %Y").to_string())
        .x_max_light_lines(0)
        .y_label_formatter(&|price| format!("{price:.0}"))
        .set_all_tick_mark_size(4)
        .draw()?;

    chart_log
        .draw_series(LineSeries::new(
            clean_data_with_analytics
                .iter()
                .map(|d| (d.date, d.values.close)),
            &CHART_COLOR_PRICE_SERIES,
        ))?
        .label(CHART_LEGEND_PRICE_SERIES_LABEL)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], CHART_COLOR_PRICE_SERIES));

    chart_log
        .draw_series(
            LineSeries::new(
                clean_data_with_analytics
                    .iter()
                    .map(|d| (d.date, d.moving_averages.close)),
                CHART_COLOR_WMA_SERIES,
            )
            .point_size(2), // Makes the line thicker
        )?
        .label(CHART_LEGEND_WMA_SERIES_LABEL)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], CHART_COLOR_WMA_SERIES));

    chart_log
        .configure_series_labels()
        .background_style(CHART_COLOR_LEGEND_BACKGROUND.mix(0.8))
        .border_style(CHART_COLOR_LEGEND_BORDER)
        .label_font(CHART_FONT_LEGEND)
        .position(SeriesLabelPosition::MiddleRight)
        .draw()?;

    root_log.present()?;

    // Generate HTML table rows
    let table_rows: String = clean_data_with_analytics
        .iter()
        .map(|d| {
            format!(
                "<tr>
                    <td>{}</td>
                    <td>{:.2}</td>
                    <td>{:.2}</td>
                    <td>{:.2}</td>
                    <td>{:.2}</td>
                    <td>{:.2}</td>
                    <td>{:.2}</td>
                    <td>{:.2}</td>
                    <td>{:.2}</td>
                </tr>",
                d.date,
                d.values.open,
                d.values.high,
                d.values.low,
                d.values.close,
                d.moving_averages.open,
                d.moving_averages.high,
                d.moving_averages.low,
                d.moving_averages.close
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Generate HTML output
    let output_html_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_HTML_FILENAME);
    let html_content = format!(
        "<!DOCTYPE html>
        <html>
            <head>
                <title>{CHART_TITLE}</title>
                <link rel='icon' type='image/png' href='{OUTPUT_FAVICON_FILENAME}'>
                <style>
                    th {{
                        padding: 5px;
                        vertical-align: bottom;
                    }}
                    td {{
                        padding: 5px;
                        text-align: right;
                    }}
                </style>
            </head>
            <body>
                <h1>{CHART_TITLE}</h1>
                <a href='https://github.com/bitcoin-tools/btracker'>Link to the btracker repo</a>
                <br><br>
                <img src='{OUTPUT_LINEAR_IMAGE_FILENAME}' style='border: 2px solid black;' alt='Linear Chart'>
                <br><br>
                <img src='{OUTPUT_LOG_IMAGE_FILENAME}' style='border: 2px solid black;' alt='Log Chart'>
                <br><br>
                <a href='https://github.com/bitcoin-tools/btracker/raw/gh-pages/clean_data_with_analytics.csv'>Link to CSV data</a>
                <br><br>
                <table style='border-width: 1px; border-style: solid; border-color: black;'>
                    <tr>
                        <th rowspan='2'>Date</th>
                        <th colspan='4'>Daily Prices</th>
                        <th colspan='4'>200-Week Moving Averages</th>
                    </tr>
                    <tr>
                        <th>Open</th>
                        <th>High</th>
                        <th>Low</th>
                        <th>Close</th>
                        <th>Open</th>
                        <th>High</th>
                        <th>Low</th>
                        <th>Close</th>
                    </tr>
                    {table_rows}
                </table>
            </body>
        </html>"
    );
    write(output_html_path, html_content)?;

    Ok(())
}
