use std::{error::Error, net::AddrParseError, str::FromStr};

use crate::IP_REGEX;


#[derive(Debug, Default)]
pub struct DeviceDoc {
    pub ip:String,
}

#[derive(Debug,PartialEq,Eq)]
pub struct ParseIPError;

impl FromStr for DeviceDoc {
    type Err = ParseIPError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let matched = IP_REGEX.find(s);
        if let Some(ip) = matched {
            return Ok(DeviceDoc {ip: ip.as_str().to_string()})
        } else {
            return Err(ParseIPError);
        };
    }
}
impl DeviceDoc {

}