use crate::WSClient;
use std::collections::HashMap;

use super::ws_client_internal::WSClientInternal;
use serde_json::{json, Value};

pub(super) const EXCHANGE_NAME: &str = "Binance";

const SPOT_WEBSOCKET_URL: &str = "wss://stream.binance.com:9443/stream";
const FUTURES_WEBSOCKET_URL: &str = "wss://fstream.binance.com/stream";
const DELIVERY_WEBSOCKET_URL: &str = "wss://dstream.binance.com/stream";

/// The WebSocket client for Binance Spot market(<https://binance-docs.github.io/apidocs/spot/en/>).
pub struct BinanceSpotWSClient<'a> {
    client: WSClientInternal<'a>,
}

/// The WebSocket client for Binance USDT Futures market(<https://binance-docs.github.io/apidocs/futures/en/>).
pub struct BinanceFuturesWSClient<'a> {
    client: WSClientInternal<'a>,
}

/// The WebSocket client for Binance Coin Dilivery market(<https://binance-docs.github.io/apidocs/delivery/en/>).
pub struct BinanceDeliveryWSClient<'a> {
    client: WSClientInternal<'a>,
}

fn serialize_command(channels: &[String], subscribe: bool) -> Vec<String> {
    let mut object = HashMap::<&str, Value>::new();

    object.insert(
        "method",
        serde_json::to_value(if subscribe {
            "SUBSCRIBE"
        } else {
            "UNSUBSCRIBE"
        })
        .unwrap(),
    );

    object.insert("params", json!(channels));
    object.insert("id", json!(9527));

    vec![serde_json::to_string(&object).unwrap()]
}

define_client!(
    BinanceSpotWSClient,
    EXCHANGE_NAME,
    SPOT_WEBSOCKET_URL,
    serialize_command
);
define_client!(
    BinanceFuturesWSClient,
    EXCHANGE_NAME,
    FUTURES_WEBSOCKET_URL,
    serialize_command
);
define_client!(
    BinanceDeliveryWSClient,
    EXCHANGE_NAME,
    DELIVERY_WEBSOCKET_URL,
    serialize_command
);