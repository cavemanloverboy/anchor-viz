import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Test0 } from "../target/types/test_0";

describe("test_0", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Test0 as Program<Test0>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
