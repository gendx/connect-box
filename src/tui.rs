use crate::types::{ClientInfo, LanUserTable};
use std::cmp::Ordering;

pub struct Tui {
    window: *mut i8,
    size: Size,
    clients: Vec<ClientInfo>,
    has_colors: bool,
}

struct Size {
    x: i32,
    y: i32,
}

impl Tui {
    // Width and position of various fields.
    const MAC_WIDTH: i32 = 19;
    const IPV4_WIDTH: i32 = 19;
    const IPV6_WIDTH: i32 = 41;
    const SPEED_WIDTH: i32 = 7;
    const LEASE_WIDTH: i32 = 13;

    const MAC_POS: i32 = 0;
    const IPV4_POS: i32 = Self::MAC_POS + Self::MAC_WIDTH;
    const IPV6_POS: i32 = Self::IPV4_POS + Self::IPV4_WIDTH;
    const SPEED_POS: i32 = Self::IPV6_POS + Self::IPV6_WIDTH;
    const LEASE_POS: i32 = Self::SPEED_POS + Self::SPEED_WIDTH;
    const HOSTNAME_POS: i32 = Self::LEASE_POS + Self::LEASE_WIDTH;

    // Indices for ncurses color pairs.
    const ADDED_PAIR: i16 = 1;
    const REMOVED_PAIR: i16 = 2;
    const HIGHLIGHT_PAIR: i16 = 3;

    pub fn new() -> Self {
        let window = ncurses::initscr();
        ncurses::cbreak();
        ncurses::noecho();
        ncurses::intrflush(window, false);
        ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);

        let has_colors = ncurses::has_colors();
        if has_colors {
            ncurses::start_color();
            ncurses::init_pair(
                Self::ADDED_PAIR,
                ncurses::constants::COLOR_BLACK,
                ncurses::constants::COLOR_GREEN,
            );
            ncurses::init_pair(
                Self::REMOVED_PAIR,
                ncurses::constants::COLOR_WHITE,
                ncurses::constants::COLOR_RED,
            );
            ncurses::init_pair(
                Self::HIGHLIGHT_PAIR,
                ncurses::constants::COLOR_BLACK,
                ncurses::constants::COLOR_WHITE,
            );
        }

        Self {
            window,
            size: Size { x: 0, y: 0 },
            clients: Vec::new(),
            has_colors,
        }
    }

    pub fn update(&mut self, table: LanUserTable) {
        let mut newclients: Vec<ClientInfo> = table.wifi.clientinfo;
        newclients.sort_by(|x, y| x.mac.cmp(&y.mac));

        ncurses::clear();
        ncurses::getmaxyx(self.window, &mut self.size.y, &mut self.size.x);
        if self.has_colors {
            self.print_clients_diff(&self.clients, &newclients);
        } else {
            self.print_clients(&newclients);
        }
        ncurses::refresh();

        self.clients = newclients;
    }

    fn print_clients(&self, clients: &[ClientInfo]) {
        self.print_header();
        for (i, c) in clients.iter().enumerate() {
            self.print_client((i + 1) as i32, c);
        }
    }

    fn print_clients_diff(&self, oldclients: &[ClientInfo], newclients: &[ClientInfo]) {
        self.print_header_colored();

        let mut old_it = oldclients.iter().peekable();
        let mut new_it = newclients.iter().peekable();
        for i in 1.. {
            match (old_it.peek(), new_it.peek()) {
                (Some(o), Some(n)) => match o.mac.cmp(&n.mac) {
                    Ordering::Equal => {
                        self.print_client_diff(i, o, n);
                        new_it.next();
                        old_it.next();
                    }
                    Ordering::Less => {
                        self.print_removed_client(i, o);
                        old_it.next();
                    }
                    Ordering::Greater => {
                        self.print_new_client(i, n);
                        new_it.next();
                    }
                },
                (None, Some(n)) => {
                    self.print_new_client(i, n);
                    new_it.next();
                }
                (Some(o), None) => {
                    self.print_removed_client(i, o);
                    old_it.next();
                }
                (None, None) => break,
            }
        }
    }

    fn print_removed_client(&self, i: i32, client: &ClientInfo) {
        ncurses::attron(ncurses::COLOR_PAIR(Self::REMOVED_PAIR));
        self.print_client(i, client);
        ncurses::attroff(ncurses::COLOR_PAIR(Self::REMOVED_PAIR));
    }

    fn print_new_client(&self, i: i32, client: &ClientInfo) {
        ncurses::attron(ncurses::COLOR_PAIR(Self::ADDED_PAIR));
        self.print_client(i, client);
        ncurses::attroff(ncurses::COLOR_PAIR(Self::ADDED_PAIR));
    }

    fn print_client_diff(&self, i: i32, oldclient: &ClientInfo, newclient: &ClientInfo) {
        self.addstr_at(i, Self::MAC_POS, &newclient.mac);
        self.addstr_at_option_diff(
            i,
            Self::IPV4_POS,
            oldclient.ipv4.as_ref(),
            newclient.ipv4.as_ref(),
        );
        self.addstr_at_option_diff(
            i,
            Self::IPV6_POS,
            oldclient.ipv6.as_ref(),
            newclient.ipv6.as_ref(),
        );

        let raw_speed = format!("{:5}", newclient.speed);
        let trimmed_speed = raw_speed.trim_start();
        let trimmed_count = (raw_speed.len() - trimmed_speed.len()) as i32;
        self.addstr_at_cmp(
            i,
            Self::SPEED_POS + trimmed_count,
            &trimmed_speed,
            newclient.speed.cmp(&oldclient.speed),
        );

        self.addstr_at_diff(
            i,
            Self::LEASE_POS,
            &oldclient.lease_time,
            &newclient.lease_time,
        );
        self.addstr_at_diff(
            i,
            Self::HOSTNAME_POS,
            &oldclient.hostname,
            &newclient.hostname,
        );
    }

    fn print_header_colored(&self) {
        ncurses::attron(ncurses::A_BOLD());
        ncurses::attron(ncurses::A_STANDOUT());
        ncurses::wmove(self.window, 0, 0);
        for _ in 0..self.size.x {
            ncurses::waddch(self.window, ' '.into());
        }
        self.print_header();
        ncurses::attroff(ncurses::A_STANDOUT());
        ncurses::attroff(ncurses::A_BOLD());
    }

    fn print_header(&self) {
        self.addstr_at(0, Self::MAC_POS, "MAC");
        self.addstr_at(0, Self::IPV4_POS, "IPv4");
        self.addstr_at(0, Self::IPV6_POS, "IPv6");
        self.addstr_at(0, Self::SPEED_POS, "Speed");
        self.addstr_at(0, Self::LEASE_POS, "Lease");
        self.addstr_at(0, Self::HOSTNAME_POS, "Host");
    }

    fn print_client(&self, i: i32, client: &ClientInfo) {
        self.addstr_at(i, Self::MAC_POS, &client.mac);

        if let Some(ipv4) = &client.ipv4 {
            self.addstr_at(i, Self::IPV4_POS, ipv4);
        }
        if let Some(ipv6) = &client.ipv6 {
            self.addstr_at(i, Self::IPV6_POS, ipv6);
        }

        let raw_speed = format!("{:5}", client.speed);
        let trimmed_speed = raw_speed.trim_start();
        let trimmed_count = (raw_speed.len() - trimmed_speed.len()) as i32;
        self.addstr_at(i, Self::SPEED_POS + trimmed_count, &trimmed_speed);

        self.addstr_at(i, Self::LEASE_POS, &client.lease_time);
        self.addstr_at(i, Self::HOSTNAME_POS, &client.hostname);
    }

    fn addstr_at_option_diff(&self, y: i32, x: i32, old: Option<&String>, new: Option<&String>) {
        match (old, new) {
            (Some(o), Some(n)) => self.addstr_at_diff(y, x, o, n),
            (Some(o), None) => {
                self.addstr_at_pair(y, x, o, Self::REMOVED_PAIR);
            }
            (None, Some(n)) => {
                self.addstr_at_pair(y, x, n, Self::ADDED_PAIR);
            }
            (None, None) => (),
        }
    }

    fn addstr_at_cmp(&self, y: i32, x: i32, s: &str, cmp: Ordering) {
        match cmp {
            Ordering::Equal => {
                self.addstr_at(y, x, s);
            }
            Ordering::Less => {
                self.addstr_at_pair(y, x, s, Self::REMOVED_PAIR);
            }
            Ordering::Greater => {
                self.addstr_at_pair(y, x, s, Self::ADDED_PAIR);
            }
        }
    }

    fn addstr_at_diff(&self, y: i32, x: i32, old: &str, new: &str) {
        if new != old {
            self.addstr_at_pair(y, x, new, Self::HIGHLIGHT_PAIR);
        } else {
            self.addstr_at(y, x, new);
        }
    }

    fn addstr_at_pair(&self, y: i32, x: i32, s: &str, pair: i16) {
        ncurses::attron(ncurses::COLOR_PAIR(pair));
        self.addstr_at(y, x, s);
        ncurses::attroff(ncurses::COLOR_PAIR(pair));
    }

    fn addstr_at(&self, y: i32, x: i32, mut s: &str) {
        if y >= self.size.y {
            return;
        }
        if x >= self.size.x {
            return;
        }

        let available_len = (self.size.x - x) as usize;
        if s.len() >= available_len {
            s = &s[..available_len];
        }

        ncurses::wmove(self.window, y, x);
        ncurses::addstr(s);
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        ncurses::endwin();
    }
}
