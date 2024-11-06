use anchor_lang::prelude::*;
use solana_program::ed25519_program::ID as ED25519_ID;
use solana_program::instruction::Instruction;
use solana_program::sysvar::instructions::load_instruction_at_checked;
use solana_ed25519_verify::verify_signature;
use std::convert::TryInto;

pub fn verify_ed25519(
    pubkey: [u8; 32],
    msg: Vec<u8>,
    sig: [u8; 64],
) -> Result<()> {
    // Get what should be the Ed25519Program instruction
    let publickey = Pubkey::new(&pubkey);
    let verification_result = verify_signature(&publickey, &sig, &msg);
    // Do other stuff
    if verification_result.is_err(){
        return Err(ErrorCode::SigVerificationFailed.into());
    }
    Ok(())
}


pub fn hex_to_array(hex_string: &str) -> [u8; 32] {
    // 创建一个 32 字节的数组
    let mut array = [0u8; 32];

    // 将十六进制字符串转换为字节
    for (i, byte) in hex_string.as_bytes().chunks(2).enumerate() {
        // 解析每两个字符为一个字节
        let byte_str = std::str::from_utf8(byte).unwrap();
        array[i] = u8::from_str_radix(byte_str, 16).unwrap();
    }

    return array;
}

pub fn hex_to_array_64(hex_string: &str) -> [u8; 64] {
    // 创建一个 32 字节的数组
    let mut array = [0u8; 64];

    // 将十六进制字符串转换为字节
    for (i, byte) in hex_string.as_bytes().chunks(2).enumerate() {
        // 解析每两个字符为一个字节
        let byte_str = std::str::from_utf8(byte).unwrap();
        array[i] = u8::from_str_radix(byte_str, 16).unwrap();
    }

    return array;
}


#[error_code]
pub enum ErrorCode {
    #[msg("Signature verification failed.")]
    SigVerificationFailed,
}
