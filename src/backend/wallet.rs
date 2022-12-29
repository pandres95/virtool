use crate::model::{
    error::{Cause, Error},
    result::Result,
};
use dirs::home_dir;
use libwallet::{self, vault::Pass, Account, Language, Signer};
use sp_core::{crypto::Ss58Codec, sr25519::Public};
use std::str::FromStr;

pub type Wallet = libwallet::Wallet<Pass>;

pub async fn get_wallet(uname: &str) -> Result<Wallet> {
    let mut store_path = home_dir().ok_or(Error::Wallet(Box::new(Cause::from(
        "Could not find the HOME dir. Are you using a Linux/Windows/Mac based OS?",
    ))))?;
    store_path.push(".password-store");
    let store_path = store_path
        .to_str()
        .ok_or(Error::Wallet(Box::new(Cause::from(
            "Error opening secrets path",
        ))))?;

    let vault = Pass::new(store_path, Language::default());
    let mut wallet = Wallet::new(vault);
    wallet
        .unlock(String::from(uname))
        .await
        .map_err(|e| Error::Wallet(Box::new(e)))?;

    println!("Get account for {}:", uname);
    Ok(wallet)
}

pub async fn get_account(wallet: &Wallet) -> Result<&Account> {
    let account = wallet.default_account().clone();

    let pk_string = format!("0x{}", account);
    let address = get_address(&pk_string).await?.to_ss58check();

    println!(
        "Public Key: {}
SS58 Address: {}\n",
        pk_string, address
    );

    Ok(account)
}

pub async fn get_address(address: &str) -> Result<Public> {
    let public = match Public::from_str(address) {
        Ok(r) => r,
        Err(_) => {
            let dest_bytes = if address.starts_with("0x") {
                hex::decode(&address[2..]).map_err(|e| Error::Codec(Box::new(e)))?
            } else {
                address[..].as_bytes().to_vec()
            };

            let dest_bytes = dest_bytes
                .try_into()
                .map_err(|_| Error::Codec(Box::new(Cause::from("Invalid public key length"))))?;

            sp_core::sr25519::Public(dest_bytes)
        }
    };

    Ok(public)
}

pub async fn sign(wallet: &Wallet, payload: Vec<u8>) -> Result<Vec<u8>> {
    let payload = if payload.len() > 256 {
        sube::hasher::hash(&sube::meta_ext::Hasher::Blake2_256, &payload[..])
    } else {
        payload.clone()
    };
    println!("Payload to sign: 0x{}", hex::encode(&payload));

    let signature = wallet.sign(payload.as_slice());
    println!("Signature: 0x01{}", hex::encode(&signature));

    let verifiable = wallet.default_account().verify(payload, signature.as_ref());
    println!("Is signature verifiable? {}", verifiable);

    if !verifiable {
        return Err(Error::Wallet(Box::new(Cause::from(
            "Signature could not be verifiable",
        ))));
    }

    Ok([vec![0x01], signature.as_ref().to_vec()].concat())
}
