//! This module provides a extremely hacky way of obtaining the latest release
//! number (e.g. 24.05) of Nixpkgs, by parsing the manual on nixos.org.

use anyhow::{anyhow, bail};
use log::info;
use scraper::Html;
use serde::Deserialize;

use crate::SoupFind;

/// Currently supported Nixpkgs channel version,
/// obtained from parsing the manual on nixos.org
#[derive(Deserialize, Debug, Clone)]
pub struct NixpkgsChannelVersion {
    #[serde(rename = "channel")]
    status: String,
    version: String,
}

impl NixpkgsChannelVersion {
    fn fetch() -> anyhow::Result<Vec<Self>> {
        info!("fetching the latest version of Nixpkgs from nixos.org");
        let html_string = reqwest::blocking::get("https://nixos.org/manual/nixpkgs/stable/")?
            .error_for_status()?
            .text()?;
        let html = Html::parse_document(&html_string);
        let channels_spec = html
            .find("body")
            .ok_or(anyhow!("fail to read <body> of the Nixpkgs manual"))?
            .attr("data-nixpkgs-channels")
            .ok_or(anyhow!(
                "failed to read current channels from the Nixpkgs manual"
            ))?;
        Ok(serde_json::from_str(channels_spec)?)
    }

    fn fetch_channel(spec: &str) -> anyhow::Result<String> {
        let channels = Self::fetch()?;
        for channel in channels.clone() {
            if channel.status == spec {
                return Ok(channel.version);
            }
        }
        bail!(
            "could not find '{spec}' from supported channels: {:?}",
            channels
        )
    }

    /// Fetches the current stable version of Nixpkgs,
    /// from the manual on nixos.org
    pub fn stable() -> anyhow::Result<String> {
        Self::fetch_channel("stable")
    }
}

#[test]
fn fetch_stable() -> anyhow::Result<()> {
    println!(
        "latest stable version: {}",
        NixpkgsChannelVersion::stable()?
    );
    Ok(())
}
