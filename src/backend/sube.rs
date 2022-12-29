use sube::{ws::{Backend, WS2}, Result, Sube};

pub type SubeClient = Sube<Backend<WS2>>;

pub async fn initialize_client(chain_url: &str) -> Result<SubeClient> {
    let backend = Backend::new_ws2(chain_url).await?;
    Ok(Sube::from(backend))
}
