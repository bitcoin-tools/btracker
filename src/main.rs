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
                    position: sticky;
                    top: 0;
                    background-color: whitesmoke;
                }}
                td {{
                    border: 1px solid black;
                    padding: 5px;
                    text-align: right;
                }}
                .scrollable-table {{
                    height: 100px;
                    overflow: auto;
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
            {yearly_summary_html_table}
            <br><br>
            {histogram_html_table}
            <br><br>
            <a href='{output_price_analytics_csv_url}'>Link to Price Analytics data</a>
            <br><br>
            <div class='scrollable-table'>
                <table>
                    <thead>
                        <tr>
                            <th rowspan='2'>Date</th>
                            <th colspan='4'>Daily Prices</th>
                            <th rowspan='2' class='wma-column'>200-Week<br>Moving<br>Average</th>
                            <th colspan='2'>200-WMA Change</th>
                            <th colspan='2'>200-Week Change</th>
                            <th colspan='2'>Same-Day Swing</th>
                            <th colspan='2'>1-Day Change</th>
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
                            <th>$ Change</th>
                            <th>% Change</th>
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
