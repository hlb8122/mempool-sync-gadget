pub mod mempool;

use std::sync::{Arc, RwLock};

use mempool::Mempool;
use log::{info, error};
use futures_zmq::{prelude::*, Sub};
use futures::{Future, Stream};

fn main() {
    // Logging
    std::env::set_var("RUST_LOG", "INFO");
    pretty_env_logger::init_timed();

    info!("starting...");

    // Mempool
    let mempool_shared = Arc::new(RwLock::new(Mempool::default()));

    // ZeroMQ
    let context = Arc::new(zmq::Context::new());
    let sub = Sub::builder(context)
        .bind("tcp://127.0.0.1:29000")
        .filter(b"")
        .build();
    let runner = sub.and_then(|sub| {
        sub.stream()
            .for_each(|multipart| {
                for msg in &multipart {
                    if let Some(msg) = msg.as_str() {
                        info!("Found msg: {}", msg);
                    }
                }
                futures::future::ok(())
            })
    });

    tokio::run(runner.map(|_| ()).map_err(|e| {
        error!("{}", e);
    }));
}