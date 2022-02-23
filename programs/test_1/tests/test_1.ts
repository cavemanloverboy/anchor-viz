import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Test1 } from "../target/types/test_1";
const assert = require("assert");
const { SystemProgram } = anchor.web3;

describe("test_1", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Test1 as Program<Test1>;

  const storage = anchor.web3.Keypair.generate();
  it("storageAccount initialized with 32!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({
      accounts : {
        storage: storage.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId
      },
      signers: [storage]
    });
    console.log("Your transaction signature", tx);

    let storageAccount = await program.account.storage.fetch(storage.publicKey);
    assert.ok(storageAccount.data.toNumber() === 32);
  });
});
