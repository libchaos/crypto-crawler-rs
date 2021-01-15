use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use std::{collections::HashMap, time::Duration};

use crate::{msg::Message, MessageType};
use crypto_markets::{fetch_symbols, MarketType};
use crypto_rest_client::*;
use crypto_ws_client::*;
use log::*;
use serde_json::Value;

const EXCHANGE_NAME: &str = "huobi";

gen_check_args!(EXCHANGE_NAME);

fn extract_symbol(json: &str) -> String {
    let obj = serde_json::from_str::<HashMap<String, Value>>(&json).unwrap();
    let channel = obj.get("ch").unwrap().as_str().unwrap();
    let symbol = channel.split('.').nth(1).unwrap();
    symbol.to_string()
}

#[rustfmt::skip]
gen_crawl_event!(crawl_trade_spot, market_type, symbols, on_msg, duration, HuobiSpotWSClient, MessageType::Trade, subscribe_trade, true);
#[rustfmt::skip]
gen_crawl_event!(crawl_trade_future, market_type, symbols, on_msg, duration, HuobiFutureWSClient, MessageType::Trade, subscribe_trade, true);
#[rustfmt::skip]
gen_crawl_event!(crawl_trade_linear_swap, market_type, symbols, on_msg, duration, HuobiLinearSwapWSClient, MessageType::Trade, subscribe_trade, true);
#[rustfmt::skip]
gen_crawl_event!(crawl_trade_inverse_swap, market_type, symbols, on_msg, duration, HuobiInverseSwapWSClient, MessageType::Trade, subscribe_trade, true);
#[rustfmt::skip]
gen_crawl_event!(crawl_trade_option, market_type, symbols, on_msg, duration, HuobiOptionWSClient, MessageType::Trade, subscribe_trade, true);

#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_future, market_type, symbols, on_msg, duration, HuobiFutureWSClient, MessageType::L2Event, subscribe_orderbook, true);
#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_linear_swap, market_type, symbols, on_msg, duration, HuobiLinearSwapWSClient, MessageType::L2Event, subscribe_orderbook, true);
#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_inverse_swap, market_type, symbols, on_msg, duration, HuobiInverseSwapWSClient, MessageType::L2Event, subscribe_orderbook, true);
#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_option, market_type, symbols, on_msg, duration, HuobiOptionWSClient, MessageType::L2Event, subscribe_orderbook, true);

#[rustfmt::skip]
gen_crawl_snapshot!(crawl_l2_snapshot_spot, market_type, symbols, on_msg, MessageType::L2Snapshot, HuobiSpotRestClient::fetch_l2_snapshot);
#[rustfmt::skip]
gen_crawl_snapshot!(crawl_l2_snapshot_future, market_type, symbols, on_msg, MessageType::L2Snapshot, HuobiFutureRestClient::fetch_l2_snapshot);
#[rustfmt::skip]
gen_crawl_snapshot!(crawl_l2_snapshot_linear_swap, market_type, symbols, on_msg, MessageType::L2Snapshot, HuobiLinearSwapRestClient::fetch_l2_snapshot);
#[rustfmt::skip]
gen_crawl_snapshot!(crawl_l2_snapshot_inverse_swap, market_type, symbols, on_msg, MessageType::L2Snapshot, HuobiInverseSwapRestClient::fetch_l2_snapshot);
#[rustfmt::skip]
gen_crawl_snapshot!(crawl_l2_snapshot_option, market_type, symbols, on_msg, MessageType::L2Snapshot, HuobiOptionRestClient::fetch_l2_snapshot);

pub(crate) fn crawl_trade(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
    duration: Option<u64>,
) -> Option<std::thread::JoinHandle<()>> {
    match market_type {
        MarketType::Spot => crawl_trade_spot(market_type, symbols, on_msg, duration),
        MarketType::LinearFuture | MarketType::InverseFuture => {
            crawl_trade_future(market_type, symbols, on_msg, duration)
        }
        MarketType::LinearSwap => crawl_trade_linear_swap(market_type, symbols, on_msg, duration),
        MarketType::InverseSwap => crawl_trade_inverse_swap(market_type, symbols, on_msg, duration),
        MarketType::Option => crawl_trade_option(market_type, symbols, on_msg, duration),
        _ => panic!("Huobi does NOT have the {} market type", market_type),
    }
}

pub(crate) fn crawl_l2_event(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
    duration: Option<u64>,
) -> Option<std::thread::JoinHandle<()>> {
    match market_type {
        MarketType::Spot => {
            let on_msg_ext = |msg: String| {
                let message = Message::new(
                    EXCHANGE_NAME.to_string(),
                    market_type,
                    extract_symbol(&msg),
                    MessageType::L2Event,
                    msg.to_string(),
                );
                (on_msg.lock().unwrap())(message);
            };
            // Huobi Spot market.$symbol.mbp.$levels must use wss://api.huobi.pro/feed
            // or wss://api-aws.huobi.pro/feed
            let ws_client = HuobiSpotWSClient::new(
                Arc::new(Mutex::new(on_msg_ext)),
                Some("wss://api.huobi.pro/feed"),
            );
            ws_client.subscribe_orderbook(symbols.unwrap());
            ws_client.run(duration);
            None
        }
        MarketType::LinearFuture | MarketType::InverseFuture => {
            crawl_l2_event_future(market_type, symbols, on_msg, duration)
        }
        MarketType::LinearSwap => {
            crawl_l2_event_linear_swap(market_type, symbols, on_msg, duration)
        }
        MarketType::InverseSwap => {
            crawl_l2_event_inverse_swap(market_type, symbols, on_msg, duration)
        }
        MarketType::Option => crawl_l2_event_option(market_type, symbols, on_msg, duration),
        _ => panic!("Huobi does NOT have the {} market type", market_type),
    }
}

pub(crate) fn crawl_l2_snapshot(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
) {
    match market_type {
        MarketType::Spot => crawl_l2_snapshot_spot(market_type, symbols, on_msg),
        MarketType::LinearFuture | MarketType::InverseFuture => {
            crawl_l2_snapshot_future(market_type, symbols, on_msg)
        }
        MarketType::LinearSwap => crawl_l2_snapshot_linear_swap(market_type, symbols, on_msg),
        MarketType::InverseSwap => crawl_l2_snapshot_inverse_swap(market_type, symbols, on_msg),
        MarketType::Option => crawl_l2_snapshot_option(market_type, symbols, on_msg),
        _ => panic!("Huobi does NOT have the {} market type", market_type),
    }
}
