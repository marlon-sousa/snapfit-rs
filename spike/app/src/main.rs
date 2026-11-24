mod db;
mod impls;
mod model;
mod shop;

use db::{Database, Store};
use model::Order;

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    use std::task::{Context, Poll, Waker};
    let mut future = std::pin::pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(v) = future.as_mut().poll(&mut cx) { return v; }
    }
}

fn main() {
    let shop = shop::Shop::new();
    let mut db = shop.get_db();
    println!("name: {}", db.name());
    println!("find: {:?}", db.find(7).map(|o| o.id));
    println!("rows: {:?}", db.rows());
    println!("tagged: {}", db.tagged(42));
    println!("default: {}", db.with_default());
    println!("ping: {}", block_on(db.ping()));
    db.store(Order { id: 9 });
    println!("store: {:?}", db.rows());
    println!("put: {}", db.put(3u8));
    println!("orders: {}", shop.get_orders().describe());
    println!("reports: {}", shop.get_reports().db.name());
    println!("into_name: {}", db.into_name());
}
