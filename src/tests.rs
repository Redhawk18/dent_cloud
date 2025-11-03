use chrono::{Datelike, Local, Month, Timelike};
use dotenv::dotenv;
use std::env::var;

use super::*;
use crate::data::Parameters;

fn get_api_and_id() -> (String, String) {
    let _ = dotenv();

    (
        var("DENTCLOUD_API_KEY").expect("Missing 'DENTCLOUD_API_KEY' environment variable."),
        var("DENTCLOUD_KEY_ID").expect("Missing 'DENTCLOUD_KEY_ID' environment variable."),
    )
}

#[test]
fn has_required_keys() {
    get_api_and_id();
}

#[tokio::test]
async fn meter() {
    let (api, id) = get_api_and_id();

    let session = Session::new(api, id);
    let result = meters::meters(session).await;

    dbg!("{}", &result);
    assert!(result.is_ok())
}

#[tokio::test]
async fn topics() {
    let (api, id) = get_api_and_id();

    let session = Session::new(api, id);
    let result = topics::topics(session).await;

    dbg!("{}", &result);
    assert!(result.is_ok())
}

#[tokio::test]
async fn data() {
    let (api, id) = get_api_and_id();
    let meter = var("METER").expect("Missing 'METER' environment variable.");

    let session = Session::new(api, id);
    let time = Local::now();
    let params = Parameters {
        year: time
            .year()
            .try_into()
            .expect("infalliable until 32,000 CE."),
        // month: Month::try_from(time.month().try_into::<u8>().expect("infalliable"))

        // .expect("infalliable"),
        month: Month::try_from(TryInto::<u8>::try_into(time.month()).expect("infalliable"))
            .expect("infalliable"),
        day: Some(time.day().try_into().expect("infalliable")),
        hour: Some(time.hour().try_into().expect("infalliable")),
        topics: vec![
            "kVAHNet".to_owned(),
            "kWHNet".to_owned(),
            "DemandkW".to_owned(),
            "A".to_owned(),
            "dPF".to_owned(),
            "V".to_owned(),
        ],
        meter,
    };
    let result = data::data(session, params).await;

    dbg!("{}", &result);
    assert!(result.is_ok())
}
