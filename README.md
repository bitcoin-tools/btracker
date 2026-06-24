# ₿tracker

Plot the bitcoin price and 200-week moving average.

Please see [bitcoin-tools.github.io/btracker](https://bitcoin-tools.github.io/btracker/).

## About the Math

btracker is a calculation and data visualization tool that:
- inputs historica price data
- calculates the 200-week moving average
- builds plots on linear and log scales
- saves the calculated data to a CSV file
- builds a static HTML page with the plots
- deploys the HTML page [on GitHub Pages](https://bitcoin-tools.github.io/btracker)

### Input Data

The input data come from multiple sources.

- For `2014-09-17` to `present`, the data are from [Yahoo Finance](https://finance.yahoo.com/quote/BTC-USD/history/).

- For `2011-12-10` to `2014-09-16`, the data are from CoinMarketCap.

Please open an issue to add a consistent source from pre-2011 through present.

### Calculated Data - 200-Week Moving Average

The 200-week moving average is the simple arithemtic mean of the Bitcoin price over the last 200 weeks.

`btracker` uses the last 1,400 days, which improves granularity and accuracy over weekly data.

## Contributing

This project is Open Source and welcomes contributions.

### Report an Issue

Please open an [issue](https://github.com/bitcoin-tools/btracker/issues) for any bug reports or feature requests.

### Contribute Code

Pull requests are welcome. For major changes, please open an issue first to discuss the implentation.

## License

This project is licensed under the terms of [the MIT No Attribution / MIT-0 license](./LICENSE).
