use std::{future, pin::Pin, thread};

use reqwest::{
    header::{HeaderMap, HeaderValue}, Error, Method, Response
};

#[derive(Debug)]
pub struct GetServiceCapabilities {
    ip: String,
    headers: HeaderMap,
    body: String,
    username: Option<String>,
    password: Option<String>,
}
impl Default for GetServiceCapabilities {
    fn default() -> Self {
        let mut r = GetServiceCapabilities {
            body: String::default(),
            headers: HeaderMap::default(),
            ip: String::default(),
            username: None,
            password: None,
        };
        r.headers.append(
            "content-type",
            HeaderValue::from_str("text/xml;charset=utf-8").unwrap(),
        );
        r.headers
            .append("User-Agent", HeaderValue::from_str("ONVIF-CLI").unwrap());
        r.headers
            .append("Accept", HeaderValue::from_str("*/*").unwrap());
        r.headers.append(
            "SOAPAction",
            HeaderValue::from_str("http://www.onvif.org/ver10/media/wsdl/GetServiceCapabilities")
                .unwrap(),
        );

        r
    }
}
impl GetServiceCapabilities {
    fn ip(mut self, ip: String) -> Self {
        self.ip = ip;
        self
    }
    async fn run(self) -> impl Future<Output = Result<Response, Error>> {
        let client = reqwest::Client::default();
        let mut builder = client
            .request(Method::POST, self.ip)
            .body(self.body);
        if let Some(user) = self.username {
            builder = builder.basic_auth(
                user,
                if let Some(pass) = self.password {
                    Some(pass)
                } else {
                    None
                },
            );
        }
        
        client.execute(builder.build().unwrap()).into_future()
    }
}
