mod pc;
use pc::Price;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount, Transfer, CloseAccount};
use solana_program::clock::Clock;
use spl_token::instruction::AuthorityType;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// Comparators
const COMPARATOR_LESS_THAN_AT_EXPIRY : u8 = 0;
const COMPARATOR_GREATER_THAN_AT_EXPIRY : u8 = 1;

// Player bets
const PLAYER_BET_FOR : u8 = 1;
const PLAYER_BET_AGAINST : u8 = 2;

// State
const STATE_RUNNING : u8 = 1;
const STATE_FOR_WINS : u8 = 2;
const STATE_AGAINST_WINS : u8 = 3;
const _STATE_FAILED_GAME : u8 = 4;

//state 
// 0
#[program]
pub mod betting_game {
    use super::*;
    const GAME_SEED: &[u8] = b"BET_ON";

    pub fn initialize_dashboard(ctx: Context<Initializedashboard>) -> ProgramResult {
        let dashboard = &mut ctx.accounts.dashboard;
        dashboard.count = 0;
        dashboard.address = *dashboard.to_account_info().key;
        Ok(())
    }

    pub fn initialize(  ctx: Context<Initialize>, 
                        security: String, 
                        comparator: u8, 
                        value: u64, 
                        exp: u64, 
                        expiry: u64, 
                        freeze: u64) -> ProgramResult {

        msg!("Started");
        if freeze == 0 || expiry == 0 {
            msg!("Freezing or expiry offset is 0");
            return Err(ErrorCodes::CannotInitate.into());
        }
        else if freeze > expiry{
            msg!("Freezed after expiry");
            return Err(ErrorCodes::CannotInitate.into());
        }
        else if comparator > 2 {
            msg!("Unknown comparator");
            return Err(ErrorCodes::CannotInitate.into());
        }
        
        msg!("Initialized");
        // Set game and dashboard states
        let dashboard = &mut ctx.accounts.dashboard;
        let game = &mut ctx.accounts.bet_on;
        game.game_id = dashboard.count;
        dashboard.count = dashboard.count + 1;
        let game_count_bytes : [u8; 8] = game.game_id.to_ne_bytes();
        let seed: &[u8] = &([GAME_SEED, &game_count_bytes].concat());
        let (pda, _bump_seed) = Pubkey::find_program_address( &[seed], ctx.program_id);

        let sec = security.as_bytes();
        game.security[..sec.len()].copy_from_slice(sec);

        game.comparator = comparator;
        game.value = value as i64;
        game.exp =exp as i32;
        let clock = solana_program::clock::Clock::get()?;
        game.start = clock.unix_timestamp;
        game.expiry = game.start + expiry as i64;
        game.freeze = game.start + freeze as i64;

        game.creator = *ctx.accounts.creator.key;
        game.total_pot = 0;
        game.player_count = 0;

        game.state = STATE_RUNNING;

        msg!("Setting auth");
        let msg = format!("owner : {} creator : {}", ctx.accounts.vault.owner, ctx.accounts.creator.key);
        msg!( &msg[..] );
        // transfer vault authority
        let cpi_vault = SetAuthority {
            account_or_mint: ctx.accounts.vault.to_account_info().clone(),
            current_authority: ctx.accounts.creator.clone(),
        };
        let cpi = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_vault);
        token::set_authority( cpi,  AuthorityType::AccountOwner, Some(pda))?;

        Ok(())
    }

    pub fn add_player( ctx: Context<AddPlayer>, side : u8, amount: u64 ) -> ProgramResult {
        // Check if a player can bet
        if side == 0 || side > 2 {
            return Err(ErrorCodes::UnknownBet.into());
        }
        let clock = solana_program::clock::Clock::get()?;
        let time = clock.unix_timestamp;
        let bet_on = &mut ctx.accounts.bet_on;

        if time > bet_on.freeze || time > bet_on.expiry {
            return Err( ErrorCodes::BettingFrozen.into() );
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.player.to_account_info().clone(),
            to: ctx.accounts.vault.to_account_info().clone(),
            authority: ctx.accounts.player.to_account_info().clone(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info().clone(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        
        ctx.accounts.player_data.amount = amount;
        ctx.accounts.player_data.key = *ctx.accounts.player.to_account_info().key;
        ctx.accounts.player_data.game_id = bet_on.game_id;
        ctx.accounts.player_data.bet = side;
        ctx.accounts.player_data.state = 0;
        bet_on.player_count += 1;
        if side == PLAYER_BET_FOR{
            bet_on.amount_for += amount;
        }
        else {
            bet_on.amount_against += amount;
        }

        bet_on.total_pot += amount;

        let player_account = SetAuthority {
            account_or_mint: ctx.accounts.player_data.to_account_info().clone(),
            current_authority: ctx.accounts.system_program.to_account_info().clone(),
        };
        let cpi =  CpiContext::new(ctx.accounts.token_program.to_account_info(), player_account);

        let game_count_bytes : [u8; 8] = ctx.accounts.bet_on.game_id.to_ne_bytes();
        let seed: &[u8] = &([GAME_SEED, &game_count_bytes].concat());
        let (pda, _bump_seed) = Pubkey::find_program_address( &[seed], ctx.program_id);

        token::set_authority(cpi, AuthorityType::AccountOwner, Some(pda))?;

        Ok(())
    }

    #[access_control(can_execute(&ctx.accounts.bet_on))]
    pub fn execute(ctx: Context<Execute>, _security: String) -> ProgramResult {
        //TO DO CHECK SECURITY MATCHES
        let price_oracle = Price::load(&ctx.accounts.oracle).unwrap();
        let price = (price_oracle.agg.price as f64).powf(price_oracle.expo as f64);
        let bet_price = (ctx.accounts.bet_on.value as f64).powf(ctx.accounts.bet_on.exp as f64);
        let _conf = price_oracle.agg.conf; //TO DO USE CONFIDENCE

        if price > bet_price
        {
            match ctx.accounts.bet_on.comparator {
                COMPARATOR_LESS_THAN_AT_EXPIRY => ctx.accounts.bet_on.state = STATE_AGAINST_WINS,
                COMPARATOR_GREATER_THAN_AT_EXPIRY => ctx.accounts.bet_on.state = STATE_FOR_WINS,
                _ => return Err(ErrorCodes::UnknownBet.into()),
            }
        }
        else {
            match ctx.accounts.bet_on.comparator {
                COMPARATOR_LESS_THAN_AT_EXPIRY => ctx.accounts.bet_on.state = STATE_FOR_WINS,
                COMPARATOR_GREATER_THAN_AT_EXPIRY => ctx.accounts.bet_on.state = STATE_AGAINST_WINS,
                _ => return Err(ErrorCodes::UnknownBet.into()),
            }
        }

        Ok(())
    }

    pub fn withdraw(ctx: Context<WithdrawWinner>) -> ProgramResult {
        let player_data = &mut ctx.accounts.player_data;
        let game = &mut ctx.accounts.bet_on;

        let game_count_bytes : [u8; 8] = game.game_id.to_ne_bytes();
        let seed: &[u8] = &([GAME_SEED, &game_count_bytes].concat());
        let (_pda, bump_seed) = Pubkey::find_program_address( &[seed], ctx.program_id);
        let seeds = &[&seed[..], &[bump_seed]];
        let seed_to_use = &[&seeds[..]];

        let amount = player_data.amount
                            .checked_mul(game.total_pot).unwrap()
                            .checked_div( if game.state == STATE_FOR_WINS {game.amount_for} else {game.amount_against}).unwrap();
    
        player_data.state = 1;
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info().clone(),
            to: ctx.accounts.player.to_account_info().clone(),
            authority: ctx.accounts.pda_account.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seed_to_use);
        token::transfer(cpi_ctx, amount)?;

        game.player_count -= 1;
        if game.player_count == 0 {
            // close game if everyone has withdrawn and send the rent to the creator
            let cpi_accounts = CloseAccount {
                account: ctx.accounts.bet_on.to_account_info().clone(),
                destination: ctx.accounts.creator.clone(),
                authority: ctx.accounts.pda_account.clone(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seed_to_use);
            token::close_account(cpi_ctx)?;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initializedashboard<'info> {
    #[account(zero)]
    dashboard: Account<'info, Dashboard>,
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(signer)]
    creator: AccountInfo<'info>,
    #[account(mut)]
    dashboard: Account<'info, Dashboard>,
    #[account(init, payer=creator, space = BetOn::LEN + 8)]
    bet_on: Account<'info, BetOn>,
    #[account(mut,constraint= vault.owner == *creator.key)]
    vault : Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AddPlayer<'info> {
    #[account(mut,signer)]
    player: Account<'info, TokenAccount>,
    #[account(mut, constraint = bet_on.state == STATE_RUNNING)]
    bet_on: Account<'info, BetOn>,

    #[account(constraint = bet_on.vault == *vault.to_account_info().key, 
        constraint = player.mint == vault.mint)]
    vault: Account<'info, TokenAccount>,

    #[account(init, payer=player, space = 8 + PlayerData::LEN)]
    player_data : Account<'info, PlayerData>,

    #[account(signer)]
    pda: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Execute<'info> {
    #[account(mut, constraint = bet_on.state == STATE_RUNNING)]
    bet_on: Account<'info, BetOn>,

    #[account(signer)]
    pda: AccountInfo<'info>,

    oracle : AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WithdrawWinner<'info> {
    player: Account<'info, TokenAccount>,

    #[account(mut, 
        constraint = bet_on.state < 4 && bet_on.state > 1,
        constraint = player_data.game_id == bet_on.game_id)]
    bet_on: Account<'info, BetOn>,

    #[account(mut,close=player, 
        constraint = player_data.key == *player.to_account_info().key,
        constraint = player_data.state == 0,
        constraint = (player_data.bet == PLAYER_BET_FOR && bet_on.state == STATE_FOR_WINS) ||
                     (player_data.bet == PLAYER_BET_AGAINST && bet_on.state == STATE_AGAINST_WINS) )]
    player_data: Account<'info, PlayerData>,

    #[account(signer)]
    creator: AccountInfo<'info>,

    #[account(mut,
        constraint = *vault.to_account_info().key == bet_on.vault)]
    vault : Account<'info, TokenAccount>,
    pub pda_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Dashboard {
    count: u64,
    address: Pubkey
}

#[account]
pub struct PlayerData {
    amount: u64,
    key: Pubkey,
    bet: u8,
    game_id: u64,
    state: u8,
}

impl PlayerData {
    const LEN : usize = 64 + 32 + 8 + 64 + 8;
}

#[account]
pub struct BetOn {
    game_id: u64,
    security: [u8; 10],
    comparator: u8,
    value: i64,
    exp: i32,
    start: i64,
    expiry: i64,
    freeze: i64,
    creator: Pubkey,
    vault: Pubkey,
    total_pot: u64,
    amount_for: u64,
    amount_against: u64,
    player_count: u32,
    state: u8,
}

impl BetOn{
    const LEN: usize = 64 + 10 * 8 + 8 + 64 + 32 + 64 + 64 + 64 + 64 + 64 + 32 + 32 + 64 + 32+ 8; 
}

#[error]
pub enum ErrorCodes {
    #[msg("You are Unauthorized")]
    Unathorized,
    #[msg("Cannot Create a bet conditions already met")]
    ConditionsAlreadyMet,
    #[msg("Betting frozen")]
    BettingFrozen,
    #[msg("Cannot initiate")]
    CannotInitate,
    #[msg("Unknown Bet")]
    UnknownBet,
    #[msg("Cannot execute yet bet is not expired")]
    BetNotExpired,
    #[msg("Withdraw error")]
    WithdrawError,
    #[msg("Error Getting Prices")]
    OracleError,
}

fn can_execute(bet_on: &BetOn) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp < bet_on.expiry {
        return Err(ErrorCodes::BetNotExpired.into());
    }
    Ok(())
}