import yfinance as yf
import pandas as pd

# Function to fetch the latest data (Open, High, Low, Close, Volume)
def get_latest_data(ticker):
    data = yf.download(ticker, period="1d", interval="1m")
    latest_data = data.iloc[-1]
    return latest_data['Open'], latest_data['High'], latest_data['Low'], latest_data['Close'], latest_data['Volume']

# Load existing pipe-delimited CSV
csv_file = 'prices.csv'
df = pd.read_csv(csv_file, delimiter='|')

# Assuming the CSV has columns: Month, Day, Year, Open, High, Low, Close, AdjClose, Volume
last_record = df.iloc[-1]
last_volume = str(last_record['Volume'])  # Convert to string for comparison

# Get latest data from Yahoo Finance (you can change this ticker symbol)
ticker = 'AAPL'  # Change this to your desired ticker symbol
latest_open, latest_high, latest_low, latest_close, latest_volume = get_latest_data(ticker)

# Convert latest volume to string for comparison
latest_volume_str = str(latest_volume)

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
        'AdjClose': latest_close,  # Adjust if necessary
        'Volume': latest_volume_str
    }
    
    # Append new data to the DataFrame
    df = df.append(new_row, ignore_index=True)
    
    # Save updated CSV with pipe delimiter
    df.to_csv(csv_file, sep='|', index=False)
