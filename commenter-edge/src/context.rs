use anyhow::{bail, Result};
use prost::Message;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    RwLock,
};

use crate::{
    comments::Comment,
    stomp::{SendClientFrame, StompClientFrame, StompFrame},
};

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<StompFrame>>>>;

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub struct ApplicationContext {
    distribution_map: RwLock<HashMap<String, HashSet<usize>>>,
    producer: FutureProducer,
    consumer: StreamConsumer,
    users: Users,
}

impl ApplicationContext {
    const CONSUMER_GROUP_ID: &str = "commenter-edge";
    const TOPIC: &str = "comments";

    pub fn new(kafka_brokers: &str) -> ApplicationContext {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", kafka_brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Kafka producer created");

        let consumer = ClientConfig::new()
            .set("group.id", ApplicationContext::CONSUMER_GROUP_ID)
            .set("bootstrap.servers", kafka_brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()
            .expect("Kafka consumer created");

        return ApplicationContext {
            producer,
            consumer,
            users: Users::default(),
            distribution_map: RwLock::new(HashMap::new()),
        };
    }

    pub async fn add_user(&self, sender: UnboundedSender<StompFrame>) -> usize {
        let user_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
        self.users.write().await.insert(user_id, sender);
        return user_id;
    }

    pub async fn remove_user(&self, user_id: usize) {
        self.remove_user_from_distribution_map(user_id).await;
        self.users.write().await.remove(&user_id);
    }

    pub async fn handle_client_frame(&self, user_id: usize, frame: StompClientFrame) -> Result<()> {
        match frame {
            StompClientFrame::SEND(send_frme) => self.send(send_frme).await,
            StompClientFrame::SUBSCRIBE(destination) => self.subscribe(user_id, destination).await,
            StompClientFrame::UNSUBSCRIBE(destination) => {
                self.unsubscribe(user_id, destination).await
            }
            StompClientFrame::DISCONNECT => Ok(()),
        }
    }

    pub async fn listen_blocking(&self) {
        self.consumer
            .subscribe(&[ApplicationContext::TOPIC])
            .expect("Subscribed to topic");

        loop {
            match self.consumer.recv().await {
                Ok(msg) => {
                    if let Some(payload) = rdkafka::Message::payload(&msg) {
                        match Comment::decode(payload) {
                            Ok(comment) => {
                                let distibution_group_read_lock =
                                    self.distribution_map.read().await;

                                if let Some(distribution_group) =
                                    distibution_group_read_lock.get(&comment.group_id)
                                {
                                    let senders_read_lock = self.users.read().await;
                                    let stomp_frame = StompFrame::new(&comment);

                                    for recipient_id in distribution_group {
                                        if let Some(sender) = senders_read_lock.get(recipient_id) {
                                            let _ = sender.send(stomp_frame.clone());
                                        }
                                    }
                                }
                            }
                            Err(err) => panic!("{:?}", err),
                        }
                    }
                }
                Err(err) => panic!("{:?}", err),
            }
        }
    }

    async fn subscribe(&self, user_id: usize, group: String) -> Result<()> {
        if !self.users.read().await.contains_key(&user_id) {
            bail!("Unable to register to group an user that was not added to context");
        }

        self.distribution_map
            .write()
            .await
            .entry(group)
            .or_default()
            .insert(user_id);

        Ok(())
    }

    async fn unsubscribe(&self, user_id: usize, group: String) -> Result<()> {
        let mut distribution_map_write_lock = self.distribution_map.write().await;

        if let Some(distribution_group) = distribution_map_write_lock.get_mut(&group) {
            distribution_group.remove(&user_id);
        }

        Ok(())
    }

    async fn send(&self, frame: SendClientFrame) -> Result<()> {
        let comment = match frame {
            SendClientFrame::CREATE { destination, text } => Comment::new_create(destination, text),
            SendClientFrame::UPDATE { id, text } => Comment::new_update(id, text).await?,
            SendClientFrame::DELETE { id } => Comment::new_delete(id).await?,
        };

        let sending_result = self
            .producer
            .send(
                FutureRecord::to(ApplicationContext::TOPIC)
                    .payload(&comment.encode_to_vec())
                    .key(&comment.group_id),
                Duration::from_secs(0),
            )
            .await;

        if sending_result.is_err() {
            bail!(sending_result.err().unwrap().0);
        }

        Ok(())
    }

    async fn remove_user_from_distribution_map(&self, user_id: usize) {
        self.distribution_map
            .write()
            .await
            .values_mut()
            .for_each(|recipients| {
                recipients.remove(&user_id);
            });
    }
}
