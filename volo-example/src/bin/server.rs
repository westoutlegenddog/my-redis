#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use volo_example::LogLayer;
//use std::collections::HashMap;
use volo_example::{S};

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::volo::example::ItemServiceServer::new(S::new())
    .layer_front(LogLayer)
    .run(addr)
    .await
    .unwrap();
}
