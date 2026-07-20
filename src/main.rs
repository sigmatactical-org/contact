#![forbid(unsafe_code)]

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt::init();
    let addr = sigma_theme::warp::listen_addr_from_env();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let store = sigma_contact::store::ContactStore::connect().await?;
            sigma_theme::warp::serve("Sigma Contact", addr, sigma_contact::routes(store)).await?;
            Ok::<(), BoxError>(())
        })
}
