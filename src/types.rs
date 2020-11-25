use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanUserTable {
    #[serde(rename = "Customer")]
    pub customer: String,
    #[serde(rename = "totalClient")]
    pub total_client: usize,
    #[serde(rename = "WIFI")]
    pub wifi: Wifi,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wifi {
    pub clientinfo: Vec<ClientInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientInfo {
    #[serde(rename = "MACAddr")]
    pub mac: String,
    pub hostname: String,
    pub index: usize,
    #[serde(rename = "IPv4Addr")]
    pub ipv4: Option<String>,
    #[serde(rename = "IPv6Addr")]
    pub ipv6: Option<String>,
    pub interface: String,
    pub interfaceid: usize,
    pub method: usize,
    #[serde(rename = "leaseTime")]
    pub lease_time: String,
    pub speed: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CmState {
    #[serde(rename = "TunnerTemperature")]
    pub tunner_temperature: usize,
    #[serde(rename = "Temperature")]
    pub temperature: usize,
    #[serde(rename = "OperState")]
    pub oper_state: String,
    pub wan_ipv6_addr: WanIpv6Addr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WanIpv6Addr {
    #[serde(rename = "wan_ipv6_addr_entry")]
    pub addresses: Vec<String>,
}

pub struct LanUserTableDiff<'a> {
    pub old: &'a LanUserTable,
    pub new: &'a LanUserTable,
}

impl fmt::Debug for LanUserTableDiff<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.old == self.new {
            return Ok(());
        }

        let mut dbg = f.debug_struct("LanUserTable");
        if self.old.customer != self.new.customer {
            dbg.field(
                "customer",
                &format_args!("{:?} => {:?}", self.old.customer, self.new.customer),
            );
        }
        if self.old.total_client != self.new.total_client {
            dbg.field(
                "total_client",
                &format_args!("{:?} => {:?}", self.old.total_client, self.new.total_client),
            );
        }
        if self.old.wifi != self.new.wifi {
            dbg.field(
                "wifi",
                &WifiDiff {
                    old: &self.old.wifi,
                    new: &self.new.wifi,
                },
            );
        }
        dbg.finish()
    }
}

struct WifiDiff<'a> {
    old: &'a Wifi,
    new: &'a Wifi,
}

impl fmt::Debug for WifiDiff<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.old == self.new {
            return Ok(());
        }

        f.debug_struct("Wifi")
            .field(
                "clientinfo",
                &ClientInfosDiff {
                    old: &self.old.clientinfo,
                    new: &self.new.clientinfo,
                },
            )
            .finish()
    }
}

struct ClientInfosDiff<'a> {
    old: &'a [ClientInfo],
    new: &'a [ClientInfo],
}

impl fmt::Debug for ClientInfosDiff<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.old == self.new {
            return Ok(());
        }

        let mut dbg = f.debug_list();
        let mut sorted_old = self.old.iter().collect::<Vec<_>>();
        sorted_old.sort_by_key(|x| &x.mac);
        let mut sorted_new = self.new.iter().collect::<Vec<_>>();
        sorted_new.sort_by_key(|x| &x.mac);

        let mut old_it = sorted_old.iter().peekable();
        let mut new_it = sorted_new.iter().peekable();
        loop {
            match (old_it.peek(), new_it.peek()) {
                (Some(o), Some(n)) => match o.mac.cmp(&n.mac) {
                    Ordering::Equal => {
                        if o != n {
                            dbg.entry(&ClientInfoDiff { old: o, new: n });
                        }
                        new_it.next();
                        old_it.next();
                    }
                    Ordering::Less => {
                        dbg.entry(&format_args!("{:#?} => ()", o));
                        old_it.next();
                    }
                    Ordering::Greater => {
                        dbg.entry(&format_args!("() => {:#?}", n));
                        new_it.next();
                    }
                },
                (None, Some(n)) => {
                    dbg.entry(&format_args!("() => {:#?}", n));
                    new_it.next();
                }
                (Some(o), None) => {
                    dbg.entry(&format_args!("{:#?} => ()", o));
                    old_it.next();
                }
                (None, None) => break,
            }
        }
        dbg.finish()
    }
}

struct ClientInfoDiff<'a> {
    old: &'a ClientInfo,
    new: &'a ClientInfo,
}

impl fmt::Debug for ClientInfoDiff<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.old == self.new {
            return Ok(());
        }

        let mut dbg = f.debug_struct("ClientInfo");
        if self.old.mac != self.new.mac {
            dbg.field(
                "mac",
                &format_args!("{:?} => {:?}", self.old.mac, self.new.mac),
            );
        } else {
            dbg.field("mac", &self.old.mac);
        }

        if self.old.hostname != self.new.hostname {
            dbg.field(
                "hostname",
                &format_args!("{:?} => {:?}", self.old.hostname, self.new.hostname),
            );
        } else {
            dbg.field("hostname", &self.old.hostname);
        }

        if self.old.index != self.new.index {
            dbg.field(
                "index",
                &format_args!("{:?} => {:?}", self.old.index, self.new.index),
            );
        }
        if self.old.ipv4 != self.new.ipv4 {
            dbg.field(
                "ipv4",
                &format_args!("{:?} => {:?}", self.old.ipv4, self.new.ipv4),
            );
        }
        if self.old.ipv6 != self.new.ipv6 {
            dbg.field(
                "ipv6",
                &format_args!("{:?} => {:?}", self.old.ipv6, self.new.ipv6),
            );
        }
        if self.old.interface != self.new.interface {
            dbg.field(
                "interface",
                &format_args!("{:?} => {:?}", self.old.interface, self.new.interface),
            );
        }
        if self.old.interfaceid != self.new.interfaceid {
            dbg.field(
                "interfaceid",
                &format_args!("{:?} => {:?}", self.old.interfaceid, self.new.interfaceid),
            );
        }
        if self.old.method != self.new.method {
            dbg.field(
                "method",
                &format_args!("{:?} => {:?}", self.old.method, self.new.method),
            );
        }
        if self.old.lease_time != self.new.lease_time {
            dbg.field(
                "lease_time",
                &format_args!("{:?} => {:?}", self.old.lease_time, self.new.lease_time),
            );
        }
        if self.old.speed != self.new.speed {
            dbg.field(
                "speed",
                &format_args!("{:?} => {:?}", self.old.speed, self.new.speed),
            );
        }
        dbg.finish()
    }
}
