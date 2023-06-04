import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SingleAssetLendingProgram } from "../target/types/single_asset_lending_program";

describe("single-asset-lending-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SingleAssetLendingProgram as Program<SingleAssetLendingProgram>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
