import yfinance as yf
import pandas as pd

# Function to fetch the latest data (Open, High, Low, Close, Volume)
def get_latest_data(ticker):
    print('Fetching history')
    api_response_ticker = yf.Ticker(ticker)
    api_response_history = api_response_ticker.history(period='5d', interval='1d')
                            # .history(period='1mo', interval='1d')
    latest_row_of_history = api_response_history.tail(1)
    latest_row_date = latest_row_of_history.index[0]
    print('Latest row date:', latest_row_date.strftime('%b %d %Y'))
    #print('Latest row month:', latest_row_date.strftime('%b'))
    #print('Latest row day:', latest_row_date.day)
    #print('Latest row year:', latest_row_date.year)
    return latest_row_date, latest_row_of_history['Open'], latest_row_of_history['High'], latest_row_of_history['Low'], latest_row_of_history['Close'], latest_row_of_history['Volume']

#script_dir = os.path.dirname(os.path.abspath(__file__))
#project_root = os.path.abspath(os.path.join(script_dir, '..', '..'))
#csv_file = os.path.join(project_root, 'resources/data/historical_data.csv')
csv_file = 'resources/data/historical_data.csv'
df = pd.read_csv(csv_file, delimiter='\t')

last_record = df.iloc[0]
last_month = last_record['Month']
last_day = last_record['Day']
last_year = last_record['Year']
print('Last date:', last_month, last_day, last_year)
last_volume = str(last_record['Volume']).replace(',', '')
print('Last volume:', last_volume)

# Get latest data from Yahoo Finance (you can change this ticker symbol)
ticker = 'BTC-USD'  # Change this to your desired ticker symbol
latest_date, latest_open, latest_high, latest_low, latest_close, latest_volume = get_latest_data(ticker)


# Convert latest volume to string for comparison
latest_volume_str = str(latest_volume.values[0])

print('Latest volume is: ' + latest_volume_str)


exit(0)


# Compare volume (as strings)
if last_volume != latest_volume_str:
    # Prepare new row with the latest data
    new_row = {
        'Month': pd.to_datetime('today').strftime('%b'),
        'Day': pd.to_datetime('today').day,
        'Year': pd.to_datetime('today').year,
        'Open': latest_open,
        'High': latest_high,
        'Low': latest_low,
        'Close': latest_close,
        'AdjClose': latest_close,
        'Volume': latest_volume_str
    }
    
    # Append new data to the DataFrame
    df = pd.concat([new_row, df], ignore_index=True)
    #df = df.append(new_row, ignore_index=True)
    
    # Save updated CSV with pipe delimiter
    df.to_csv(csv_file, sep='\t', index=False)
