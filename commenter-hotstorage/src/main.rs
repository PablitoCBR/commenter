use commenter_database::{comments::Comment, establish_connection};

use diesel::RunQueryDsl;
use prost::Message;

use std::time::Duration;

use rdkafka::{
    consumer::{BaseConsumer, CommitMode, Consumer},
    error::KafkaError,
    message::BorrowedMessage,
    ClientConfig,
};

use anyhow::{bail, Context, Result};

const CONSUMER_GROUP_ID: &str = "commenter-hotstorage";
const TOPIC: &str = "comments";

fn main() {
    let consumer: BaseConsumer = ClientConfig::new()
        .set("group.id", CONSUMER_GROUP_ID)
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .set("allow.auto.create.topics", "true")
        .create()
        .expect("Kafka consumer created");

    consumer.subscribe(&[TOPIC]).expect("Subscribed to topic");

    loop {
        if let Some(result) = consumer.poll(Duration::from_secs(1)) {
            if let Err(err) = wrap_result(&result)
                .and_then(handle_message)
                .and_then(|msg| commit(&consumer, msg))
            {
                panic!("I let it panic... for pleasure... because: {:?}", err);
            }
        }
    }
}

fn wrap_result<'a>(
    result: &'a Result<BorrowedMessage<'a>, KafkaError>,
) -> Result<&'a BorrowedMessage<'a>> {
    match result {
        Ok(msg) => Ok(msg),
        Err(err) => bail!(err.to_owned()),
    }
}

fn handle_message<'a>(message: &'a BorrowedMessage<'a>) -> Result<&'a BorrowedMessage<'a>> {
    let comment = Comment::decode(
        rdkafka::Message::payload(message).context("Message should have payload")?,
    )?;

    //TODO: Add connection pool from r2d2

    diesel::insert_into(commenter_database::schema::comments::dsl::comments)
        .values(comment)
        .execute(& mut establish_connection())?;

    Ok(message)
}

fn commit(consumer: &BaseConsumer, message: &BorrowedMessage<'_>) -> Result<()> {
    consumer.commit_message(message, CommitMode::Sync)?;
    Ok(())
}