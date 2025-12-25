use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{Result, Session};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meters {
    pub success: bool,
    pub meters: Vec<Meter>,
}

pub type Meter = String;

/// Fetches a list of meters and a success field.
///
/// Example
/// ```json
/// {"success":true,"meters":["P482311252","P482102272","P482102270"]}
/// ```
pub async fn meters(session: &Session) -> Result<Meters> {
    trace!("Sending meter request",);
    session.send(&[("request", "getMeters")]).await
}
