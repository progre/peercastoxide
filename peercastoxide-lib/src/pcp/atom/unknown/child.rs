use std::fmt::{Display, Formatter};
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::{anyhow, bail, Result};
use regex::Regex;

use crate::pcp::atom::{values::Flg1, well_known_identifiers::*};

use super::Identifier;

#[derive(Debug, Eq, PartialEq, derive_new::new, getset::Getters)]
pub struct AtomChild {
    #[get = "pub"]
    identifier: Identifier,
    data: Vec<u8>,
}

impl AtomChild {
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn to_data_string(&self) -> String {
        match self.identifier().0.as_ref() {
            PING | PORT | UPPT | VEXP | VEXN if self.data().len() == 2 => {
                self.to_u16().unwrap().to_string()
            }
            PCP | BITR | NEWP | NUML | NUMR | OK | OLDP | POS | QUIT | UPHP | UPPT | UPTM | VER
            | VERS | VEVP | VRVP
                if self.data().len() == 4 =>
            {
                self.to_u32().unwrap().to_string()
            }
            IP | RIP | UPIP if self.data().len() == 4 => self.to_ipv4().unwrap().to_string(),
            IP | RIP | UPIP if self.data().len() == 16 => self.to_ipv6().unwrap().to_string(),
            AGNT | ALBM | CMNT | CREA | DESC | GNRE | NAME | STYP | SEXT | TITL | TYPE | URL
                if !self.data().is_empty() =>
            {
                format!(
                    "{:?}{}",
                    Regex::new("\0$")
                        .unwrap()
                        .replace(&String::from_utf8_lossy(self.data()), ""),
                    if *self.data().last().unwrap() != 0 {
                        "(NUL missing)"
                    } else {
                        ""
                    }
                )
            }
            FLG1 if self.data().len() == 1 => format!("{:?}", Flg1(self.data()[0])),
            _ => self
                .data()
                .iter()
                .map(|&x| format!("{:02x}", x))
                .collect::<Vec<_>>()
                .join(""),
        }
    }

    pub fn to_u16(&self) -> Result<u16> {
        let mut num = [0u8; 2];
        (&mut num[0..2]).write_all(&self.data)?;
        Ok(u16::from_le_bytes(num))
    }

    pub fn to_u32(&self) -> Result<u32> {
        let mut num = [0u8; 4];
        (&mut num[0..4]).write_all(&self.data)?;
        Ok(u32::from_le_bytes(num))
    }

    pub fn to_ip(&self) -> Result<IpAddr> {
        match self.data.len() {
            4 => Ok(IpAddr::V4(self.to_ipv4().unwrap())),
            16 => Ok(IpAddr::V6(self.to_ipv6().unwrap())),
            _ => bail!("Invalid IP atom"),
        }
    }

    fn to_ipv4(&self) -> Result<Ipv4Addr> {
        let vec: Vec<_> = self.data().iter().cloned().rev().collect();
        let octets: [_; 4] = vec.try_into().map_err(|_| anyhow!("size mismatch"))?;
        Ok(Ipv4Addr::from(octets))
    }

    fn to_ipv6(&self) -> Result<Ipv6Addr> {
        let vec: Vec<_> = self.data().iter().cloned().rev().collect();
        let octets: [_; 16] = vec.try_into().map_err(|_| anyhow!("size mismatch"))?;
        Ok(Ipv6Addr::from(octets))
    }
}

impl Display for AtomChild {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let identifier = self.identifier().to_string();
        if identifier.chars().all(|x| !x.is_ascii_control()) {
            write!(f, "{}", identifier)?;
        } else {
            write!(f, "{:?}", identifier)?;
        };
        write!(f, "({})", self.to_data_string())
    }
}
