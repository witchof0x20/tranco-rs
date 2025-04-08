// Copyright 2024 witchof0x20
//
// This file is part of tranco-rs.
//
// tranco-rs is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// tranco-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with tranco-rs. If not, see <https://www.gnu.org/licenses/>.

use serde::Deserialize;
use std::fmt;
use std::io::{self, BufRead, BufReader, Cursor};

const API_BASE: &str = "https://tranco-list.eu/api";

/// Client used to make Tranco API calls
pub struct Client {
    client: reqwest::Client,
}
impl Client {
    /// Constructor
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self::from_client(client)
    }
    /// Constructor from client
    pub fn from_client(client: reqwest::Client) -> Self {
        Self { client }
    }
    /// List ranks for a domain
    ///
    /// # Parameters
    /// * `domain` - domain for which to query ranks in the daily lists of (at least) the past 30 days
    pub async fn ranks(&self, domain: &str) -> Result<RanksResponse, reqwest::Error> {
        let url = format!("{API_BASE}/ranks/domain/{domain}");
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    /// List ranks for a domain
    ///
    /// # Parameters
    /// * `domain` - domain for which to query ranks in the daily lists of (at least) the past 30 days
    pub async fn list(&self, id: &str) -> Result<ListsResponse, reqwest::Error> {
        let url = format!("{API_BASE}/lists/id/{id}");
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    /// List ranks for a domain
    ///
    /// # Parameters
    /// * `domain` - domain for which to query ranks in the daily lists of (at least) the past 30 days
    pub async fn list_date(
        &self,
        year: u16,
        month: u8,
        day: u8,
        subdomains: Option<bool>,
    ) -> Result<ListsResponse, reqwest::Error> {
        let url = format!(
            "{API_BASE}/lists/date/{year:04}{month:02}{day:02}{}",
            if let Some(subdomains) = subdomains {
                format!("?subdomains={subdomains}")
            } else {
                String::new()
            }
        );
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    /// Download a list
    ///
    /// # Parameters
    /// * `list` - ListsResponse from either `list` or `list_date`
    pub async fn download_list(
        &self,
        response: &ListsResponse,
    ) -> Result<Vec<RankedDomain>, DownloadListError> {
        let csv_body = self
            .client
            .get(response.download.clone())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await
            .map(Cursor::new)
            .map(BufReader::new)?;
        csv_body
            .lines()
            .map(|line| {
                let line = line?;
                let mut toks = line.split(",");
                let rank = toks.next().ok_or(DownloadListError::MissingRank)?.parse()?;
                let domain = toks.next().ok_or(DownloadListError::MissingDomain)?.into();
                Ok(RankedDomain { rank, domain })
            })
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadListError {
    #[error("Error making request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Error reading line from csv: {0}")]
    ReadLine(#[from] io::Error),
    #[error("CSV is missing rank")]
    MissingRank,
    #[error("CSV had invalid rank: {0}")]
    InvalidRank(#[from] std::num::ParseIntError),
    #[error("CSV is missing domain")]
    MissingDomain,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct RanksResponse {
    pub ranks: Vec<DomainRank>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct DomainRank {
    pub date: String,
    pub rank: u64,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ListsResponse {
    list_id: String,
    available: bool,
    download: String,
    created_on: String,
    configuration: Configuration,
    failed: bool,
    jobs_ahead: Option<i64>,
}

/// Represents a configuration for domain aggregation and filtering
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    /// List of data providers to use
    pub providers: Vec<Provider>,
    /// Start date for data collection (format: YYYY-MM-DD)
    pub start_date: String,
    /// End date for data collection (format: YYYY-MM-DD)
    pub end_date: String,
    /// Method used to combine rankings from different providers
    pub combination_method: CombinationMethod,
    /// Limit aggregation to domains from list prefixes of specified length
    pub list_prefix: ListPrefix,
    /// Whether to retain only pay-level domains
    #[serde(rename = "filterPLD")]
    pub filter_pld: ToggleOption,
    /// Whether to only include domains present for a minimum number of days
    #[serde(default)]
    pub inclusion_days: ToggleOption,
    /// Minimum number of days domains must be present
    #[serde(default)]
    pub inclusion_days_value: Option<u32>,
    /// Whether to only include domains present in a minimum number of lists
    #[serde(default)]
    pub inclusion_lists: ToggleOption,
    /// Minimum number of lists domains must be present in
    #[serde(default)]
    pub inclusion_lists_value: Option<u32>,
    /// TLD filtering mode
    #[serde(default)]
    #[serde(rename = "filterTLD")]
    pub filter_tld: Option<FilterTldOption>,
    /// TLDs to retain if filter_tld is Include
    #[serde(default)]
    #[serde(rename = "filterTLDValue")]
    pub filter_tld_value: Option<Vec<String>>,
    /// Whether to retain only one domain per organization
    #[serde(default)]
    pub filter_organization: ToggleOption,
    /// Whether to retain only specific subdomains
    #[serde(default)]
    pub filter_subdomain: ToggleOption,
    /// Subdomains to retain if filter_subdomain is On
    #[serde(default)]
    pub filter_subdomain_value: Option<Vec<String>>,
    /// Whether to filter out Google Safe Browsing domains
    #[serde(default)]
    pub filter_safe_browsing: ToggleOption,
    /// Whether to filter on Chrome User Experience Report domains
    #[serde(default)]
    #[serde(rename = "filterCRUX")]
    pub filter_crux: ToggleOption,
    /// Month of CrUX data, or latest available month
    #[serde(default)]
    #[serde(rename = "filterCRUXMonth")]
    pub filter_crux_month: Option<CruxMonth>,
    /// Type of selected CrUX dataset
    #[serde(default)]
    #[serde(rename = "filterCRUXType")]
    pub filter_crux_type: Option<CruxType>,
    /// Value for selected CrUX dataset (except "global")
    #[serde(default)]
    #[serde(rename = "filterCRUXValue")]
    pub filter_crux_value: Option<Vec<String>>,
}

/// Supported data providers
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Crux,
    Majestic,
    Radar,
    Umbrella,
    Alexa,
    Quantcast,
    Farsight,
}

/// Methods for combining rankings from different providers
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CombinationMethod {
    Dowdall,
    Borda,
}

/// Options for list_prefix field
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ListPrefix {
    Full,
    Length(u32),
}

/// Toggle options (on/off)
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ToggleOption {
    On,
    Off,
}

/// Default implementation for ToggleOption
impl Default for ToggleOption {
    fn default() -> Self {
        ToggleOption::Off
    }
}

/// Filter TLD options
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FilterTldOption {
    Include,
    False,
}

/// Month specification for CrUX data
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CruxMonth {
    Latest,
    Specific(String), // Format: YYYYMM
}

/// Type of CrUX dataset to filter on
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CruxType {
    Global,
    Country,
    Region,
    Subregion,
}

// Custom implementation for deserialization of ListPrefix
impl<'de> Deserialize<'de> for ListPrefix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ListPrefixVisitor;

        impl<'de> serde::de::Visitor<'de> for ListPrefixVisitor {
            type Value = ListPrefix;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer or the string \"full\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<ListPrefix, E>
            where
                E: serde::de::Error,
            {
                if value == "full" {
                    Ok(ListPrefix::Full)
                } else {
                    value.parse().map(ListPrefix::Length).map_err(E::custom)
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<ListPrefix, E>
            where
                E: serde::de::Error,
            {
                Ok(ListPrefix::Length(value as u32))
            }
        }

        deserializer.deserialize_any(ListPrefixVisitor)
    }
}

// Custom implementation for deserialization of CruxMonth
impl<'de> Deserialize<'de> for CruxMonth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CruxMonthVisitor;

        impl<'de> serde::de::Visitor<'de> for CruxMonthVisitor {
            type Value = CruxMonth;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string in format YYYYMM or \"latest\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<CruxMonth, E>
            where
                E: serde::de::Error,
            {
                if value == "latest" {
                    Ok(CruxMonth::Latest)
                } else {
                    // Validate YYYYMM format
                    if value.len() == 6 && value.chars().all(|c| c.is_digit(10)) {
                        Ok(CruxMonth::Specific(value.to_string()))
                    } else {
                        Err(E::custom(format!(
                            "Expected YYYYMM format or \"latest\", got {}",
                            value
                        )))
                    }
                }
            }
        }

        deserializer.deserialize_str(CruxMonthVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankedDomain {
    pub rank: u64,
    pub domain: String,
}
