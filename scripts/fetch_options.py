#!/usr/bin/env python3
"""Fetch European-style index option chain data from Yahoo Finance.

Downloads option chains and writes CSVs in the canonical schema defined
in data/README.md (Phase 9). Helper tool for data acquisition, not
production code.

Dependencies: pip install yfinance
"""

import os
import time
from datetime import date

import pandas as pd
import yfinance as yf

# --- Configuration ---

TICKERS = {
    "^SPX": ("cboe", "spx"),   # S&P 500 Index — European
    "^NDX": ("cboe", "ndx"),   # Nasdaq 100 Index — European
}

DATA_DIR = "data"

# Canonical CSV column order (from Phase 9 schema)
REQUIRED_COLS = [
    "quote_date", "expiry", "strike", "option_type",
    "bid", "ask", "underlying_price",
]
OPTIONAL_COLS = ["volume", "open_interest", "implied_vol"]


def fetch_and_save(yahoo_ticker, source, underlying, data_dir):
    """Fetch full option chain and write canonical CSV."""
    print(f"\n--- Fetching {yahoo_ticker} ({source}/{underlying}) ---")

    ticker = yf.Ticker(yahoo_ticker)

    # Get underlying price
    try:
        underlying_price = ticker.info.get("regularMarketPrice")
        if underlying_price is None:
            underlying_price = ticker.fast_info["lastPrice"]
    except Exception:
        underlying_price = ticker.fast_info.get("lastPrice")
    if underlying_price is None:
        print(f"  WARNING: Could not get underlying price for {yahoo_ticker}")
        return None

    # Get available expiration dates
    expirations = ticker.options
    if not expirations:
        print(f"  WARNING: No option expirations found for {yahoo_ticker}")
        return None
    print(f"  Found {len(expirations)} expiration dates")

    # Fetch all expiry slices
    all_rows = []
    quote_date = date.today().isoformat()

    for expiry in expirations:
        try:
            chain = ticker.option_chain(expiry)
        except Exception as e:
            print(f"  WARNING: Failed to fetch {expiry}: {e}")
            continue

        for opt_type, df in [("C", chain.calls), ("P", chain.puts)]:
            if df.empty:
                continue
            slice_df = pd.DataFrame({
                "quote_date": quote_date,
                "expiry": expiry,
                "strike": df["strike"],
                "option_type": opt_type,
                "bid": df["bid"],
                "ask": df["ask"],
                "underlying_price": underlying_price,
                "volume": df.get("volume"),
                "open_interest": df.get("openInterest"),
                "implied_vol": df.get("impliedVolatility"),
            })
            all_rows.append(slice_df)

        time.sleep(0.1)  # Brief pause between expiry fetches

    if not all_rows:
        print(f"  WARNING: No data rows collected for {yahoo_ticker}")
        return None

    result = pd.concat(all_rows, ignore_index=True)

    # Filter out rows with no market (bid=0 AND ask=0)
    before = len(result)
    result = result[~((result["bid"] == 0) & (result["ask"] == 0))]
    filtered = before - len(result)

    # Sort by expiry, strike, option_type
    result = result.sort_values(["expiry", "strike", "option_type"])
    result = result.reset_index(drop=True)

    # Write CSV
    out_dir = os.path.join(data_dir, source, underlying)
    os.makedirs(out_dir, exist_ok=True)
    out_path = os.path.join(out_dir, f"{quote_date}.csv")

    col_order = REQUIRED_COLS + [c for c in OPTIONAL_COLS if c in result.columns]
    result[col_order].to_csv(out_path, index=False)

    expiry_count = result["expiry"].nunique()
    print(f"  Wrote: {out_path}")
    print(f"  Rows: {len(result)} ({filtered} zero-market rows filtered)")
    print(f"  Expiries: {expiry_count}")
    print(f"  Underlying: {underlying_price}")
    return out_path


if __name__ == "__main__":
    written = []
    for yahoo_ticker, (source, underlying) in TICKERS.items():
        try:
            path = fetch_and_save(yahoo_ticker, source, underlying, DATA_DIR)
            if path:
                written.append(path)
        except Exception as e:
            print(f"  ERROR fetching {yahoo_ticker}: {e}")
        time.sleep(2)  # Rate limit between tickers

    print(f"\n=== Summary ===")
    print(f"Files written: {len(written)}")
    for p in written:
        print(f"  {p}")
