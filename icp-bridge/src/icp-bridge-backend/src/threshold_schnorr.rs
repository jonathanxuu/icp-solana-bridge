use candid::{CandidType, Principal};
use ic_cdk::{query, update};
use serde::{Deserialize, Serialize};
use std::{convert::{TryFrom, TryInto}};

type CanisterId = Principal;

#[derive(CandidType, Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SchnorrAlgorithm {
    #[serde(rename = "bip340secp256k1")]
    Bip340Secp256k1,
    #[serde(rename = "ed25519")]
    Ed25519,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct PublicKeyReply {
    pub public_key_hex: String,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct SignatureReply {
    pub signature_hex: String,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct SignatureVerificationReply {
    pub is_signature_valid: bool,
}

#[derive(CandidType, Serialize, Debug)]
struct ManagementCanisterSchnorrPublicKeyRequest {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct ManagementCanisterSchnorrPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug, Clone)]
struct SchnorrKeyId {
    pub algorithm: SchnorrAlgorithm,
    pub name: String,
}

#[derive(CandidType, Serialize, Debug)]
struct ManagementCanisterSignatureRequest {
    pub message: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct ManagementCanisterSignatureReply {
    pub signature: Vec<u8>,
}

#[update]
// async fn public_key(algorithm: SchnorrAlgorithm) -> Result<PublicKeyReply, String> {
pub async fn solana_address() -> Result<String, String> {

    let publickey_result = schnorr_public_key().await;
    let ed25519_public_key_hex = hex::decode(publickey_result.unwrap().public_key_hex).unwrap();

    let address = bs58::encode(ed25519_public_key_hex).into_string();
    return Ok(address)
}

#[update]
// async fn public_key(algorithm: SchnorrAlgorithm) -> Result<PublicKeyReply, String> {
async fn schnorr_public_key() -> Result<PublicKeyReply, String> {
    let request = ManagementCanisterSchnorrPublicKeyRequest {
        canister_id: None,
        derivation_path: vec![],
        #[cfg(feature = "local")]
        key_id: SchnorrKeyIds::TestKeyLocalDevelopment.to_key_id(SchnorrAlgorithm::Ed25519),
        #[cfg(any(feature = "prod", feature = "dev"))]
        key_id: SchnorrKeyIds::TestKey1.to_key_id(SchnorrAlgorithm::Ed25519),
    };

    let (res,): (ManagementCanisterSchnorrPublicKeyReply,) = ic_cdk::call(
        Principal::management_canister(),
        "schnorr_public_key",
        (request,),
    )
    .await
    .map_err(|e| format!("schnorr_public_key failed {}", e.1))?;

    Ok(PublicKeyReply {
        public_key_hex: hex::encode(&res.public_key),
    })
}

#[update]
pub(crate) async fn schnorr_sign(message: String) -> Result<SignatureReply, String> {
    let internal_request = ManagementCanisterSignatureRequest {
        message: hex::decode(message).unwrap(),
        derivation_path: vec![],
        #[cfg(feature = "local")]
        key_id: SchnorrKeyIds::TestKeyLocalDevelopment.to_key_id(SchnorrAlgorithm::Ed25519),
        #[cfg(any(feature = "prod", feature = "dev"))]
        key_id: SchnorrKeyIds::TestKey1.to_key_id(SchnorrAlgorithm::Ed25519),    };

    let (internal_reply,): (ManagementCanisterSignatureReply,) =
        ic_cdk::api::call::call_with_payment(
            Principal::management_canister(),
            "sign_with_schnorr",
            (internal_request,),
            25_000_000_000,
        )
        .await
        .map_err(|e| format!("sign_with_schnorr failed {e:?}"))?;

    Ok(SignatureReply {
        signature_hex: hex::encode(&internal_reply.signature),
    })
}


enum SchnorrKeyIds {
    #[allow(unused)]
    TestKeyLocalDevelopment,
    #[allow(unused)]
    TestKey1,
    #[allow(unused)]
    ProductionKey1,
}

impl SchnorrKeyIds {
    fn to_key_id(&self, algorithm: SchnorrAlgorithm) -> SchnorrKeyId {
        SchnorrKeyId {
            algorithm,
            name: match self {
                Self::TestKeyLocalDevelopment => "dfx_test_key",
                Self::TestKey1 => "test_key_1",
                Self::ProductionKey1 => "key_1",
            }
            .to_string(),
        }
    }
}

// In the following, we register a custom getrandom implementation because
// otherwise getrandom (which is a dependency of k256) fails to compile.
// This is necessary because getrandom by default fails to compile for the
// wasm32-unknown-unknown target (which is required for deploying a canister).
// Our custom implementation always fails, which is sufficient here because
// we only use the k256 crate for verifying secp256k1 signatures, and such
// signature verification does not require any randomness.
getrandom::register_custom_getrandom!(always_fail);
pub fn always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::UNSUPPORTED)
}