use crate::errors::Error;
use dirs::home_dir;
use libwallet::{
    self,
    vault::{Pass, PassCreds},
    Account, Language,
};
use sp_core::{crypto::Ss58Codec, sr25519::Public};
use std::str::FromStr;

pub type Wallet = libwallet::Wallet<Pass>;

pub async fn get_wallet(uname: &str) -> Result<Wallet, Box<dyn std::error::Error>> {
    let mut store_path = home_dir().ok_or(Error::Dir)?;
    store_path.push(".password-store");
    let store_path = store_path.to_str().ok_or(Error::Dir)?;

    let vault = Pass::new(store_path, Language::default());
    let mut wallet = Wallet::new(vault);
    wallet
        .unlock(PassCreds {
            secret_path: String::from(uname),
        })
        .await?;

    println!("Get account for {}:", uname);
    Ok(wallet)
}

pub async fn get_account(wallet: &Wallet) -> Result<&Account, Box<dyn std::error::Error>> {
    let account = wallet.default_account().clone();
    let public_key = account.public();

    let bytes = public_key.as_ref().to_vec();
    let mut bytes_vec = [0 as u8; 32];

    for ix in 0..(bytes.len()) {
        let byte = bytes.get(ix).ok_or(Error::PkDecode)?;
        bytes_vec[ix] = *byte;
    }

    let pk_string = format!("0x{}", account);
    let address = get_address(&pk_string).await?.to_ss58check();

    println!(
        "Public Key: {}
SS58 Address: {}\n",
        pk_string, address
    );

    Ok(account)
}

pub async fn get_address(address: &str) -> Result<Public, Box<dyn std::error::Error>> {
    let public = Public::from_str(address).unwrap_or_else(|_| {
        let dest_bytes = if address.starts_with("0x") {
            hex::decode(&address[2..])
                .expect("address: hex-compatible format")
                .to_vec()
        } else {
            address[..].as_bytes().to_vec()
        };

        sp_core::sr25519::Public(
            dest_bytes
                .try_into()
                .expect("address: incorrect key lenght"),
        )
    });

    Ok(public)
}
