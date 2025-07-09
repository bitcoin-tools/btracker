use crate::full_palette::ORANGE;
use chrono::{Datelike, NaiveDate};
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
const INPUT_CSS_PATH_STR: &str = "resources/web/style.css";
const INPUT_FAVICON_PATH_STR: &str = "resources/web/favicon.png";
const OUTPUT_DIRECTORY: &str = "output/";
const OUTPUT_PRICE_ANALYTICS_CSV_FILENAME: &str = "processed_data.csv";
const OUTPUT_HISTOGRAM_CSV_FILENAME: &str = "histogram.csv";
const OUTPUT_YEARLY_SUMMARY_CSV_FILENAME: &str = "yearly_summary.csv";
const OUTPUT_CSS_FILENAME: &str = "style.css";
const OUTPUT_FAVICON_FILENAME: &str = "favicon.png";
const OUTPUT_HTML_FILENAME: &str = "index.html";
const OUTPUT_LINEAR_IMAGE_FILENAME: &str = "200_week_moving_average_linear.png";
const OUTPUT_LOG_IMAGE_FILENAME: &str = "200_week_moving_average_log.png";
const OUTPUT_HISTOGRAM_IMAGE_FILENAME: &str = "price_changes_histogram.png";

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
    volume: f32,
}

impl CleanValues {
    fn new(record: &csv::StringRecord) -> Result<Self, Box<dyn Error>> {
        let open: f32 = record[3].replace(',', "").parse()?;
        let high: f32 = record[4].replace(',', "").parse()?;
        let low: f32 = record[5].replace(',', "").parse()?;
        let close: f32 = record[6].replace(',', "").parse()?;
        let volume: f32 = record[8].replace(',', "").parse()?;

        Ok(CleanValues {
            open,
            high,
            low,
            close,
            volume,
        })
    }
}

#[derive(Debug, Clone)]
struct PriceChanges {
    two_hundred_wma_dollar_change_1_day: f32,
    two_hundred_wma_percent_change_1_day: f32,
    dollar_change_200_week: f32,
    percent_change_200_week: f32,
    dollar_swing_same_day: f32,
    percent_swing_same_day: f32,
    dollar_change_1_day: f32,
    percent_change_1_day: f32,
}

impl PriceChanges {
    fn new(clean_data: &[CleanData], moving_averages: &[MovingAverages]) -> Vec<PriceChanges> {
        let mut price_changes_vec: Vec<PriceChanges> = Vec::new();
        for i in 0..clean_data.len() {
            let i_previous_1_day = usize::min(i + 1, clean_data.len() - 1);

            let wma_now = moving_averages[i].close;
            let wma_previous_1_day = moving_averages[i_previous_1_day].close;
            let two_hundred_wma_dollar_change_1_day =
                PriceChanges::get_price_change(wma_now, wma_previous_1_day, false);
            let two_hundred_wma_percent_change_1_day =
                PriceChanges::get_price_change(wma_now, wma_previous_1_day, true);

            let price_now = clean_data[i].values.close;
            let dollar_swing_same_day = clean_data[i].values.high - clean_data[i].values.low;
            let percent_swing_same_day = 100.0 * (dollar_swing_same_day / price_now);

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
                two_hundred_wma_dollar_change_1_day,
                two_hundred_wma_percent_change_1_day,
                dollar_change_200_week,
                percent_change_200_week,
                dollar_swing_same_day,
                percent_swing_same_day,
                dollar_change_1_day,
                percent_change_1_day,
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

#[derive(Debug, Clone)]
struct PriceChangesHistogram {
    below_negative_15_percent: usize,
    between_negative_15_and_12_percent: usize,
    between_negative_12_and_9_percent: usize,
    between_negative_9_and_6_percent: usize,
    between_negative_6_and_3_percent: usize,
    between_negative_3_and_0_percent: usize,
    between_0_and_3_percent: usize,
    between_3_and_6_percent: usize,
    between_6_and_9_percent: usize,
    between_9_and_12_percent: usize,
    between_12_and_15_percent: usize,
    above_15_percent: usize,
    total_days: usize,
}

impl PriceChangesHistogram {
    fn new(data: &[CleanDataWithAnalytics]) -> Self {
        let mut histogram = PriceChangesHistogram {
            below_negative_15_percent: 0,
            between_negative_15_and_12_percent: 0,
            between_negative_12_and_9_percent: 0,
            between_negative_9_and_6_percent: 0,
            between_negative_6_and_3_percent: 0,
            between_negative_3_and_0_percent: 9,
            between_0_and_3_percent: 0,
            between_3_and_6_percent: 0,
            between_6_and_9_percent: 0,
            between_9_and_12_percent: 0,
            between_12_and_15_percent: 0,
            above_15_percent: 0,
            total_days: 0,
        };

        for d in data {
            histogram.total_days += 1;
            let percent_change = d.price_changes.percent_change_1_day;
            if percent_change < -15.0 {
                histogram.below_negative_15_percent += 1;
            } else if (-15.0..-12.0).contains(&percent_change) {
                histogram.between_negative_15_and_12_percent += 1;
            } else if (-12.0..-9.0).contains(&percent_change) {
                histogram.between_negative_12_and_9_percent += 1;
            } else if (-9.0..-6.0).contains(&percent_change) {
                histogram.between_negative_9_and_6_percent += 1;
            } else if (-6.0..-3.0).contains(&percent_change) {
                histogram.between_negative_6_and_3_percent += 1;
            } else if (-3.0..0.0).contains(&percent_change) {
                histogram.between_negative_3_and_0_percent += 1;
            } else if (0.0..3.0).contains(&percent_change) {
                histogram.between_0_and_3_percent += 1;
            } else if (3.0..6.0).contains(&percent_change) {
                histogram.between_3_and_6_percent += 1;
            } else if (6.0..9.0).contains(&percent_change) {
                histogram.between_6_and_9_percent += 1;
            } else if (9.0..12.0).contains(&percent_change) {
                histogram.between_9_and_12_percent += 1;
            } else if (12.0..15.0).contains(&percent_change) {
                histogram.between_12_and_15_percent += 1;
            } else if percent_change >= 15.0 {
                histogram.above_15_percent += 1;
            }
        }
        assert_eq!(
            data.len(),
            histogram.total_days,
            "The clean data length does not match the histogram total days"
        );
        histogram
    }

    fn save_to_csv(data: PriceChangesHistogram, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new().from_path(path)?;

        writer.write_record(["One-Day Price Change", "Days"])?;
        writer.write_record(["Below -15%", &data.below_negative_15_percent.to_string()])?;
        writer.write_record([
            "-15% to -12%",
            &data.between_negative_15_and_12_percent.to_string(),
        ])?;
        writer.write_record([
            "-12% to -9%",
            &data.between_negative_12_and_9_percent.to_string(),
        ])?;
        writer.write_record([
            "-9% to -6%",
            &data.between_negative_9_and_6_percent.to_string(),
        ])?;
        writer.write_record([
            "-6% to -3%",
            &data.between_negative_6_and_3_percent.to_string(),
        ])?;
        writer.write_record([
            "-3% to 0%",
            &data.between_negative_3_and_0_percent.to_string(),
        ])?;
        writer.write_record(["0% to 3%", &data.between_0_and_3_percent.to_string()])?;
        writer.write_record(["3% to 6%", &data.between_3_and_6_percent.to_string()])?;
        writer.write_record(["6% to 9%", &data.between_6_and_9_percent.to_string()])?;
        writer.write_record(["9% to 12%", &data.between_9_and_12_percent.to_string()])?;
        writer.write_record(["12% to 15%", &data.between_12_and_15_percent.to_string()])?;
        writer.write_record(["Above 15%", &data.above_15_percent.to_string()])?;

        writer.flush()?;
        Ok(())
    }

    fn create_chart(&self, output_path: &Path) -> Result<(), Box<dyn Error>> {
        let root = BitMapBackend::new(&output_path, OUTPUT_IMAGES_DIMENSIONS).into_drawing_area();
        root.fill(&CHART_COLOR_BACKGROUND)?;

        let bins = vec![
            ("<-15%", self.below_negative_15_percent as f32),
            (
                "-15% to -12%",
                self.between_negative_15_and_12_percent as f32,
            ),
            ("-12% to -9%", self.between_negative_12_and_9_percent as f32),
            ("-9% to -6%", self.between_negative_9_and_6_percent as f32),
            ("-6% to -3%", self.between_negative_6_and_3_percent as f32),
            ("-3% to 0%", self.between_negative_3_and_0_percent as f32),
            ("0% to 3%", self.between_0_and_3_percent as f32),
            ("3% to 6%", self.between_3_and_6_percent as f32),
            ("6% to 9%", self.between_6_and_9_percent as f32),
            ("9% to 12%", self.between_9_and_12_percent as f32),
            ("12% to 15%", self.between_12_and_15_percent as f32),
            (">15%", self.above_15_percent as f32),
        ];

        let max_days = bins
            .iter()
            .map(|(_, count)| count)
            .fold(0.0_f32, |a, b| a.max(*b));
        let y_axis_height = 1.1 * max_days;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Daily Price Changes Histogram",
                (CHART_CAPTION_FONT_NAME, CHART_CAPTION_FONT_SIZE)
                    .into_font()
                    .color(&CHART_CAPTION_FONT_COLOR),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0..bins.len(), 0.0..y_axis_height)?;

        chart
            .configure_mesh()
            .x_label_formatter(&|x| {
                if *x < bins.len() {
                    bins[*x].0.to_string()
                } else {
                    "".to_string()
                }
            })
            .x_labels(bins.len())
            .y_label_formatter(&|y| format!("{y:.0}"))
            .x_desc("Percentage Change")
            .y_desc("Number of Days")
            .axis_desc_style(("sans-serif", 15))
            .draw()?;

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(CHART_COLOR_PRICE_SERIES.filled())
                .data(bins.iter().enumerate().map(|(i, (_, count))| (i, *count))),
        )?;

        root.present()?;
        Ok(())
    }

    fn to_html_table(&self) -> String {
        format!(
            "    <table class='inline-table'>
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
          <td>Below -15%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>-15% to -12%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>-12% to -9%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>-9% to -6%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>-6% to -3%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>-3% to 0%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>0% to 3%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>3% to 6%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>6% to 9%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>9% to 12%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>12% to 15%</td>
          <td>{}</td>
        </tr>
        <tr>
          <td>Above 15%</td>
          <td>{}</td>
        </tr>
        <tr class='histogram-footer'>
          <td>Total Days</td>
          <td>{}</td>
        </tr>
      </tbody>
    </table>",
            self.below_negative_15_percent,
            self.between_negative_15_and_12_percent,
            self.between_negative_12_and_9_percent,
            self.between_negative_9_and_6_percent,
            self.between_negative_6_and_3_percent,
            self.between_negative_3_and_0_percent,
            self.between_0_and_3_percent,
            self.between_3_and_6_percent,
            self.between_6_and_9_percent,
            self.between_9_and_12_percent,
            self.between_12_and_15_percent,
            self.above_15_percent,
            self.total_days
        )
    }
}

#[derive(Debug, Clone)]
struct YearlySummary {
    year: i32,
    open: Option<f32>,
    high: f32,
    low: f32,
    close: Option<f32>,
    volume: f32,
}

impl YearlySummary {
    fn new(data: &[CleanDataWithAnalytics]) -> Vec<YearlySummary> {
        let mut yearly_summaries: Vec<YearlySummary> = Vec::new();

        let ending_year = data[0].date.year();
        let starting_year = data[data.len() - 1].date.year();

        assert!(
            ending_year >= starting_year,
            "The data must be sorted in reverse chronological order"
        );

        for current_year in starting_year..ending_year + 1 {
            let current_year_first_day =
                NaiveDate::from_ymd_opt(current_year, 1, 1).expect("Invalid date");
            let current_year_open: Option<f32> = data
                .iter()
                .find(|d| d.date == current_year_first_day)
                .map(|d| d.values.open);

            let current_year_last_day =
                NaiveDate::from_ymd_opt(current_year, 12, 31).expect("Invalid date");
            let current_year_close = data
                .iter()
                .find(|d| d.date == current_year_last_day)
                .map(|d| d.values.close);

            let mut current_year_high: f32 = f32::NEG_INFINITY;
            let mut current_year_low: f32 = f32::INFINITY;
            let mut current_year_volume: f32 = 0.0;

            for d in data.iter().filter(|d| d.date.year() == current_year) {
                current_year_high = f32::max(current_year_high, d.values.high);
                current_year_low = f32::min(current_year_low, d.values.low);
                current_year_volume += d.values.volume;
            }

            yearly_summaries.push(YearlySummary {
                year: current_year,
                open: current_year_open,
                high: current_year_high,
                low: current_year_low,
                close: current_year_close,
                volume: current_year_volume,
            });
        }

        yearly_summaries.into_iter().rev().collect()
    }

    fn save_to_csv(data: &[YearlySummary], path: &Path) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new().from_path(path)?;

        writer.write_record(["Year", "Open", "High", "Low", "Close", "Volume"])?;

        data.iter().try_for_each(|current_year_summary| {
            let current_year_open = match current_year_summary.open {
                Some(value) => format!("{value:.2}"),
                None => "".to_string(),
            };
            let current_year_close = match current_year_summary.close {
                Some(value) => format!("{value:.2}"),
                None => "".to_string(),
            };
            let current_year_high = format!("{:.2}", current_year_summary.high);
            let current_year_low = format!("{:.2}", current_year_summary.low);
            let current_year_volume = format!("{:.0}", current_year_summary.volume);

            writer.write_record(&[
                current_year_summary.year.to_string(),
                current_year_open,
                current_year_high,
                current_year_low,
                current_year_close,
                current_year_volume,
            ])
        })?;

        writer.flush()?;
        Ok(())
    }

    fn to_html_table(yearly_summary: &[YearlySummary]) -> String {
        let rows: String = yearly_summary
            .iter()
            .map(|current_year_summary| {
                let current_year = current_year_summary.year;
                let current_year_open = match current_year_summary.open {
                    Some(value) => format_number_with_commas(value, 2),
                    None => "".to_string(),
                };
                let current_year_close = match current_year_summary.close {
                    Some(value) => format_number_with_commas(value, 2),
                    None => "".to_string(),
                };
                let current_year_high = format_number_with_commas(current_year_summary.high, 2);
                let current_year_low = format_number_with_commas(current_year_summary.low, 2);
                let current_year_volume = format_number_with_commas(current_year_summary.volume, 0);

                format!(
                    "        <tr>
          <td>{current_year}</td>
          <td>{current_year_open}</td>
          <td>{current_year_high}</td>
          <td>{current_year_low}</td>
          <td>{current_year_close}</td>
          <td>{current_year_volume}</td>
        </tr>"
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            "    <table class='inline-table'>
      <thead>
        <tr>
          <th colspan='6'>Yearly Summary</th>
        </tr>
        <tr>
          <th>Year</th>
          <th>Open</th>
          <th>High</th>
          <th>Low</th>
          <th>Close</th>
          <th>Volume</th>
        </tr>
      </thead>
      <tbody>
{rows}
      </tbody>
    </table>"
        )
    }
}

#[derive(Debug)]
struct CleanDataWithAnalytics {
    date: NaiveDate,
    values: CleanValues,
    moving_averages: MovingAverages,
    price_changes: PriceChanges,
}

impl CleanDataWithAnalytics {
    fn new(clean_data: &[CleanData], moving_average_size: usize) -> Vec<CleanDataWithAnalytics> {
        let moving_averages = MovingAverages::new(clean_data, moving_average_size);
        let price_changes = PriceChanges::new(clean_data, &moving_averages);
        clean_data
            .iter()
            .enumerate()
            .map(|(i, row)| CleanDataWithAnalytics {
                date: row.date,
                values: row.values.clone(),
                moving_averages: moving_averages[i].clone(),
                price_changes: price_changes[i].clone(),
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
            .y_label_formatter(&|price| format!("{price:.0}"))
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
            "200_WMA_Open",
            "200_WMA_High",
            "200_WMA_Low",
            "200_WMA_Close",
            "200_WMA_Dollar_1_Day",
            "200_WMA_Percent_1_Day",
            "Price_Change_Dollar_200_Week",
            "Price_Change_Percent_200_Week",
            "Price_Change_Dollar_Same_Day_Swing",
            "Price_Change_Percent_Same_Day_Swing",
            "Price_Change_Dollar_1_Day",
            "Price_Change_Percent_1_Day",
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
                format!(
                    "{:.2}",
                    row.price_changes.two_hundred_wma_dollar_change_1_day
                ),
                format!(
                    "{:.2}",
                    row.price_changes.two_hundred_wma_percent_change_1_day
                ),
                format!("{:.2}", row.price_changes.dollar_change_200_week),
                format!("{:.2}", row.price_changes.percent_change_200_week),
                format!("{:.2}", row.price_changes.dollar_swing_same_day),
                format!("{:.1}", row.price_changes.percent_swing_same_day),
                format!("{:.2}", row.price_changes.dollar_change_1_day),
                format!("{:.2}", row.price_changes.percent_change_1_day),
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
    let clean_data_rows = clean_data_with_analytics.len();

    println!("Loaded {clean_data_rows} rows of data");
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

    let input_css_path = Path::new(INPUT_CSS_PATH_STR);
    let output_css_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_CSS_FILENAME);
    std::fs::copy(input_css_path, output_css_path)?;

    let input_favicon_path = Path::new(INPUT_FAVICON_PATH_STR);
    let output_favicon_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_FAVICON_FILENAME);
    std::fs::copy(input_favicon_path, output_favicon_path)?;

    let output_price_analytics_csv_path =
        Path::new(OUTPUT_DIRECTORY).join(OUTPUT_PRICE_ANALYTICS_CSV_FILENAME);
    CleanDataWithAnalytics::save_to_csv(
        &clean_data_with_analytics,
        &output_price_analytics_csv_path,
    )?;

    let output_linear_image_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_LINEAR_IMAGE_FILENAME);
    CleanDataWithAnalytics::create_linear_chart(
        &clean_data_with_analytics,
        &output_linear_image_path,
    )?;

    let output_log_image_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_LOG_IMAGE_FILENAME);
    CleanDataWithAnalytics::create_log_chart(&clean_data_with_analytics, &output_log_image_path)?;

    let yearly_summary = YearlySummary::new(&clean_data_with_analytics);
    let yearly_summary_html_table = YearlySummary::to_html_table(&yearly_summary);
    let output_yearly_summary_csv_path =
        Path::new(OUTPUT_DIRECTORY).join(OUTPUT_YEARLY_SUMMARY_CSV_FILENAME);
    YearlySummary::save_to_csv(&yearly_summary, &output_yearly_summary_csv_path)?;

    let histogram = PriceChangesHistogram::new(&clean_data_with_analytics);
    let histogram_html_table = histogram.to_html_table();
    let output_histogram_csv_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_HISTOGRAM_CSV_FILENAME);
    PriceChangesHistogram::save_to_csv(histogram.clone(), &output_histogram_csv_path)?;

    let output_histogram_image_path =
        Path::new(OUTPUT_DIRECTORY).join(OUTPUT_HISTOGRAM_IMAGE_FILENAME);
    histogram.create_chart(&output_histogram_image_path)?;

    let table_rows: String = clean_data_with_analytics
        .iter()
        .map(|d| {
            format!(
                "          <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td class='wma-column'>{}</td>
            <td>{} ({} %)</td>
            <td>{} ({} %)</td>
            <td>{} ({} %)</td>
            <td>{} ({} %)</td>
          </tr>",
                d.date,
                format_number_with_commas(d.values.open, 2),
                format_number_with_commas(d.values.high, 2),
                format_number_with_commas(d.values.low, 2),
                format_number_with_commas(d.values.close, 2),
                format_number_with_commas(d.moving_averages.close, 2),
                format_number_with_commas(d.price_changes.two_hundred_wma_dollar_change_1_day, 2),
                format_number_with_commas(d.price_changes.two_hundred_wma_percent_change_1_day, 2),
                format_number_with_commas(d.price_changes.dollar_change_200_week, 2),
                format_number_with_commas(d.price_changes.percent_change_200_week, 1),
                format_number_with_commas(d.price_changes.dollar_swing_same_day, 2),
                format_number_with_commas(d.price_changes.percent_swing_same_day, 1),
                format_number_with_commas(d.price_changes.dollar_change_1_day, 2),
                format_number_with_commas(d.price_changes.percent_change_1_day, 1)
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Generate HTML output
    let output_price_analytics_csv_url: String =
        format!("{REPOSITORY_URL}/raw/gh-pages/{OUTPUT_PRICE_ANALYTICS_CSV_FILENAME}");
    let output_html_path = Path::new(OUTPUT_DIRECTORY).join(OUTPUT_HTML_FILENAME);
    let html_content = format!(
        "<!DOCTYPE html>
<html>
  <head>
    <title>{CHART_TITLE}</title>
    <link rel='icon' type='image/png' href='{OUTPUT_FAVICON_FILENAME}'>
    <link rel='stylesheet' href='style.css'>
  </head>
  <body>
    <h1>{CHART_TITLE}</h1>
    <a href='{REPOSITORY_URL}'>Link to the btracker repo</a>
    <br><br>
    <img src='{OUTPUT_LINEAR_IMAGE_FILENAME}' alt='Linear Chart'>
    <br><br>
    <img src='{OUTPUT_LOG_IMAGE_FILENAME}' alt='Log Chart'>
    <br><br>
    <img src='{OUTPUT_HISTOGRAM_IMAGE_FILENAME}' alt='Price Changes Histogram'>
    <br><br>
{yearly_summary_html_table}
{histogram_html_table}
    <br><br>
    <a href='{output_price_analytics_csv_url}'>Link to Price Analytics data</a>
    <br><br>
    <div class='scrollable-table'>
      <table>
        <thead>
          <tr>
            <th>Date</th>
            <th>Open</th>
            <th>High</th>
            <th>Low</th>
            <th>Close</th>
            <th class='wma-column'>200-Week<br>Moving<br>Average</th>
            <th>200-WMA<br>Change</th>
            <th>200-Week<br>Change</th>
            <th>Same-Day<br>Swing</th>
            <th>1-Day<br>Change</th>
          </tr>
        </thead>
        <tbody>
{table_rows}
        </tbody>
      </table>
    </div>
  </body>
</html>"
    );
    write(output_html_path, html_content)?;

    Ok(())
}
