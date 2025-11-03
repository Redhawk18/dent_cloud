use std::collections::{BTreeMap, HashMap};

use chrono::{Month, NaiveDateTime};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{Result, Session};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    pub year: u16,
    pub month: Month,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub topics: Vec<String>,
    pub meter: String,
}

// TODO make DataRaw and then manually remove both date and time from the vec
// then use the shared logic to have a vec of keys

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    /// Time, Date, and the additional keys passed in with their meter elements.
    pub headers: Vec<Key>,
    /// Every key in each instance of this type.
    pub topics: Vec<Topic>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Key {
    AmpChannels(String),
    KilowattHoursNetElement(String),
    DemandKilowattElements(String),
    DisplacementPowerFactorChannel(String),
    DisplacementPowerFactorElement(String),
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct DataRaw {
            pub headers: Vec<String>,
            pub topics: Vec<Topic>,
        }

        let blacklist = vec!["time".to_string(), "date".to_string()];

        let mut raw_data = DataRaw::deserialize(deserializer)?;
        raw_data.headers.retain(|item| !blacklist.contains(item));

        let mut headers =
            Vec::with_capacity(raw_data.headers.len().saturating_sub(blacklist.len()));

        for raw_key in raw_data.headers {
            let new_key = raw_key_to_key(raw_key).expect("Failed to convert raw key.");
            headers.push(new_key);
        }

        Ok(Data {
            headers,
            topics: raw_data.topics,
        })
    }
}

type Map = BTreeMap<String, f32>;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    /// The timezone is unknown from the API, assume the timezone of the devices real world
    /// location.
    pub time: NaiveDateTime,
    pub amps_channels: Map,
    pub kilowatt_hours_net_elements: Map,
    pub demand_kilowatt_elements: Map,
    pub displacement_power_factor: DisplacementPowerFactor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplacementPowerFactor {
    pub channels: Map,
    pub elements: Map,
}

impl<'de> Deserialize<'de> for Topic {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct FlatTopic {
            date: String,
            time: String,
            #[serde(flatten)]
            all_fields: HashMap<String, String>,
        }

        let flat = FlatTopic::deserialize(deserializer)?;
        dbg!("{}", &flat);

        let time = NaiveDateTime::parse_from_str(
            &format!("{} {}", flat.date, flat.time),
            "%Y-%m-%d %H:%M",
        )
        .unwrap();
        let mut amps_channels = Map::new();
        let mut kilowatt_hours_net_elements = Map::new();
        let mut demand_kilowatt_elements = Map::new();
        let mut dpf_channels = Map::new();
        let mut dpf_elements = Map::new();

        for (key, value) in flat.all_fields {
            if let Ok(float) = value.parse::<f32>() {
                let field = raw_key_to_key(key).unwrap();

                match field {
                    Key::AmpChannels(channel) => amps_channels.insert(channel, float),
                    Key::KilowattHoursNetElement(element) => {
                        kilowatt_hours_net_elements.insert(element, float)
                    }
                    Key::DemandKilowattElements(element) => {
                        demand_kilowatt_elements.insert(element, float)
                    }
                    Key::DisplacementPowerFactorChannel(channel) => {
                        dpf_channels.insert(channel, float)
                    }
                    Key::DisplacementPowerFactorElement(element) => {
                        dpf_elements.insert(element, float)
                    }
                };
            }
        }

        let displacement_power_factor = DisplacementPowerFactor {
            channels: dpf_channels,
            elements: dpf_elements,
        };

        Ok(Topic {
            time,
            amps_channels,
            kilowatt_hours_net_elements,
            demand_kilowatt_elements,
            displacement_power_factor,
        })
    }
}

pub async fn data(session: Session, parameters: Parameters) -> Result<Data> {
    let topics = format!("[ {} ]", parameters.topics.join(", "));
    let mut query_parameters = vec![
        ("request", "getData".to_string()),
        ("year", parameters.year.to_string()),
        ("month", parameters.month.number_from_month().to_string()),
        ("topics", topics),
        ("meter", parameters.meter),
    ];

    if let Some(day) = parameters.day {
        query_parameters.push(("day", day.to_string()));
    }
    if let Some(hour) = parameters.hour {
        query_parameters.push(("hour", hour.to_string()));
    }

    session.send(&query_parameters).await
}

fn raw_key_to_key(raw_key: String) -> Option<Key> {
    let divided = raw_key.split("/").collect::<Vec<&str>>();

    // dbg!("{}", &divided);

    let field = divided.get(0)?.to_string();
    let sub_field = divided.get(1)?.to_string();
    let field_key = divided.get(2)?.to_string();

    const AMPS: &str = "A";
    const KILOWATT_HOURS: &str = "kWHNet";
    const POWER_FACTOR: &str = "dPF";
    const DEMAND_KILOWATT: &str = "DemandkW";
    const CHANNEL: &str = "Ch";
    const ELEMENT: &str = "Elm";

    if field == AMPS && sub_field == CHANNEL {
        Some(Key::AmpChannels(field_key))
    } else if field == KILOWATT_HOURS && sub_field == ELEMENT {
        Some(Key::KilowattHoursNetElement(field_key))
    } else if field == POWER_FACTOR && sub_field == CHANNEL {
        Some(Key::DisplacementPowerFactorChannel(field_key))
    } else if field == POWER_FACTOR && sub_field == ELEMENT {
        Some(Key::DisplacementPowerFactorElement(field_key))
    } else if field == DEMAND_KILOWATT && sub_field == ELEMENT {
        Some(Key::DemandKilowattElements(field_key))
    } else {
        panic!("Unchecked block in `Topic`'s deserializer.")
    }
}
