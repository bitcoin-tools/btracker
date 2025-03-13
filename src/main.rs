use crate::full_palette::ORANGE;
use chrono::NaiveDate;
use csv::{ReaderBuilder, WriterBuilder};
use num_format::{Locale, ToFormattedString};
use plotters::prelude::*;
use std::error::Error;
use std::fs::write;
use std::path::Path;

// Analytics constants
const MOVING_AVERAGE_DAYS: usize = 1400;

// Input and output constants
const REPOSITORY_URL: &str = "https://github.com/bitcoin-tools/btracker";
const INPUT_DATA_PATH_STR: &str = "./resources/data/historical_data.tsv";
const INPUT_FAVICON_PATH_STR: &str = "resources/media/favicon.png";
const OUTPUT_DIRECTORY: &str = "output/";
const OUTPUT_CSV_FILENAME: &str = "processed_data.csv";
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
const CHART_CAPTION_FONT_NAME: &str = "sans-serif";
const CHART_CAPTION_FONT_SIZE: u32 = 32;
const CHART_CAPTION_FONT_STYLE: FontStyle = FontStyle::Normal;
const CHART_CAPTION_FONT_COLOR: RGBColor = BLUE;

// Chart content
const CHART_TITLE: &str = "Price and 200-WMA";
const CHART_LEGEND_PRICE_SERIES_LABEL: &str = "Daily Price";
const CHART_LEGEND_WMA_SERIES_LABEL: &str = "200-WMA";

// Image dimensions
const OUTPUT_IMAGE_WIDTH: u32 = 1024;
const OUTPUT_IMAGE_HEIGHT: u32 = 600;
// TODO: try others like 1024x768, 800x600, 640x480, 320x240, 1280x1024, 1920x1080
const OUTPUT_IMAGES_DIMENSIONS: (u32, u32) = (OUTPUT_IMAGE_WIDTH, OUTPUT_IMAGE_HEIGHT);

// Helper function to format numbers with commas and decimal places
fn format_number_with_commas(value: f32, decimal_places: usize) -> String {
    let integer_part = value.trunc() as i64;
    let decimal_part = (value.fract() * 10f32.powi(decimal_places as i32)).abs() as i64;
    format!(
        "{}.{:0width$}",
        integer_part.to_formatted_string(&Locale::en),
        decimal_part,
        width = decimal_places
    )
}

#[derive(Debug, Clone)]
struct CleanData {
    date: NaiveDate,
    values: CleanValues,
}

impl CleanData {
    fn new(path: &Path) -> Result<Vec<CleanData>, Box<dyn Error>> {
        let mut clean_data_vec: Vec<CleanData> = Vec::new();

        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_path(path)?;

        for result in reader.records() {
            let record = result?;
            let date_str = format!("{} {} {}", &record[0], &record[1], &record[2]);
            let date = NaiveDate::parse_from_str(&date_str, "%b %d %Y")?;
            let values = CleanValues::new(&record)?;
            clean_data_vec.push(CleanData { date, values });
        }

        Ok(clean_data_vec)
    }
}

#[derive(Debug, Clone)]
struct CleanValues {
    open: f32,
    high: f32,
    low: f32,
    close: f32,
}

impl CleanValues {
    fn new(record: &csv::StringRecord) -> Result<Self, Box<dyn Error>> {
        let open: f32 = record[3].replace(',', "").parse()?;
        let high: f32 = record[4].replace(',', "").parse()?;
        let low: f32 = record[5].replace(',', "").parse()?;
        let close: f32 = record[6].replace(',', "").parse()?;

        Ok(CleanValues {
            open,
            high,
            low,
            close,
        })
    }
}

#[derive(Debug, Clone)]
struct PriceChanges {
    dollar_swing_same_day: f32,
    percent_swing_same_day: f32,
    dollar_change_1_day: f32,
    percent_change_1_day: f32,
    dollar_change_200_week: f32,
    percent_change_200_week: f32,
}

impl PriceChanges {
    fn new(clean_data: &[CleanData]) -> Vec<PriceChanges> {
        let mut price_changes_vec: Vec<PriceChanges> = Vec::new();
        for i in 0..clean_data.len() {
            let price_now = clean_data[i].values.close;

            let dollar_swing_same_day = clean_data[i].values.high - clean_data[i].values.low;
            let percent_swing_same_day = 100.0 * (dollar_swing_same_day / price_now);

            let i_previous_1_day = usize::min(i + 1, clean_data.len() - 1);
            let price_previous_1_day = clean_data[i_previous_1_day].values.close;
            let dollar_change_1_day =
                PriceChanges::get_price_change(price_now, price_previous_1_day, false);
            let percent_change_1_day =
                PriceChanges::get_price_change(price_now, price_previous_1_day, true);

            let i_previous_200_week = usize::min(i + MOVING_AVERAGE_DAYS, clean_data.len() - 1);
            let price_previous_200_week = clean_data[i_previous_200_week].values.close;
            let dollar_change_200_week =
                PriceChanges::get_price_change(price_now, price_previous_200_week, false);
            let percent_change_200_week =
                PriceChanges::get_price_change(price_now, price_previous_200_week, true);

            price_changes_vec.push(PriceChanges {
                dollar_swing_same_day,
                percent_swing_same_day,
                dollar_change_1_day,
                percent_change_1_day,
                dollar_change_200_week,
                percent_change_200_week,
            });
        }
        price_changes_vec
    }

    fn get_price_change(price_now: f32, price_previous: f32, report_percent: bool) -> f32 {
        if report_percent {
            100.0 * (price_now / price_previous - 1.0)
        } else {
            price_now - price_previous
        }
    }
}

#[derive(Debug)]
struct PriceChangesHistogram {
    below_negative_20_percent: u32,
    between_negative_20_and_15_percent: u32,
    between_negative_15_and_10_percent: u32,
    between_negative_10_and_5_percent: u32,
    between_negative_5_and_2_percent: u32,
    between_netative_2_and_0_percent: u32,
    between_0_and_2_percent: u32,
    between_2_and_5_percent: u32,
    between_5_and_10_percent: u32,
    between_10_and_15_percent: u32,
    between_15_and_20_percent: u32,
    above_20_percent: u32,
    total_days: u32,
}

impl PriceChangesHistogram {
    fn new(data: &[CleanDataWithAnalytics]) -> Self {
        let mut histogram = PriceChangesHistogram {
            total_days: 0,
            below_negative_20_percent: 0,
            between_negative_20_and_15_percent: 0,
            between_negative_15_and_10_percent: 0,
            between_negative_10_and_5_percent: 0,
            between_negative_5_and_2_percent: 0,
            between_netative_2_and_0_percent: 0,
            between_0_and_2_percent: 0,
            between_2_and_5_percent: 0,
            between_5_and_10_percent: 0,
            between_10_and_15_percent: 0,
            between_15_and_20_percent: 0,
            above_20_percent: 0,
        };

        for d in data {
            histogram.total_days += 1;
            let percent_change = d.price_changes.percent_change_1_day;
            if percent_change < -20.0 {
                histogram.below_negative_20_percent += 1;
            } else if (-20.0..-15.0).contains(&percent_change) {
                histogram.between_negative_20_and_15_percent += 1;
            } else if (-15.0..-10.0).contains(&percent_change) {
                histogram.between_negative_15_and_10_percent += 1;
            } else if (-10.0..-5.0).contains(&percent_change) {
                histogram.between_negative_10_and_5_percent += 1;
            } else if (-5.0..-2.0).contains(&percent_change) {
                histogram.between_negative_5_and_2_percent += 1;
            } else if (-2.0..0.0).contains(&percent_change) {
                histogram.between_netative_2_and_0_percent += 1;
            } else if (0.0..2.0).contains(&percent_change) {
                histogram.between_0_and_2_percent += 1;
            } else if (2.0..5.0).contains(&percent_change) {
                histogram.between_2_and_5_percent += 1;
            } else if (5.0..10.0).contains(&percent_change) {
                histogram.between_5_and_10_percent += 1;
            } else if (10.0..15.0).contains(&percent_change) {
                histogram.between_10_and_15_percent += 1;
            } else if (15.0..20.0).contains(&percent_change) {
                histogram.between_15_and_20_percent += 1;
            } else if percent_change >= 20.0 {
                histogram.above_20_percent += 1;
            }
        }
        histogram
    }

    fn save_to_csv(data: PriceChangesHistogram, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new().from_path(path)?;

        writer.write_record(["One-Day Price Change", "Days"])?;
        writer.write_record(["Below -20%", data.below_negative_20_percent])?;
        writer.write_record(["-20% to -15%", data.between_negative_20_and_15_percent])?;
        writer.write_record(["-15% to -10%", data.between_negative_15_and_10_percent])?;
        writer.write_record(["-10% to -5%", data.between_negative_10_and_5_percent])?;
        writer.write_record(["-5% to -2%", data.between_negative_5_and_2_percent])?;
        writer.write_record(["-2% to 0%", data.between_negative_2_and_0_percent])?;
        writer.write_record(["0% to 2%", data.between_0_and_2_percent])?;
        writer.write_record(["2% to 5%", data.between_2_and_5_percent])?;
        writer.write_record(["5% to 10%", data.between_5_and_10_percent])?;
        writer.write_record(["10% to 15%", data.between_10_and_15_percent])?;
        writer.write_record(["15% to 20%", data.between_15_and_20_percent])?;
        writer.write_record(["Above 20%", data.above_20_percent])?;

        writer.flush()?;
        Ok(())
    }

    fn to_html_table(&self) -> String {
        format!(
            "<table>
                <thead>
                    <tr>
                        <th colspan='2'>Price Change Histogram</th>
                    <tr>
                        <th>1-Day Change</th>
                        <th>Days</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>Below -20%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>-20% to -15%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>-15% to -10%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>-10% to -5%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>-5% to -2%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>-2% to 0%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>0% to 2%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>2% to 5%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>5% to 10%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>10% to 15%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>15% to 20%</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>Above 20%</td>
                        <td>{}</td>
                    </tr>
                    <tr class='histogram-footer'>
                        <td>Total Days</td>
                        <td>{}</td>
                    </tr>
                </tbody>
            </table>",
            self.below_negative_20_percent,
            self.between_negative_20_and_15_percent,
            self.between_negative_15_and_10_percent,
            self.between_negative_10_and_5_percent,
            self.between_negative_5_and_2_percent,
            self.between_netative_2_and_0_percent,
            self.between_0_and_2_percent,
            self.between_2_and_5_percent,
            self.between_5_and_10_percent,
            self.between_10_and_15_percent,
            self.between_15_and_20_percent,
            self.above_20_percent,
            self.total_days
        )
    }
}

#[derive(Debug)]
struct CleanDataWithAnalytics {
    date: NaiveDate,
    values: CleanValues,
    price_changes: PriceChanges,
    moving_averages: MovingAverages,
}

impl CleanDataWithAnalytics {
    fn new(clean_data: &[CleanData], moving_average_size: usize) -> Vec<CleanDataWithAnalytics> {
        let price_changes = PriceChanges::new(clean_data);
        let moving_averages = MovingAverages::new(clean_data, moving_average_size);
        clean_data
            .iter()
            .enumerate()
            .map(|(i, row)| CleanDataWithAnalytics {
                date: row.date,
                values: row.values.clone(),
                price_changes: price_changes[i].clone(),
                moving_averages: moving_averages[i].clone(),
            })
            .collect()
    }

    fn create_linear_chart(
        data: &[CleanDataWithAnalytics],
        output_path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let min_date = Self::min_date(data);
        let max_date = Self::max_date(data);
        let min_value = Self::min_value(data);
        let max_value = Self::max_value(data);

        let root = BitMapBackend::new(output_path, OUTPUT_IMAGES_DIMENSIONS).into_drawing_area();
        root.fill(&CHART_COLOR_BACKGROUND)?;

        let chart_caption_font: TextStyle = FontDesc::new(
            FontFamily::Name(CHART_CAPTION_FONT_NAME),
            f64::from(CHART_CAPTION_FONT_SIZE),
            CHART_CAPTION_FONT_STYLE,
        )
        .color(&CHART_CAPTION_FONT_COLOR);

        let chart_caption_label = format!("Linear scale from {min_date} to {max_date}");
        let mut chart = ChartBuilder::on(&root)
            .caption(chart_caption_label, chart_caption_font.clone())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(min_date..max_date, min_value..max_value)?;

        chart
            .configure_mesh()
            .x_label_formatter(&|date| date.format("%b %Y").to_string())
            .x_max_light_lines(0)
            .y_label_formatter(&|price| format!("{:.0}", price))
            .y_max_light_lines(10)
            .set_all_tick_mark_size(4)
            .draw()?;

        chart
            .draw_series(LineSeries::new(
                data.iter().map(|d| (d.date, d.values.close)),
                &CHART_COLOR_PRICE_SERIES,
            ))?
            .label(CHART_LEGEND_PRICE_SERIES_LABEL)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], CHART_COLOR_PRICE_SERIES));

        chart
            .draw_series(
                LineSeries::new(
                    data.iter().map(|d| (d.date, d.moving_averages.close)),
                    &CHART_COLOR_WMA_SERIES,
                )
                .point_size(2),
            )?
            .label(CHART_LEGEND_WMA_SERIES_LABEL)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], CHART_COLOR_WMA_SERIES));

        chart
            .configure_series_labels()
            .background_style(CHART_COLOR_LEGEND_BACKGROUND.mix(0.8))
            .border_style(CHART_COLOR_LEGEND_BORDER)
            .label_font(CHART_FONT_LEGEND)
            .position(SeriesLabelPosition::LowerRight)
            .draw()?;

        root.present()?;
        Ok(())
    }

    fn create_log_chart(
        data: &[CleanDataWithAnalytics],
        output_path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let min_date = Self::min_date(data);
        let max_date = Self::max_date(data);
        let min_value = Self::min_value(data);
        let max_value = Self::max_value(data);

        let root = BitMapBackend::new(output_path, OUTPUT_IMAGES_DIMENSIONS).into_drawing_area();
        root.fill(&CHART_COLOR_BACKGROUND)?;

        let chart_caption_font: TextStyle = FontDesc::new(
            FontFamily::Name(CHART_CAPTION_FONT_NAME),
            f64::from(CHART_CAPTION_FONT_SIZE),
            CHART_CAPTION_FONT_STYLE,
        )
        .color(&CHART_CAPTION_FONT_COLOR);

        let chart_caption_label = format!("Log scale from {min_date} to {max_date}");
        let mut chart = ChartBuilder::on(&root)
            .caption(chart_caption_label, chart_caption_font.clone())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(min_date..max_date, (min_value..max_value).log_scale())?;

        chart
            .configure_mesh()
            .x_label_formatter(&|date| date.format("%b %Y").to_string())
            .x_max_light_lines(0)
            .y_label_formatter(&|price| format!("{price:.0}"))
            .set_all_tick_mark_size(4)
            .draw()?;

        chart
            .draw_series(LineSeries::new(
                data.iter().map(|d| (d.date, d.values.close)),
                &CHART_COLOR_PRICE_SERIES,
            ))?
            .label(CHART_LEGEND_PRICE_SERIES_LABEL)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], CHART_COLOR_PRICE_SERIES));

        chart
            .draw_series(
                LineSeries::new(
                    data.iter().map(|d| (d.date, d.moving_averages.close)),
                    &CHART_COLOR_WMA_SERIES,
                )
                .point_size(2),
            )?
            .label(CHART_LEGEND_WMA_SERIES_LABEL)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], CHART_COLOR_WMA_SERIES));

        chart
            .configure_series_labels()
            .background_style(CHART_COLOR_LEGEND_BACKGROUND.mix(0.8))
            .border_style(CHART_COLOR_LEGEND_BORDER)
            .label_font(CHART_FONT_LEGEND)
            .position(SeriesLabelPosition::MiddleRight)
            .draw()?;

        root.present()?;
        Ok(())
    }

    fn save_to_csv(data: &[CleanDataWithAnalytics], path: &Path) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new().from_path(path)?;

        writer.write_record([
            "Date",
            "Open",
            "High",
            "Low",
            "Close",
            "Price_Change_Dollar_Same_Day_Swing",
            "Price_Change_Percent_Same_Day_Swing",
            "Price_Change_Dollar_1_Day",
            "Price_Change_Percent_1_Day",
            "Price_Change_Dollar_200_Week",
            "Price_Change_Percent_200_Week",
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
                format!("{:.2}", row.price_changes.dollar_swing_same_day),
                format!("{:.1}", row.price_changes.percent_swing_same_day),
                format!("{:.2}", row.price_changes.dollar_change_1_day),
                format!("{:.2}", row.price_changes.percent_change_1_day),
                format!("{:.2}", row.price_changes.dollar_change_200_week),
                format!("{:.2}", row.price_changes.percent_change_200_week),
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

#[derive(Debug, Clone)]
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
    let clean_data = CleanData::new(raw_data_path)?;
    let clean_data_with_analytics = CleanDataWithAnalytics::new(&clean_data, MOVING_AVERAGE_DAYS);

    println!("Loaded {} rows of data", clean_data_with_analytics.len());
    clean_data_with_analytics
        .iter()
        .take(2)
        .enumerate()
        .for_each(|(i, row)| {
            println!("Row {i} of clean_data: {row:?}");
        });
    clean_data_with_analytics
        .iter()
        .rev()
        .take(2)
        .rev()
        .enumerate()
        .for_each(|(i, row)| {
            println!("Row -{} of clean_data: {:?}", 2 - i, row);
        });

    std::fs::create_dir_all(OUTPUT_DIRECTORY)?;

    let input_favicon_path = Path::new(INPUT_FAVICON_PATH_STR);
    let output_favicon_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_FAVICON_FILENAME);
    std::fs::copy(input_favicon_path, output_favicon_path)?;

    let output_csv_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_CSV_FILENAME);
    CleanDataWithAnalytics::save_to_csv(&clean_data_with_analytics, &output_csv_path)?;

    let output_linear_image_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_LINEAR_IMAGE_FILENAME);
    CleanDataWithAnalytics::create_linear_chart(
        &clean_data_with_analytics,
        &output_linear_image_path,
    )?;

    let output_log_image_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_LOG_IMAGE_FILENAME);
    CleanDataWithAnalytics::create_log_chart(&clean_data_with_analytics, &output_log_image_path)?;

    let histogram = PriceChangesHistogram::new(&clean_data_with_analytics);
    let histogram_html_table = histogram.to_html_table();

    let table_rows: String = clean_data_with_analytics
        .iter()
        .map(|d| {
            format!(
                "                        <tr>
                            <td>{}</td>
                            <td>{}</td>
                            <td>{}</td>
                            <td>{}</td>
                            <td>{}</td>
                            <td class='wma-column'>{}</td>
                            <td>{}</td>
                            <td>{} %</td>
                            <td>{}</td>
                            <td>{} %</td>
                            <td>{}</td>
                            <td>{} %</td>
                        </tr>",
                d.date,
                format_number_with_commas(d.values.open, 2),
                format_number_with_commas(d.values.high, 2),
                format_number_with_commas(d.values.low, 2),
                format_number_with_commas(d.values.close, 2),
                format_number_with_commas(d.moving_averages.close, 2),
                format_number_with_commas(d.price_changes.dollar_swing_same_day, 2),
                format_number_with_commas(d.price_changes.percent_swing_same_day, 1),
                format_number_with_commas(d.price_changes.dollar_change_1_day, 2),
                format_number_with_commas(d.price_changes.percent_change_1_day, 1),
                format_number_with_commas(d.price_changes.dollar_change_200_week, 2),
                format_number_with_commas(d.price_changes.percent_change_200_week, 1)
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Generate HTML output
    let output_csv_url: String = format!("{REPOSITORY_URL}/raw/gh-pages/{OUTPUT_CSV_FILENAME}");
    let output_html_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_HTML_FILENAME);
    let html_content = format!(
        "<!DOCTYPE html>
        <html>
            <head>
                <title>{CHART_TITLE}</title>
                <link rel='icon' type='image/png' href='{OUTPUT_FAVICON_FILENAME}'>
                <style>
                    tr.histogram-footer {{
                        background-color: whitesmoke;
                        border: 2px solid black;
                        font-weight: bold;
                    }}
                    th.wma-column {{
                        background-color: whitesmoke;
                        border: 3px solid blue;
                        padding: 7px;
                    }}
                    td.wma-column {{
                        background-color: whitesmoke;
                        border-left: 3px solid blue;
                        border-right: 3px solid blue;
                        font-weight: bold;
                    }}
                    img {{
                        border: 2px solid black;
                    }}
                    table {{
                        border-color: black;
                        border-style: solid;
                        border-width: 1px;
                    }}
                    th {{
                        border: 1px solid black;
                        padding: 5px;
                        vertical-align: bottom;
                    }}
                    td {{
                        border: 1px solid black;
                        padding: 5px;
                        text-align: right;
                    }}
                </style>
            </head>
            <body>
                <h1>{CHART_TITLE}</h1>
                <a href='{REPOSITORY_URL}'>Link to the btracker repo</a>
                <br><br>
                <img src='{OUTPUT_LINEAR_IMAGE_FILENAME}' alt='Linear Chart'>
                <br><br>
                <img src='{OUTPUT_LOG_IMAGE_FILENAME}' alt='Log Chart'>
                <br><br>
                {histogram_html_table}
                <br><br>
                <a href='{output_csv_url}'>Link to CSV data</a>
                <br><br>
                <table>
                    <thead>
                        <tr>
                            <th rowspan='2'>Date</th>
                            <th colspan='4'>Daily Prices</th>
                            <th rowspan='2' class='wma-column'>200-Week<br>Moving<br>Average</th>
                            <th colspan='2'>Same-Day Swing</th>
                            <th colspan='2'>1-Day Change</th>
                            <th colspan='2'>200-Week Change</th>
                        </tr>
                        <tr>
                            <th>Open</th>
                            <th>High</th>
                            <th>Low</th>
                            <th>Close</th>
                            <th>$ Change</th>
                            <th>% Change</th>
                            <th>$ Change</th>
                            <th>% Change</th>
                            <th>$ Change</th>
                            <th>% Change</th>
                        </tr>
                    </thead>
                    <tbody>
{table_rows}
                    </tbody>
                </table>
            </body>
        </html>"
    );
    write(output_html_path, html_content)?;

    Ok(())
}
