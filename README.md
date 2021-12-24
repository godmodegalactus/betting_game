# Betting game

## Project
This project is a smart contract with solana to bet on prices of crypto.

Creator can create a betting session for a given security (crypto pair e.g BTC/USD) what will be the price at expiry time (either below or abouve a threashold price).
Players can bet only a single type of tokens for the moment For or Against bets. Players cannot bet after a freeze time.
At the expiry the price will checked and winner is decided. Then all the tokens deposited during session is distributed amoungst the winners.

Oracle for prices is Pyth.
