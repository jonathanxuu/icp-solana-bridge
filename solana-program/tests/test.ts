import * as anchor from "@project-serum/anchor";
import { Program, AnchorProvider, Idl, web3 } from "@project-serum/anchor";
import { Vault } from "../target/types/vault"; // 确保这个类型已生成
import vaultIdlJson from "../target/idl/vault.json"; // IDL JSON 文件路径
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import assert from "assert";

describe("Vault Program", () => {
  // 初始化 provider 和 program
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);

  const vaultIdl = vaultIdlJson as unknown as Idl;
  const programId = new PublicKey(vaultIdl.metadata.address); // 使用 IDL 中的地址
  const program = new Program<Vault>(vaultIdl, programId, provider);  // 使用类型Program<Vault>

  // 定义全局变量
  let vaultAccount: web3.Keypair;
  let vaultAuthority: PublicKey;
  let mint: PublicKey;
  let userTokenAccount: PublicKey;
  let vaultTokenAccount: PublicKey;
  const depositAmount = 1000; // 定义一个测试存款金额

  before(async () => {
    // 创建 mint 代币
    mint = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      9 // 小数位数
    );

    // 创建用户的 Token 账户
    userTokenAccount = await createAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      provider.wallet.publicKey
    );

    // 给用户的账户铸造一些代币
    await mintTo(
      provider.connection,
      provider.wallet.payer,
      mint,
      userTokenAccount,
      provider.wallet.publicKey,
      depositAmount * 10 // 赋予用户账户更多余额
    );

    // 创建 Vault 账户
    vaultAccount = web3.Keypair.generate();
    vaultAuthority = await PublicKey.createProgramAddress(
      [vaultAccount.publicKey.toBuffer()],
      program.programId
    );

    // 创建 Vault 的 Token 账户
    vaultTokenAccount = await createAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      vaultAuthority
    );
  });

  it("Initializes the Vault", async () => {
    await program.methods
      .initializeVault()
      .accounts({
        vault: vaultAccount.publicKey,
        authority: provider.wallet.publicKey,
        vaultTokenAccount,
        mint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([vaultAccount])
      .rpc();

    // 检查 Vault 账户是否成功初始化
    const vaultAccountInfo = await provider.connection.getAccountInfo(vaultAccount.publicKey);
    assert.ok(vaultAccountInfo !== null, "Vault account should be initialized");
  });

  it("Deposits tokens into the Vault", async () => {
    await program.methods
      .deposit(new anchor.BN(depositAmount))
      .accounts({
        vault: vaultAccount.publicKey,
        vaultTokenAccount,
        userTokenAccount,
        userAuthority: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // 验证 Vault Token 账户的余额是否更新
    const vaultTokenAccountInfo = await getAccount(provider.connection, vaultTokenAccount);
    assert.strictEqual(
      vaultTokenAccountInfo.amount.toNumber(),
      depositAmount,
      "Vault should hold the deposited tokens"
    );
  });

  it("Withdraws tokens from the Vault", async () => {
    await program.methods
      .withdraw(new anchor.BN(depositAmount))
      .accounts({
        vault: vaultAccount.publicKey,
        vaultTokenAccount,
        userTokenAccount,
        userAuthority: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // 验证用户 Token 账户是否收到取款金额
    const userTokenAccountInfo = await getAccount(provider.connection, userTokenAccount);
    assert.strictEqual(
      userTokenAccountInfo.amount.toNumber(),
      depositAmount * 10,
      "User should receive the withdrawn tokens"
    );

    // 验证 Vault Token 账户的余额是否归零
    const vaultTokenAccountInfo = await getAccount(provider.connection, vaultTokenAccount);
    assert.strictEqual(vaultTokenAccountInfo.amount.toNumber(), 0, "Vault should be empty after withdrawal");
  });
});
