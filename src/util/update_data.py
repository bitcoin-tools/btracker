'''This script reads the input file, pulls new data via API, and writes back to the file.'''

from datetime import datetime
import pandas as pd
import yfinance as yf

def get_latest_data(ticker_to_check='BTC-USD', days_to_fetch=1400):
    '''This function pulls historical data from yf'''
    print('Fetching history')
    api_response_ticker = yf.Ticker(ticker_to_check)
    api_response_ticker_history = api_response_ticker.history(
        period=f'{days_to_fetch}d',
        interval='1d'
    )
    return api_response_ticker_history

INPUT_DATA_FILE = 'resources/data/historical_data.tsv'
df = pd.read_csv(INPUT_DATA_FILE, sep='\t')

last_record = df.iloc[0]
last_month = last_record['Month']
print('Last month:', last_month)
last_day = last_record['Day']
print('Last day:', last_day)
last_year = last_record['Year']
print('Last year:', last_year)
last_date = pd.to_datetime(f"{last_year}-{last_month}-{last_day}").tz_localize(None)
print('Last date:', last_date.strftime('%b %d %Y'))
last_open = last_record['Open']
print('Last open:', last_open)
last_high = last_record['High']
print('Last high:', last_high)
last_low = last_record['Low']
print('Last low:', last_low)
last_close = last_record['Close']
print('Last close:', last_close)
# last_volume = str(last_record['Volume']).replace(',', '')
# print('Last volume:', last_volume)

TICKER = 'BTC-USD'
# Calculate the number of days between today and the last date in the file
today = datetime.now().date()
dynamic_days_to_fetch = (today - last_date.date()).days + 2
# "+ 2" days to account for a bug I noticed during the spring DST change in the YF API
print(f'Today: {today}')
print(f'Days to fetch: {dynamic_days_to_fetch}')

ticker_history = get_latest_data(TICKER, days_to_fetch=dynamic_days_to_fetch)

latest_row_of_history = ticker_history.tail(1)
latest_date = latest_row_of_history.index[0]
latest_open = latest_row_of_history['Open']
latest_high = latest_row_of_history['High']
latest_low = latest_row_of_history['Low']
latest_close = latest_row_of_history['Close']
latest_volume = latest_row_of_history['Volume']
print('Latest row date:', latest_date.strftime('%b %d %Y'))
print('Latest row month:', latest_date.strftime('%b'))
print('Latest row day:', latest_date.day)
print('Latest row year:', latest_date.year)
print('Latest row open:', latest_open.values[0])
print('Latest row high:', latest_high.values[0])
print('Latest row low:', latest_low.values[0])
print('Latest row close:', latest_close.values[0])
print('Latest row volume:', latest_volume.values[0])

for date, row in ticker_history.iterrows():
    date = date.tz_localize(None)

    if date < last_date:
        print(f"The date {date.strftime('%b %d %Y')} is already in the file.")
        continue
    if date == last_date:
        print(f"The date {date.strftime('%b %d %Y')} is the last date.")
        print("Removing before adding the new data.")
        df = df.iloc[1:]

    print('---')
    print(f"Date: {date.strftime('%b %d %Y')}")
    print(f"Open: {row['Open']}")
    print(f"High: {row['High']}")
    print(f"Low: {row['Low']}")
    print(f"Close: {row['Close']}")
    print(f"Volume: {int(row['Volume'])}")

    new_row = {
        'Month': date.strftime('%b'),
        'Day': date.day,
        'Year': date.year,
        'Open': row['Open'],
        'High': row['High'],
        'Low': row['Low'],
        'Close': row['Close'],
        'AdjClose': row['Close'],
        'Volume': int(row['Volume'])
    }

    df = pd.concat([pd.DataFrame([new_row]), df], ignore_index=True)

print('---')

OUTPUT_DATA_FILE = INPUT_DATA_FILE
df.to_csv(OUTPUT_DATA_FILE, sep='\t', index=False)

print(f"The updated data has been saved to {OUTPUT_DATA_FILE}")
