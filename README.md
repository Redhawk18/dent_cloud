# Tenor

A Rust crate for dent instruments' v1 dentcloud API. With rate limiting and verbose error handling. 

## Usage
```rs
use dent_cloud::{Session, meters};

async fn get_dent_meters(api_key: String, key_id: String) {
    let session = Session::new(api_key, key_id);
    let meters = meters(session).await.unwrap();

    dbg!("{}", meters);
    assert!(meters.success)
}
```

## Contributions
Welcomed.

