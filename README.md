# connect-box

Rust client to interact with the UPC Connect Box router, and monitor connected devices from your terminal.
This is inspired by [python-connect-box](https://github.com/home-assistant-ecosystem/python-connect-box).

This software is not official, developed, supported or endorsed by UPC.

## Usage

```
$ connect-box --help
ConnectBox 0.1
G. Endignoux <ggendx@gmail.com>
Monitor your ConnectBox router

USAGE:
    connect-box [FLAGS] [OPTIONS] --host <host> --password <password>

FLAGS:
        --demo       Use demonstration data
        --help       Prints help information
    -t, --tui        Launch the ncurses-based TUI
    -V, --version    Prints version information

OPTIONS:
    -h, --host <host>            IP address of the router
    -p, --password <password>    Password to connect to the router
        --refresh <refresh>      Target refresh period of the dashboard, in seconds [default: 3]
        --throttle <throttle>    Duration between retries in case of a connection error, in seconds [default: 3]
        --timeout <timeout>      Timeout for each request to the router, in seconds [default: 10]
```

## License

MIT
