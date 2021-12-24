import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { BettingGame } from '../target/types/betting_game';

describe('betting_game', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.BettingGame as Program<BettingGame>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
