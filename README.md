# Betting game

## Project
This project is a smart contract for solana to bet on prices of crypto currencies at a given time in future.

A creator can create a betting session for a given security (crypto pair e.g BTC/USD) for a price at expiry time and a comparator (either below or abouve a threashold price).
Players can bet only a single type of tokens for the moment __For__ or __Against__ bets. Players cannot bet after a freeze time.
At the expiry the price will checked and winner group is decided. Then all the tokens deposited during session are distributed amoungst the winners.

Oracle for prices is Pyth.
