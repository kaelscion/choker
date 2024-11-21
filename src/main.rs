mod limiters;

use hyper::upgrade::Upgraded;
use async_speed_limit::limiter::Limiter;
use std::panic::catch_unwind;
use crate::limiters::{LimitedStream, LimitedUpgraded};


async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let download_limiter = Limiter::new(5000000.0);
    let upload_limiter = download_limiter.clone();
    let mut server = LimitedStream::connect(&addr, upload_limiter).await?;
    let mut client = LimitedUpgraded::from_upgraded(upgraded, download_limiter).await?;
    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut client, &mut server).await?;

    tracing::debug!(
        "client wrote {} bytes and received {} bytes",
        from_client,
        from_server
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error>{
    match catch_unwind(|| tunnel) {
        Ok(_) => println!("Exited successfully"),
        Err(_) => println!("Exited with error")
    }
    Ok(())
}