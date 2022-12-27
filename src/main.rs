mod errors;
mod wallet;

use libwallet::Signer;
use parity_scale_codec::{Compact, Encode};
use sp_core::{crypto::Ss58Codec, hexdisplay::AsBytesRef};
use structopt::StructOpt;

use anyhow::Result;

use sube::{
    self,
    meta::{Entry, Meta, Pallet},
    ws, Backend, StorageKey, Sube, Value,
};

use crate::wallet::{get_account, get_wallet};
use crate::{errors::Error, wallet::get_address};

#[derive(StructOpt, Debug)]
#[structopt(name = "virto-wallet")]
struct Cli {
    // Uname of the wallet to retrieve
    #[structopt(short, long)]
    uname: String,

    #[structopt(subcommand)]
    cmd: CliCmd,
}

#[derive(StructOpt, Debug)]
enum CliCmd {
    Balance,
    Transfer {
        #[structopt(short, long)]
        dest: String,
        #[structopt(short, long)]
        value: u128,
    },
}

#[async_std::main]
async fn main() {
    match run().await {
        Ok(_) => {}
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::from_args();
    let uname = cli.uname;

    match cli.cmd {
        CliCmd::Balance => balance(uname).await,
        CliCmd::Transfer { dest, value } => transfer(uname, dest, value).await,
    }
}

async fn balance(uname: String) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = get_wallet(&uname).await?;
    let account = get_account(&wallet).await?;
    let hex_public_key = format!("0x{}", account);

    // Get balance
    let westend_backend = ws::Backend::new_ws2("wss://westend-rpc.polkadot.io").await?;
    let metadata = westend_backend.metadata().await?;
    let reg = metadata.clone().types;

    let key = StorageKey::new(&metadata, "System", "Account", &[&hex_public_key]).map_err(|e| e)?;
    println!(
        "Encoded storage key: 0x{}",
        key.as_ref()
            .clone()
            .into_iter()
            .map(|b| format!("{:02x}", b).to_string())
            .collect::<Vec<String>>()
            .join("")
    );

    let result = westend_backend.query_storage(&key).await?;
    let entry = metadata
        .storage_entry("System", "Account")
        .ok_or(Error::DecodeStorage)?;

    let result = Value::new(result, entry.ty_id(), &reg);

    println!("{}", result.to_string());

    Ok(())
}

/// Builds a [transfer extrinsic](https://github.com/paritytech/subxt/blob/8484c18624783af36476fc5bf6a0f08d5363a3db/subxt/src/tx/tx_client.rs#L124-L207), then
/// sends it using sube
async fn transfer(
    uname: String,
    dest: String,
    value: u128,
) -> Result<(), Box<dyn std::error::Error>> {
    let westend_backend = ws::Backend::new_ws2("wss://westend-rpc.polkadot.io").await?;
    let sube = Sube::new(westend_backend);

    let call_data = construct_transfer_call(&sube, &dest, value).await?;

    let wallet = get_wallet(&uname).await?;
    let account = get_account(&wallet).await?;
    let from_hex = &format!("0x{}", &account);

    let extra_params = {
        let era = 0u8; // ImmortalEra
        // Impl. Note: in a real-world use case, you should store your account's nonce somewhere else
        let account_info = sube.query(&format!("system/account/{}", &from_hex)).await?;
        let account_info =
            <serde_json::Value as std::str::FromStr>::from_str(&account_info.to_string())?;
        let nonce = account_info
            .get("nonce")
            .unwrap_or(&serde_json::json!(0))
            .as_u64()
            .expect("nonce be a number");
        let tip: u128 = 0;

        [vec![era], Compact(nonce).encode(), Compact(tip).encode()].concat()
    };

    let additional_params = {
        // Error: Still failing to deserialize the const
        // let mut constants = sube
        //     .metadata()
        //     .await?
        //     .pallet_by_name("System")
        //     .expect("System pallet should exist")
        //     .constants
        //     .iter();
        // let data = constants
        //     .find(|c| c.name == "Version");
        // let data = data.expect("System_version constant should exist");
        // let chain_version = sube.decode(data.value.clone(), data.ty.id()).await?;
        // println!("{}", serde_json::to_string_pretty(&chain_version)?);

        let spec_version = 9360u32;
        let transaction_version = 17u32;
        let genesis_block: Vec<u8> = sube.block_info(Some(0u32)).await?.into();

        [
            spec_version.to_le_bytes().to_vec(),
            transaction_version.to_le_bytes().to_vec(),
            genesis_block.clone(),
            genesis_block.clone(),
        ]
        .concat()
    };

    let extra_params_hex = hex::encode(&extra_params);
    println!("Extra params: {}", extra_params_hex);

    let signature_payload = [
        // Compact(call_data.len() as u8).encode(),
        call_data.clone(),
        extra_params.clone(),
        additional_params.clone(),
    ]
    .concat();

    let signature = {
        let bytes = if signature_payload.len() > 256 {
            sube::hasher::hash(&sube::meta_ext::Hasher::Blake2_256, &signature_payload[..])
        } else {
            signature_payload.clone()
        };

        println!("Payload to sign: 0x{}", hex::encode(&bytes));
        wallet.sign(bytes.as_slice())
    };

    let signature_hex = hex::encode(&signature);
    println!("Signature: 0x01{}", signature_hex);
    println!(
        "is verifiable? {}",
        wallet
            .default_account()
            .verify(signature_payload, signature.as_ref())
    );

    let extrinsic_call = {
        let encoded_inner = [
            // header: "is signed" (1 byte) + transaction protocol version (7 bytes)
            vec![0b10000000 + 4u8],
            // signer
            vec![0x00],
            account.public().as_ref().to_vec(),
            // signature
            vec![0x01],
            signature.as_ref().to_vec(),
            // extra
            extra_params,
            // call data
            call_data,
        ]
        .concat();

        let len = Compact(
            u32::try_from(encoded_inner.len()).expect("extrinsic size expected to be <4GB"),
        )
        .encode();

        [len, encoded_inner].concat()
    };

    println!("Ready to go: 0x{}", hex::encode(&extrinsic_call));

    sube.submit(extrinsic_call).await.map_err(|e| {
        dbg!(&e);
        e
    })?;

    Ok(())
}

async fn construct_transfer_call<B>(
    sube: &Sube<B>,
    dest: &str,
    value: u128,
) -> Result<Vec<u8>, Box<dyn std::error::Error>>
where
    B: sube::Backend,
{
    let meta = sube.metadata().await?;

    let (call_ty, pallet_index) = {
        let pallet = meta
            .pallet_by_name("Balances")
            .expect("pallet does not exist");
        (
            pallet
                .get_calls()
                .expect("pallet does not have calls")
                .ty
                .id(),
            pallet.index,
        )
    };

    let dest = get_address(&dest).await?;

    let mut encoded_call = vec![pallet_index];
    let call_payload = serde_json::json!({
        "transfer": {
            "dest": {
                "Id": dest.as_bytes_ref(),
            },
            "value": value as u64,
        }
    });

    let call_data = sube.encode(call_payload, call_ty).await.map_err(|e| {
        dbg!(&e);
        e
    })?;

    encoded_call.extend(call_data);

    println!(
        "
Transferring {} to {} (0x{})
    Hex-encoded call: {}\n",
        value,
        &dest.to_ss58check(),
        hex::encode(&dest),
        format!("0x{}", hex::encode(&encoded_call)),
    );

    Ok(encoded_call)
}
