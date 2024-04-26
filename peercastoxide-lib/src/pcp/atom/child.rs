use std::fmt::{Display, Formatter};
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::{anyhow, bail, Result};
use derive_new::new;
use regex::Regex;

use crate::pcp::atom::to_string_without_zero_padding;

use super::{well_known_identifiers::*, Identifier};

fn from_flg1_to_string(data: u8) -> String {
    let tracker = data & 1 << 0 != 0;
    let relay = data & 1 << 1 != 0;
    let direct = data & 1 << 2 != 0;
    let push = data & 1 << 3 != 0;
    let recv = data & 1 << 4 != 0;
    let cin = data & 1 << 5 != 0;
    let private = data & 1 << 6 != 0;
    let unused = data & 1 << 7 != 0;
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        if tracker { "TRACKER" } else { "tracker" },
        if relay { "RELAY" } else { "relay" },
        if direct { "DIRECT" } else { "direct" },
        if push { "PUSH" } else { "push" },
        if recv { "RECV" } else { "recv" },
        if cin { "CIN" } else { "cin" },
        if private { "PRIVATE" } else { "private" },
        if unused { "!" } else { "_" },
    )
}

#[derive(Debug, Eq, PartialEq, new)]
pub struct AtomChild {
    identifier: Identifier,
    data: Vec<u8>,
}

impl AtomChild {
    pub fn identifier(&self) -> &[u8; 4] {
        self.identifier.0.as_ref()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn to_data_string(&self) -> String {
        match self.identifier() {
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
            FLG1 if self.data().len() == 1 => from_flg1_to_string(self.data()[0]),
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
        write!(
            f,
            "{:?}({})",
            to_string_without_zero_padding(self.identifier()),
            self.to_data_string()
        )
    }
}
