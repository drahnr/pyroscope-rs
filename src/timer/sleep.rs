// Copyright 2021 Developers of Pyroscope.

// Licensed under the Apache License, Version 2.0 <LICENSE or
// https://www.apache.org/licenses/LICENSE-2.0>. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::utils::get_time_range;
use crate::Result;

use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use std::time::Duration;
use std::{thread, thread::JoinHandle};

/// A thread that sends a notification every 10th second
///
/// Timer will send an event to attached listeners (mpsc::Sender) every 10th
/// second (...10, ...20, ...)
///
/// The Timer thread will run continously until all Senders are dropped.
/// The Timer thread will be joined when all Senders are dropped.

#[derive(Debug, Default)]
pub struct Timer {
    /// A vector to store listeners (mpsc::Sender)
    txs: Arc<Mutex<Vec<Sender<u64>>>>,

    /// Thread handle
    pub handle: Option<JoinHandle<Result<()>>>,
}

impl Timer {
    /// Initialize Timer and run a thread to send events to attached listeners
    pub fn initialize(self) -> Result<Self> {
        let txs = Arc::clone(&self.txs);

        // Add Default tx
        let (tx, _rx): (Sender<u64>, Receiver<u64>) = channel();
        txs.lock()?.push(tx);

        // Spawn a Thread
        let handle = Some(thread::spawn(move || {
            // Get remaining time for 10th second fire event
            let rem = get_time_range(0)?.rem;

            // Sleep for rem seconds
            thread::sleep(Duration::from_secs(rem));

            loop {
                // Exit thread if there are no listeners
                if txs.lock()?.len() == 0 {
                    return Ok(());
                }

                // Get current time
                let current = get_time_range(0)?.from;

                // Iterate through Senders
                txs.lock()?.iter().for_each(|tx| {
                    // Send event to attached Sender
                    let _res = tx.send(current);
                });

                // Sleep for 10s
                thread::sleep(Duration::from_millis(10000));
            }
        }));

        Ok(Self { handle, ..self })
    }

    /// Attach an mpsc::Sender to Timer
    ///
    /// Timer will dispatch an event with the timestamp of the current instant,
    /// every 10th second to all attached senders
    pub fn attach_listener(&mut self, tx: Sender<u64>) -> Result<()> {
        // Push Sender to a Vector of Sender(s)
        let txs = Arc::clone(&self.txs);
        txs.lock()?.push(tx);

        Ok(())
    }

    /// Clear the listeners (txs) from Timer. This will shutdown the Timer thread
    pub fn drop_listeners(&mut self) -> Result<()> {
        let txs = Arc::clone(&self.txs);
        txs.lock()?.clear();

        Ok(())
    }
}
