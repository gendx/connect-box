use crate::router::Router;
use crate::types::{ClientInfo, CmState, LanUserTable, WanIpv6Addr, Wifi};
use async_trait::async_trait;

pub struct DemoRouter {
    states: Vec<LanUserTable>,
    i: usize,
}

impl DemoRouter {
    // Fill in some demo data.
    pub fn new() -> Self {
        let mut states = Vec::new();

        // State #0
        let mut clients = vec![
            ClientInfo {
                mac: "AB:CD:EF:01:23:45".to_owned(),
                hostname: "laptop".to_owned(),
                index: 0,
                ipv4: Some("192.168.0.1".to_owned()),
                ipv6: None,
                interface: "foo".to_owned(),
                interfaceid: 0,
                method: 0,
                lease_time: "00:00:47:57".to_owned(),
                speed: 1,
            },
            ClientInfo {
                mac: "CD:EF:01:23:45:AB".to_owned(),
                hostname: "My Super Phone".to_owned(),
                index: 0,
                ipv4: Some("192.168.0.42".to_owned()),
                ipv6: Some("2001:2345:6789:abcd:ef01:1010:3564:2".to_owned()),
                interface: "foo".to_owned(),
                interfaceid: 0,
                method: 0,
                lease_time: "00:00:38:18".to_owned(),
                speed: 123,
            },
            ClientInfo {
                mac: "EF:01:23:45:AB:CD".to_owned(),
                hostname: "Desktop".to_owned(),
                index: 0,
                ipv4: None,
                ipv6: Some("2001:2345:6789:abcd:ef01:1010:3564:5".to_owned()),
                interface: "foo".to_owned(),
                interfaceid: 0,
                method: 0,
                lease_time: "00:00:52:45".to_owned(),
                speed: 42,
            },
        ];

        states.push(LanUserTable {
            customer: "Customer".to_owned(),
            total_client: clients.len(),
            wifi: Wifi {
                clientinfo: clients.clone(),
            },
        });

        // State #1
        clients[0].lease_time = "00:00:47:54".to_owned();
        clients[1].lease_time = "00:00:38:15".to_owned();
        clients[1].speed = 234;
        clients[2].lease_time = "00:00:52:42".to_owned();
        clients[2].speed = 23;

        states.push(LanUserTable {
            customer: "Customer".to_owned(),
            total_client: clients.len(),
            wifi: Wifi {
                clientinfo: clients.clone(),
            },
        });

        // State #2
        clients[0].lease_time = "00:00:47:51".to_owned();
        clients[0].speed = 17;
        clients[0].ipv6 = Some("2001:2345:6789:abcd:ef01:1010:3564:888".to_owned());
        clients[1].lease_time = "00:00:38:12".to_owned();
        clients[1].ipv4 = None;
        clients[2].lease_time = "00:00:52:39".to_owned();
        clients[2].speed = 67;

        states.push(LanUserTable {
            customer: "Customer".to_owned(),
            total_client: clients.len(),
            wifi: Wifi {
                clientinfo: clients.clone(),
            },
        });

        // State #3
        clients[0].lease_time = "00:00:47:48".to_owned();
        clients[1].lease_time = "00:00:38:09".to_owned();
        clients[2].lease_time = "00:00:52:36".to_owned();

        states.push(LanUserTable {
            customer: "Customer".to_owned(),
            total_client: clients.len(),
            wifi: Wifi {
                clientinfo: clients.clone(),
            },
        });

        // State #4
        clients[0].lease_time = "00:00:47:45".to_owned();
        clients[2].lease_time = "00:00:52:33".to_owned();
        clients[2].speed = 128;
        clients.remove(1);

        states.push(LanUserTable {
            customer: "Customer".to_owned(),
            total_client: clients.len(),
            wifi: Wifi {
                clientinfo: clients.clone(),
            },
        });

        // State #5
        clients[0].lease_time = "00:00:47:42".to_owned();
        clients[1].lease_time = "00:00:52:30".to_owned();

        states.push(LanUserTable {
            customer: "Customer".to_owned(),
            total_client: clients.len(),
            wifi: Wifi {
                clientinfo: clients.clone(),
            },
        });

        // State #6
        clients[0].lease_time = "00:00:47:39".to_owned();
        clients[1].lease_time = "00:00:52:27".to_owned();
        clients.push(ClientInfo {
            mac: "01:23:45:AB:CD:EF".to_owned(),
            hostname: "Connected TV".to_owned(),
            index: 0,
            ipv4: Some("192.168.0.123".to_owned()),
            ipv6: None,
            interface: "foo".to_owned(),
            interfaceid: 0,
            method: 0,
            lease_time: "00:00:59:59".to_owned(),
            speed: 123,
        });

        states.push(LanUserTable {
            customer: "Customer".to_owned(),
            total_client: clients.len(),
            wifi: Wifi {
                clientinfo: clients.clone(),
            },
        });

        // State #7
        clients[0].lease_time = "00:00:47:36".to_owned();
        clients[1].lease_time = "00:00:52:24".to_owned();
        clients[2].lease_time = "00:00:59:56".to_owned();

        states.push(LanUserTable {
            customer: "Customer".to_owned(),
            total_client: clients.len(),
            wifi: Wifi {
                clientinfo: clients.clone(),
            },
        });

        // Finalize
        DemoRouter { states, i: 0 }
    }
}

#[async_trait(?Send)]
impl Router for DemoRouter {
    async fn logout(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn devices(&mut self) -> Result<LanUserTable, Box<dyn std::error::Error>> {
        let state = self.states[self.i].clone();
        self.i = (self.i + 1) % self.states.len();
        Ok(state)
    }

    async fn temperature(&mut self) -> Result<CmState, Box<dyn std::error::Error>> {
        Ok(CmState {
            tunner_temperature: 12,
            temperature: 34,
            oper_state: "foo".to_owned(),
            wan_ipv6_addr: WanIpv6Addr {
                addresses: vec!["2001:2345:6789:abcd::1".to_owned()],
            },
        })
    }
}
