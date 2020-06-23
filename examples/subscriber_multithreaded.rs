// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use iceoryx_rs::sb::{SampleReceiverWaitState, SubscriptionState, Topic};
use iceoryx_rs::Runtime;

use std::error::Error;
use std::thread;
use std::time::Duration;

#[repr(C)]
struct CounterTopic {
    counter: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    Runtime::get_instance("/subscriber_multithreaded");

    let topic = Topic::<CounterTopic>::new("Radar", "FrontLeft", "Counter");

    const CACHE_SIZE: u32 = 5;
    let (subscriber, sample_receive_token) = topic.subscribe_mt(CACHE_SIZE);

    let mut has_printed_waiting_for_subscription = false;
    while subscriber.subscription_state() != SubscriptionState::Subscribed {
        if !has_printed_waiting_for_subscription {
            println!("waiting for subscription ...");
            has_printed_waiting_for_subscription = true;
        }
        thread::sleep(Duration::from_millis(10));
    }

    if has_printed_waiting_for_subscription {
        println!("  -> subscribed");
    }

    let sample_receiver = subscriber.get_sample_receiver(sample_receive_token);

    let th = thread::spawn(move || {
        loop {
            match sample_receiver.wait_for_samples(Duration::from_secs(2)) {
                SampleReceiverWaitState::SamplesAvailable => {
                    while let Some(sample) = sample_receiver.get_sample() {
                        println!("Receiving: {}", sample.counter);
                    }
                }
                SampleReceiverWaitState::Timeout => {
                    println!("Timeout while waiting for samples!");
                    break;
                }
                SampleReceiverWaitState::Stopped => break,
            }
        }

        sample_receiver
    });

    let sample_receiver = th.join().map_err(|_| "could not join threads")?;
    subscriber.unsubscribe(sample_receiver);

    Ok(())
}
