use commenter_database::{comments::Comment, establish_connection};

use diesel::{ExpressionMethods, RunQueryDsl};
use prost::{DecodeError, Message};
use thiserror::Error;

use std::time::Duration;

use rdkafka::{
    consumer::{BaseConsumer, CommitMode, Consumer},
    error::KafkaError,
    message::BorrowedMessage,
    ClientConfig,
};

use dotenv::dotenv;
use std::env;

const CONSUMER_GROUP_ID: &str = "commenter-hotstorage";
const TOPIC: &str = "comments";

fn main() {
    dotenv().ok();
    let broker_host = env::var("BROKER").expect("BROKER must be set");

    let consumer: BaseConsumer = ClientConfig::new()
        .set("group.id", CONSUMER_GROUP_ID)
        .set("bootstrap.servers", &broker_host)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "10000")
        .set("enable.auto.commit", "false")
        .set("allow.auto.create.topics", "true")
        .set("security.protocol", "PLAINTEXT")
        .create()
        .expect("Kafka consumer created");

    consumer.subscribe(&[TOPIC]).expect("Subscribed to topic");

    loop {
        if let Some(result) = consumer.poll(Duration::from_secs(1)) {
            if let Err(error) = handle_message(&consumer, result) {
                eprintln!("{:?}", error);
            }
        }
    }
}

fn handle_message(
    consumer: &BaseConsumer,
    message_result: Result<BorrowedMessage, KafkaError>,
) -> Result<(), HotStorageError> {
    let message = message_result?;
    let comment = get_message_payload(&message)?;

    let state = comment.state.clone();
    let text = comment.text.clone();

    //TODO: Add connection pool from r2d2

    diesel::insert_into(commenter_database::schema::comments::dsl::comments)
        .values(comment)
        .on_conflict(commenter_database::schema::comments::dsl::id)
        .do_update()
        .set((
            commenter_database::schema::comments::dsl::state.eq(state),
            commenter_database::schema::comments::dsl::text.eq(text),
        ))
        .execute(&mut establish_connection())?;

    consumer.commit_message(&message, CommitMode::Sync)?;
    Ok(())
}

fn get_message_payload(message: &BorrowedMessage) -> Result<Comment, HotStorageError> {
    match rdkafka::Message::payload(message) {
        Some(payload) => Ok(Comment::decode(payload)?),
        None => Err(HotStorageError::MissingPayload(rdkafka::Message::offset(
            message,
        ))),
    }
}

#[derive(Error, Debug)]
pub enum HotStorageError {
    #[error("Error on kafka interaction")]
    Kafka(#[from] rdkafka::error::KafkaError),

    #[error("Error on database interaction")]
    Database(#[from] diesel::result::Error),

    #[error("Error on message encoding")]
    Encoding(#[from] DecodeError),

    #[error("Received message is missing payload (offset: {0})")]
    MissingPayload(i64),
}
