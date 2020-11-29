mod cli;
mod connect_box;
mod demo;
mod router;
mod tui;
mod types;

use connect_box::ConnectBox;
use demo::DemoRouter;
use futures::future::FutureExt;
use futures::select;
use futures::stream;
use log::{debug, info};
use router::Router;
use std::net::Ipv4Addr;
use tokio::stream::StreamExt;
use tokio::{signal, time};
use tui::Tui;
use types::LanUserTableDiff;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // CLI.
    let mut app = cli::build_cli();

    let matches = match app.get_matches_from_safe_borrow(std::env::args_os()) {
        Ok(matches) => matches,
        Err(error) => error.exit(),
    };

    let demo = matches.is_present("demo");
    let tui = matches.is_present("tui");
    let refresh: u64 = matches.value_of("refresh").unwrap().parse().unwrap();

    if demo {
        let mut router = DemoRouter::new();
        launch_with_router(&mut router, time::Duration::from_secs(refresh), tui).await
    } else {
        let host = matches.value_of("host").unwrap();
        let password = matches.value_of("password").unwrap();
        let timeout: u64 = matches.value_of("timeout").unwrap().parse().unwrap();
        let throttle: u64 = matches.value_of("throttle").unwrap().parse().unwrap();

        let hostip: Ipv4Addr = host.parse()?;

        let mut router = ConnectBox::new(
            hostip,
            password,
            time::Duration::from_secs(timeout),
            time::Duration::from_secs(throttle),
        )
        .await?;
        launch_with_router(&mut router, time::Duration::from_secs(refresh), tui).await
    }
}

async fn launch_with_router<R: Router>(
    router: &mut R,
    refresh_duration: time::Duration,
    tui: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    select!(
        res = main_loop(router, refresh_duration, tui).fuse() => res,
        res = wait_interrupt().fuse() => res,
    )?;

    info!("Selected");

    router.logout().await?;
    Ok(())
}

async fn wait_interrupt() -> Result<(), Box<dyn std::error::Error>> {
    signal::ctrl_c().await?;
    info!("CTRL-C received!");
    Ok(())
}

async fn main_loop<R: Router>(
    router: &mut R,
    refresh_duration: time::Duration,
    tui: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if tui {
        tui_loop(router, refresh_duration).await
    } else {
        diff_loop(router, refresh_duration).await
    }
}

async fn diff_loop<R: Router>(
    router: &mut R,
    refresh_duration: time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut devices = router.devices().await?;
    println!("Devices: {:#?}", devices);
    let temperature = router.temperature().await?;
    println!("Temperature: {:#?}", temperature);

    let mut throttle = stream::repeat(()).throttle(refresh_duration);
    loop {
        debug!("Throttling...");
        throttle.next().await;
        debug!("Querying for devices...");
        let newdevices = router.devices().await?;
        println!(
            "Devices: {:#?}",
            LanUserTableDiff {
                old: &devices,
                new: &newdevices
            }
        );
        devices = newdevices;
    }
}

async fn tui_loop<R: Router>(
    router: &mut R,
    refresh_duration: time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tui = Tui::new();

    let mut throttle = stream::repeat(()).throttle(refresh_duration);
    loop {
        debug!("Throttling...");
        throttle.next().await;
        debug!("Querying for devices...");
        let devices = router.devices().await?;
        tui.update(devices);
    }
}
