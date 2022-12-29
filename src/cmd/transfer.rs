use crate::{
    backend::{
        sube::{initialize_client, SubeClient},
        wallet::{get_account, get_address, get_wallet, sign},
    },
    model::{
        error::{Cause, Error},
        result::Result,
    },
};

use parity_scale_codec::{Compact, Encode};
use sp_core::{crypto::Ss58Codec, hexdisplay::AsBytesRef};
use sube::{
    meta::{Meta, Pallet},
    Backend,
};

/// Builds a [transfer extrinsic](https://github.com/paritytech/subxt/blob/8484c18624783af36476fc5bf6a0f08d5363a3db/subxt/src/tx/tx_client.rs#L124-L207), then
/// sends it using sube
pub async fn transfer(uname: String, dest: String, value: u128) -> Result<()> {
    let sube = initialize_client("wss://westend-rpc.polkadot.io")
        .await
        .map_err(|e| Error::Sube(Box::new(e)))?;

    let call_data = construct_transfer_call(&sube, &dest, value).await?;
    let wallet = get_wallet(&uname).await?;
    let account = get_account(&wallet).await?;
    let from_address = format!("0x{}", &account);

    let (extra_params, signature_payload) =
        construct_extrinsic_data(&sube, &from_address, &call_data).await?;

    let signature = sign(&wallet, signature_payload).await?;

    let extrinsic_call = {
        let encoded_inner = [
            // header: "is signed" (1 byte) + transaction protocol version (7 bytes)
            vec![0b10000000 + 4u8],
            // signer
            vec![0x00],
            account.public().as_ref().to_vec(),
            // signature
            signature,
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

    sube.submit(extrinsic_call).await.map_err(move |e| {
        dbg!(&e);
        Error::Sube(Box::new(e))
    })?;

    Ok(())
}

async fn construct_transfer_call(sube: &SubeClient, dest: &str, value: u128) -> Result<Vec<u8>> {
    let meta = sube
        .metadata()
        .await
        .map_err(move |e| Error::Sube(Box::new(e)))?;

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
        Error::Sube(Box::new(e))
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

async fn construct_extrinsic_data(
    sube: &SubeClient,
    from_address: &str,
    call_data: &Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let extra_params = {
        // ImmortalEra
        let era = 0u8;

        // Impl. Note: in a real-world use case, you should store your account's nonce somewhere else
        let account_info = sube
            .query(&format!("system/account/{}", &from_address))
            .await
            .map_err(move |e| Error::Sube(Box::new(e)))?;
        let account_info =
            <serde_json::Value as std::str::FromStr>::from_str(&account_info.to_string())
                .map_err(move |e| Error::Codec(Box::new(e)))?;
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
        let metadata = sube
            .metadata()
            .await
            .map_err(|e| Error::Sube(Box::new(e)))?
            .clone();

        let mut constants = metadata
            .pallet_by_name("System")
            .ok_or(Error::Codec(Box::new(Cause::from(
                "System pallet not found on metadata",
            ))))?
            .constants
            .clone()
            .into_iter();

        let data = constants
            .find(|c| c.name == "Version")
            .ok_or(Error::Codec(Box::new(Cause::from(
                "System_Version constant not found",
            ))))?
            .clone()
            .to_owned();

        let chain_version = sube
            .decode(data.value.to_vec(), data.ty.id())
            .await
            .map_err(|e| Error::Codec(Box::new(e)))?;
        let chain_version =
            serde_json::to_value(chain_version).map_err(|e| Error::Codec(Box::new(e)))?;

        let spec_version = chain_version
            .get("spec_version")
            .ok_or(Error::Codec(Box::new(Cause::from("spec_version not found"))))?
            .as_u64()
            .ok_or(Error::Codec(Box::new(Cause::from("spec_version not a Number"))))? as u32;
        let transaction_version = chain_version
            .get("transaction_version")
            .ok_or(Error::Codec(Box::new(Cause::from("transaction_version not found"))))?
            .as_u64()
            .ok_or(Error::Codec(Box::new(Cause::from("transaction_version not a Number"))))? as u32;
        let genesis_block: Vec<u8> = sube
            .block_info(Some(0u32))
            .await
            .map_err(move |e| Error::Sube(Box::new(e)))?
            .into();

        [
            spec_version.to_le_bytes().to_vec(),
            transaction_version.to_le_bytes().to_vec(),
            genesis_block.clone(),
            genesis_block.clone(),
        ]
        .concat()
    };

    let signature_payload = [
        call_data.clone(),
        extra_params.clone(),
        additional_params.clone(),
    ]
    .concat();

    Ok((extra_params, signature_payload))
}
