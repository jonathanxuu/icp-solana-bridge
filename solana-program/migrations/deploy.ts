import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, Program, Wallet, setProvider } from "@project-serum/anchor";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { readFileSync } from "fs";
import { resolve } from "path";
import * as path from 'path';
import * as dotenv from 'dotenv';
import { getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";

const envFilePath = path.resolve(__dirname, '../.env');
dotenv.config({ path: envFilePath });

// 设置 Anchor 提供的 Provider
// const provider = anchor.AnchorProvider.env();
// anchor.setProvider(provider);

// 设置自定义 RPC 端点
const connection = new Connection("https://rpc.ankr.com/solana_devnet", "confirmed");

const wallet = new Wallet(Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(process.env.ANCHOR_WALLET as string))
));

const provider = new AnchorProvider(connection, wallet, { 
  commitment: 'processed',
  skipPreflight: true
});
setProvider(provider);


// Program ID
const programId = new PublicKey("3CLbFjWR9UGNFFcUCkyNqWmTpk2vkYCTk17RxMEyyhZd");

// 从 IDL 文件读取定义
const idl = JSON.parse(
  readFileSync(resolve(__dirname, "../target/idl/token_pool.json"), "utf-8")
);
console.log(idl)
// 初始化程序
const program = new Program(idl, programId, provider);

// 替换为您的 Token Mint 地址
const tokenMint = new PublicKey("5Tzkr68aNmUB3VATYtkC2GVEXxV8KVgKSRfyVcoyQkCD");

async function initializeTokenPool() {
  try {
    const userTokenAccount = await createUserTokenAccount(connection, wallet.payer, tokenMint);
    console.log(`User Token Account: ${userTokenAccount}`);
} catch (error) {
    console.error('Error creating token account:', error);
}


}

async function createUserTokenAccount(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey
): Promise<PublicKey> {
  // 获取用户的 Token 账户（如果不存在则创建）
  const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      mint,
      new PublicKey("7wmBBNF6A5qEZ3HiZkJbzFb4rzioA6SNfa6zb8taJ9g")
  );

  console.log(`Token Account created: ${userTokenAccount.address.toString()}`);
  return userTokenAccount.address;
}

initializeTokenPool().catch(console.error);
