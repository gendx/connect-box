use crate::router::Router;
use crate::types::{CmState, LanUserTable};
use async_trait::async_trait;
use futures::stream;
use futures::stream::{Stream, StreamExt};
use log::{debug, trace, warn};
use reqwest::{Client, Response};
use std::error::Error;
use std::io;
use std::marker::Unpin;
use std::net::Ipv4Addr;
use tokio::time;
use tokio::time::Throttle;

pub struct ConnectBox<'a> {
    client: Client,
    addr: Ipv4Addr,
    password: &'a str,
    token: String,
    throttle_duration: time::Duration,
}

#[async_trait(?Send)]
impl<'a> Router for ConnectBox<'a> {
    async fn logout(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set(ConnectBox::CMD_LOGOUT, vec![]).await?;
        Ok(())
    }

    async fn devices(&mut self) -> Result<LanUserTable, Box<dyn std::error::Error>> {
        let xml = self.get(ConnectBox::CMD_DEVICES).await?;
        trace!("XML: {}", xml);
        let result = serde_xml_rs::from_str(&xml)?;
        Ok(result)
    }

    async fn temperature(&mut self) -> Result<CmState, Box<dyn std::error::Error>> {
        let xml = self.get(ConnectBox::CMD_TEMPERATURE).await?;
        trace!("XML: {}", xml);
        let result = serde_xml_rs::from_str(&xml)?;
        Ok(result)
    }
}

impl<'a> ConnectBox<'a> {
    const CMD_LOGIN: usize = 15;
    const CMD_LOGOUT: usize = 16;
    const CMD_DEVICES: usize = 123;
    const CMD_TEMPERATURE: usize = 136;

    pub async fn new(
        addr: Ipv4Addr,
        password: &'a str,
        timeout_duration: time::Duration,
        throttle_duration: time::Duration,
    ) -> Result<ConnectBox<'a>, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0")
            .cookie_store(true)
            .timeout(timeout_duration)
            .build()?;

        let mut connect = ConnectBox {
            client,
            addr,
            password,
            token: String::new(),
            throttle_duration,
        };

        connect.reset().await?;
        Ok(connect)
    }

    async fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Resetting state...");
        self.index().await?;
        self.login().await?;

        Ok(())
    }

    async fn index(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Fetching index page...");
        let mut throttle = time::throttle(self.throttle_duration, stream::repeat(()));
        loop {
            let res = self.index_impl().await;
            if let Err(ref e) = res {
                if ConnectBox::should_retry(e, &mut throttle, self.throttle_duration).await {
                    continue;
                }
            }

            return res;
        }
    }

    async fn index_impl(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!("http://{}/common_page/login.html", self.addr))
            .send()
            .await?;
        self.update_token(&response)
    }

    async fn login(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Logging in...");
        let text = self
            .set(
                ConnectBox::CMD_LOGIN,
                vec![("Username", "NULL"), ("Password", self.password)],
            )
            .await?;

        if text == "idloginincorrect" {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Provided password is incorrect.",
            )));
        }

        if !text.starts_with("successful;") {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Unexpected login result! Received: {:?}", text),
            )));
        }

        Ok(())
    }

    async fn get(&mut self, function: usize) -> Result<String, Box<dyn std::error::Error>> {
        let mut throttle = time::throttle(self.throttle_duration, stream::repeat(()));
        loop {
            let res = self.get_impl(function).await;
            if let Err(ref e) = res {
                if ConnectBox::should_retry(e, &mut throttle, self.throttle_duration).await {
                    continue;
                }
            }

            let text = res?;
            if text.starts_with("<!doctype html>") {
                trace!("HTML: {}", text);
                warn!("Received an HTML page. Resetting...");
                self.reset().await?;
            } else {
                return Ok(text);
            }
        }
    }

    async fn get_impl(&mut self, function: usize) -> Result<String, Box<dyn std::error::Error>> {
        trace!("Making a GET request...");
        let response = self
            .client
            .post(&format!("http://{}/xml/getter.xml", self.addr))
            .form(&[
                ("token", self.token.as_str()),
                ("fun", function.to_string().as_str()),
            ])
            .send()
            .await?;
        self.update_token(&response)?;
        trace!("Waiting on response text...");
        let text = response.text().await?;
        trace!("Finished GET request!");
        Ok(text)
    }

    async fn set(
        &mut self,
        function: usize,
        params: Vec<(&str, &str)>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut throttle = time::throttle(self.throttle_duration, stream::repeat(()));
        loop {
            let res = self.set_impl(function, params.clone()).await;
            if let Err(ref e) = res {
                if ConnectBox::should_retry(e, &mut throttle, self.throttle_duration).await {
                    continue;
                }
            }

            return res;
        }
    }

    async fn set_impl(
        &mut self,
        function: usize,
        mut params: Vec<(&str, &str)>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        trace!("Making a SET request...");
        let function_str = function.to_string();
        let mut form = vec![
            ("token", self.token.as_str()),
            ("fun", function_str.as_str()),
        ];
        form.append(&mut params);

        let response = self
            .client
            .post(&format!("http://{}/xml/setter.xml", self.addr))
            .form(&form)
            .send()
            .await?;
        self.update_token(&response)?;
        trace!("Waiting on response text...");
        let text = response.text().await?;
        trace!("Finished SET request!");
        Ok(text)
    }

    fn update_token(&mut self, response: &Response) -> Result<(), Box<dyn std::error::Error>> {
        trace!("Updating token...");
        if response.status() != reqwest::StatusCode::OK {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Invalid response (status = {}).", response.status()),
            )));
        }
        let token = response
            .cookies()
            .find(|cookie| cookie.name() == "sessionToken")
            .map(|cookie| cookie.value().to_owned())
            .ok_or_else(|| {
                Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Couldn't find a sessionToken cookie in the response.",
                ))
            })?;
        debug!("Session token: {}", token);

        self.token = token;
        Ok(())
    }

    // TODO: don't retry indefinitely after interrupt.
    async fn should_retry(
        error: &Box<dyn std::error::Error>,
        throttle: &mut Throttle<impl Stream + Unpin>,
        throttle_duration: time::Duration,
    ) -> bool {
        if error
            .downcast_ref::<reqwest::Error>()
            .filter(|re| re.is_request() || re.is_body())
            .and_then(|re| re.source())
            .and_then(|source| source.downcast_ref::<hyper::Error>())
            // TODO: more hyper::Error reasons.
            .map_or(false, |he| {
                he.is_connect() || he.is_timeout() || he.is_incomplete_message()
            })
            || error
                .downcast_ref::<reqwest::Error>()
                .map_or(false, |re| re.is_timeout())
        {
            warn!(
                "Connect error: {:?}.\nRetrying in {:?}...",
                error, throttle_duration
            );
            throttle.next().await;
            true
        } else {
            false
        }
    }
}
