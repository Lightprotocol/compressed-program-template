import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { {{rust-name-camel-case}} } from "../target/types/{{rust-name-snake-case}}";

describe("compressed-program-template", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.CompressedProgramTemplate as Program<CompressedProgramTemplate>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
