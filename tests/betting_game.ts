import * as anchor from "@project-serum/anchor";
import { Program, BN, IdlAccounts } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, Token, u64 } from "@solana/spl-token";
import { assert } from "chai";
import { BettingGame } from '../target/types/betting_game';
const serumCmn = require("@project-serum/common");
import { rpc } from "@project-serum/anchor/dist/cjs/utils";
import { utf8 } from "@project-serum/anchor/dist/cjs/utils/bytes";

describe('betting_game', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BettingGame as Program<BettingGame>;
  let dashboard = anchor.web3.Keypair.generate();
  let game = anchor.web3.Keypair.generate();
  let security = "BTC/USD";
  let comparator = new anchor.BN(0);
  let value = new anchor.BN(60000);
  let exp = new anchor.BN(0);
  
  const nowBn = new anchor.BN(Date.now() / 1000);
  let expiry = nowBn.add(new anchor.BN(10));
  let freeze = nowBn.add(new anchor.BN(5));
  
  it("Initialize Dashboard", async () => {
    const tx = await program.rpc.initializeDashboard({
      accounts: {
        authority: program.provider.wallet.publicKey,
        dashboard: dashboard.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [dashboard],
      instructions: [
        await program.account.dashboard.createInstruction(dashboard),
      ],
    });
  });

  let mint_account: Token= null;
  let vault : PublicKey = null;
  const bet_on = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();

  it("Initialize Game", async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10000000000),
      "confirmed"
    );

    mint_account  = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );
    vault = await mint_account.createAccount(provider.wallet.publicKey);

    const comparator : number = 0;
    const tx = await program.rpc.initialize(
      security,
      comparator,
      value, 
      exp,
      expiry,
      freeze,
      {
        accounts : {
          creator : provider.wallet.publicKey,
          dashboard : dashboard.publicKey,
          vault : vault,
          betOn : bet_on.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [bet_on],
    });
  });

  
  let player1 : Token = null;
  let player2 : Token = null;
  let player3 : Token = null;
  let player4 : Token = null;
  const player_data1 = anchor.web3.Keypair.generate();
  const player_data2 = anchor.web3.Keypair.generate();
  const player_data3 = anchor.web3.Keypair.generate();
  const player_data4 = anchor.web3.Keypair.generate();

  it("Add players", async() => {
    
  const [pda, _nonce] = await PublicKey.findProgramAddress(
    [Buffer.from(anchor.utils.bytes.utf8.encode("BET_ON"))],
    program.programId
  );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(player1.publicKey, 10 * 10e6),
      "confirmed"
    );

    const tx = await program.rpc.addPlayer(
      1,
      new anchor.BN(9),
      {
        accounts : {
          player: player1.publicKey,
          betOn: bet_on.publicKey,
          vault: vault,
          playerData: player_data1.publicKey,
          pda,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        }
      }
    )
  });
});
