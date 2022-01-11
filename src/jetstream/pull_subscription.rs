// Copyright 2020-2022 The NATS Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io;
use std::sync::Arc;

use crate::jetstream::{AckPolicy, ConsumerInfo, ConsumerOwnership, JetStream};
use crate::message::Message;

#[derive(Debug)]
pub(crate) struct Inner {
    /// Name of the stream associated with the subscription.
    pub(crate) stream: String,

    /// Name of the consumer associated with the subscription.
    pub(crate) consumer: String,

    /// Ack policy used in while processing messages.
    pub(crate) consumer_ack_policy: AckPolicy,

    /// Indicates if we own the consumer and are responsible for deleting it or not.
    pub(crate) consumer_ownership: ConsumerOwnership,

    /// Client associated with subscription.
    pub(crate) context: JetStream,
}

impl Drop for Inner {
    fn drop(&mut self) {
        // Delete the consumer, if we own it.
        if self.consumer_ownership == ConsumerOwnership::Yes {
            self.context
                .delete_consumer(&self.stream, &self.consumer)
                .ok();
        }
    }
}

/// A `PullSubscription` receives `Message`s published
/// to specific NATS `Subject`s.
#[derive(Clone, Debug)]
pub struct PullSubscription(pub(crate) Arc<Inner>);

impl PullSubscription {
    /// Creates a subscription.
    pub(crate) fn new(
        consumer_info: ConsumerInfo,
        consumer_ownership: ConsumerOwnership,
        context: JetStream,
    ) -> PullSubscription {
        PullSubscription(Arc::new(Inner {
            stream: consumer_info.stream_name,
            consumer: consumer_info.name,
            consumer_ack_policy: consumer_info.config.ack_policy,
            consumer_ownership,
            context,
        }))
    }

    /// Fetches a batch of messages
    pub fn fetch(batch: i64) -> io::Result<Vec<Message>> {
        Ok(Fetch {})
    }
}

/// Fetch iterator returned by
pub struct Fetch {}
