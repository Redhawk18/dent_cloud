use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{Result, Session};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Topics {
    pub success: bool,
    pub topics: Vec<Topic>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    /// Shorthand unit.
    pub unit: String,
    /// Used by [`crate::data::data`] as a `topic`.
    pub request_key: String,
    /// Full name.
    pub description: String,
}

/// Fetches a list of units with keys and descriptions.
///
/// Example
/// ```json
/// {
///    "success": true,
///    "topics": [
///        {
///            "unit": "kVAh",
///            "requestKey": "kVAHNet",
///            "description": "NetKilovolt Ampere Hours."
///        },
///        {
///            "unit": "kWh",
///            "requestKey": "kWHNet",
///            "description": "Power.Net Kilowatt Hours."
///        },
///        {
///            "unit": "kW",
///            "requestKey": "DemandkW",
///            "description": "Power. Demand Kilowatts."
///        },
///        {
///            "unit": "A",
///            "requestKey": "A",
///            "description": "Current. Amperes."
///        },
///        {
///            "unit": "dPF",
///            "requestKey": "dPF",
///            "description": "Displacement Power Factor. Power usage Efficiency."
///        },
///        {
///            "unit": "V",
///            "requestKey": "V",
///            "description": "Voltage."
///        }
///    ]
/// }
/// ```
pub async fn topics(session: &Session) -> Result<Topics> {
    trace!("Sending topics request",);
    session.send(&[("request", "getTopics")]).await
}
