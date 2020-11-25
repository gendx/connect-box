use clap::{App, Arg};

pub fn build_cli() -> App<'static, 'static> {
    App::new("ConnectBox")
        .version("0.1")
        .author("G. Endignoux <ggendx@gmail.com>")
        .about("Monitor your ConnectBox router")
        .arg(
            Arg::with_name("password")
                .short("p")
                .long("password")
                .takes_value(true)
                .required_unless("demo")
                .help("Password to connect to the router"),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .takes_value(true)
                .required_unless("demo")
                .help("IP address of the router"),
        )
        .arg(
            Arg::with_name("tui")
                .short("t")
                .long("tui")
                .help("Launch the ncurses-based TUI"),
        )
        .arg(
            Arg::with_name("demo")
                .long("demo")
                .help("Use demonstration data"),
        )
        .arg(
            Arg::with_name("refresh")
                .long("refresh")
                .takes_value(true)
                .default_value("3")
                .validator(|value| {
                    let i: u64 = value
                        .parse()
                        .map_err(|_| "The refresh period must be a positive integer".to_owned())?;
                    if i < 1 {
                        return Err("The refresh period must be at least 1 second".to_owned());
                    }
                    if i > 600 {
                        return Err("The refresh period must be at most 10 minutes".to_owned());
                    }
                    Ok(())
                })
                .help("Target refresh period of the dashboard, in seconds"),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .takes_value(true)
                .default_value("10")
                .validator(|value| {
                    let i: u64 = value
                        .parse()
                        .map_err(|_| "The timeout must be a positive integer".to_owned())?;
                    if i < 5 {
                        return Err("The timeout must be at least 5 seconds".to_owned());
                    }
                    if i > 600 {
                        return Err("The timeout must be at most 10 minutes".to_owned());
                    }
                    Ok(())
                })
                .help("Timeout for each request to the router, in seconds"),
        )
        .arg(
            Arg::with_name("throttle")
                .long("throttle")
                .takes_value(true)
                .default_value("3")
                .validator(|value| {
                    let i: u64 = value.parse().map_err(|_| {
                        "The throttle duration must be a positive integer".to_owned()
                    })?;
                    if i < 1 {
                        return Err("The throttle duration must be at least 1 second".to_owned());
                    }
                    if i > 600 {
                        return Err("The throttle duration must be at most 10 minutes".to_owned());
                    }
                    Ok(())
                })
                .help("Duration between retries in case of a connection error, in seconds"),
        )
}
