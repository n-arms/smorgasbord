#![allow(dead_code)]

use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use network_tables::{
    rmpv::ValueRef,
    v4::{Client, Config, PublishedTopic, Subscription, SubscriptionOptions, Type},
};
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use anyhow::Result;
use tracing::{event, Level};

use super::nt::from_nt_path;
use super::{Entry, Path, StatusUpdate};

pub struct Worker {
    read_sender: UnboundedSender<Entry>,
    write_receiver: UnboundedReceiver<Entry>,
    client: SubscribedClient,
}

pub struct SubscribedClient {
    client: Client,
    subscription: Subscription,
    published_topics: HashMap<Path, PublishedTopic>,
    status_sender: UnboundedSender<StatusUpdate>,
}

impl SubscribedClient {
    async fn write(&mut self, entry: Entry) -> Result<()> {
        let topic = if let Some(topic) = self.published_topics.get(&entry.path) {
            topic
        } else {
            let topic = self
                .client
                .publish_topic(
                    entry.path.to_string(),
                    value_type(entry.value.as_ref()),
                    None,
                )
                .await?;
            self.published_topics
                .entry(entry.path.clone())
                .or_insert(topic)
        };
        event!(
            Level::INFO,
            "Writing entry {:?} to topic {:?}",
            entry,
            topic
        );
        self.client
            .publish_value(topic, &entry.value)
            .await
            .map_err(Into::into)
    }
    async fn read(&mut self) -> Result<Entry> {
        loop {
            if let Some(message) = self.subscription.next().await {
                return Ok(Entry {
                    path: from_nt_path(message.topic_name).map_err(Into::<anyhow::Error>::into)?,
                    value: message.data,
                });
            }
            *self = Self::new(self.status_sender.clone()).await?;
        }
    }

    async fn new(status_sender: UnboundedSender<StatusUpdate>) -> Result<Self> {
        status_sender.send(StatusUpdate::IsConnectedChange(false))?;
        let client = connect_to_client().await;
        let subscription = subscribe(&client).await?;
        status_sender.send(StatusUpdate::IsConnectedChange(true))?;
        Ok(Self {
            client,
            subscription,
            published_topics: HashMap::new(),
            status_sender,
        })
    }
}

impl Worker {
    pub async fn new(
        read_sender: UnboundedSender<Entry>,
        write_receiver: UnboundedReceiver<Entry>,
        status_sender: UnboundedSender<StatusUpdate>,
    ) -> Self {
        let client;
        loop {
            let maybe_client = SubscribedClient::new(status_sender.clone()).await;
            match maybe_client {
                Ok(c) => {
                    client = c;
                    break;
                }
                Err(error) => {
                    event!(Level::ERROR, "subscribed client error {error:?}");
                }
            }
        }
        Self {
            read_sender,
            write_receiver,
            client,
        }
    }

    pub async fn run(mut self) {
        loop {
            select! {
                to_write = self.write_receiver.recv() => {
                    if let Some(entry) = to_write {
                        if let Err(error) = self.client.write(entry).await {
                            event!(Level::ERROR, "pipe error while sending write to subscribed client: {error:?}");
                        }
                    }
                },
                to_read = self.client.read() => {
                    if let Ok(entry) = to_read {

                        if let Err(error) = self.read_sender.send(entry) {
                            event!(Level::ERROR, "pipe error while sending entry to main nt thread: {error:?}");
                        }
                    }
                }
            }
        }
    }
}

async fn connect_to_client() -> Client {
    loop {
        let maybe_client = network_tables::v4::Client::try_new_w_config(
            SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5810),
            Config::default(),
        )
        .await;
        if let Ok(c) = maybe_client {
            return c;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

async fn subscribe(client: &Client) -> Result<Subscription> {
    client
        .subscribe_w_options(
            &["/SmartDashboard"],
            Some(SubscriptionOptions {
                all: Some(true),
                prefix: Some(true),
                ..Default::default()
            }),
        )
        .await
        .map_err(Into::into)
}

fn value_type(value: ValueRef<'_>) -> Type {
    match value {
        ValueRef::Boolean(_) => Type::Boolean,
        ValueRef::Integer(_) | ValueRef::Nil => Type::Int,
        ValueRef::F32(_) => Type::Float,
        ValueRef::F64(_) => Type::Double,
        ValueRef::String(_) => Type::String,
        ValueRef::Binary(_) => Type::Raw,
        ValueRef::Array(array) => {
            let inner_type = array.first().cloned().map_or(Type::Int, value_type);
            match inner_type {
                Type::Boolean => Type::BooleanArray,
                Type::Double => Type::DoubleArray,
                Type::Int => Type::IntArray,
                Type::Float => Type::FloatArray,
                Type::String => Type::StringArray,
                _ => todo!(),
            }
        }
        ValueRef::Map(_) => {
            todo!()
        }
        ValueRef::Ext(_, _) => {
            todo!()
        }
    }
}

pub async fn run_worker(
    read_sender: UnboundedSender<Entry>,
    write_receiver: UnboundedReceiver<Entry>,
    status_sender: UnboundedSender<StatusUpdate>,
) -> Result<()> {
    let worker = Worker::new(read_sender, write_receiver, status_sender).await;
    worker.run().await;
    Ok(())
}
