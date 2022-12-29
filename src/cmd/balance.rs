use crate::backend::{
    sube::initialize_client,
    wallet::{get_account, get_wallet},
};
use crate::model::{error::Error, result::Result};

use sube::Error::StorageKeyNotFound;

pub async fn balance(uname: String) -> Result<()> {
    // Initialize wallet
    let wallet = get_wallet(&uname).await?;
    let account = get_account(&wallet).await?;
    let hex_public_key = format!("0x{}", account);

    let sube = initialize_client("wss://westend-rpc.polkadot.io")
        .await
        .map_err(move |e| Error::Sube(Box::new(e)))?;

    // Get balance
    let result = sube
        .query(&format!("system/account/{}", &hex_public_key))
        .await
        .map_err(move |e| {
            if matches!(StorageKeyNotFound, e) {
                println!(
                    "The account you are looking for does not exist on the chain.
Try using another account, or maybe activating your account first"
                );
            } else {
                dbg!(&e);
            }

            Error::Sube(Box::new(e))
        })?;

    println!("{}", serde_json::to_string_pretty(&result).unwrap());

    Ok(())
}
