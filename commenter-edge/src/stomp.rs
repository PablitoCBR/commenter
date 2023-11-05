use anyhow::{bail, Ok, Result};
use std::collections::HashMap;
use std::str;
use warp::ws::Message;

use crate::comments::{Comment, CommentState};

const DESTINATION: &str = "destination";
const ACTION: &str = "action";
const ID: &str = "id";

#[derive(Clone)]
pub struct StompFrame {
    pub command: String,
    pub headers: HashMap<String, String>,
    pub text: String,
}

pub enum StompClientFrame {
    SEND(SendClientFrame),
    SUBSCRIBE(String),
    UNSUBSCRIBE(String),
    DISCONNECT,
}

pub enum SendClientFrame {
    CREATE { destination: String, text: String },
    UPDATE { id: String, text: String },
    DELETE { id: String },
}

impl StompFrame {
    pub fn new(comment: &Comment) -> StompFrame {
        let state: CommentState = num::FromPrimitive::from_i32(comment.state).unwrap();

        StompFrame {
            command: "MESSAGE".to_owned(),
            headers: HashMap::from([
                (DESTINATION.to_owned(), comment.group_id.clone()),
                (ID.to_owned(), comment.id.clone()),
                (ACTION.to_owned(), state.as_str_name().to_owned()),
            ]),
            text: comment.text.to_owned(),
        }
    }
}

impl Into<String> for StompFrame {
    fn into(self) -> String {
        let mut data = Vec::<String>::new();
        data.push(self.command.clone());
        data.push(String::from("\n"));

        for header in self.headers.iter() {
            data.push(header.0.clone());
            data.push(String::from(":"));
            data.push(header.1.clone());
            data.push(String::from("\n"));
        }

        data.push(String::from("\n"));
        data.push(self.text.clone());

        return data.join("");
    }
}

impl StompClientFrame {
    pub fn new(msg: Message) -> Result<StompClientFrame> {
        let raw_str = str::from_utf8(msg.as_bytes())?;

        let mut command: Option<String> = None;
        let mut headers = HashMap::<String, String>::new();
        let mut body: Option<Vec<u8>> = None;

        // COMMAND
        // HEADER
        //   .
        //   .
        // HEADER
        //
        // BODY..

        let mut last_new_line_index = 0;

        for (index, character) in raw_str.char_indices() {
            if character != '\n' {
                continue;
            }

            if command.is_none() {
                command = Some(raw_str[..index].to_string());
            } else if last_new_line_index != index - 1 {
                let header_splitted: Vec<String> = raw_str[(last_new_line_index + 1)..index]
                    .split(':')
                    .map(|el| el.to_string())
                    .collect();

                if header_splitted.len() != 2 {
                    bail!(
                        "Unable to parse header {}",
                        raw_str[(last_new_line_index + 1)..index].to_string()
                    );
                }

                headers.insert(header_splitted[0].clone(), header_splitted[1].clone());
            } else {
                body = Some(
                    raw_str[(last_new_line_index + 2)..]
                        .as_bytes()
                        .into_iter()
                        .map(|el| el.clone())
                        .collect(),
                );
            }

            last_new_line_index = index;
        }

        return match command {
            Some(cmd) => match cmd.as_str() {
                "SEND" => StompClientFrame::crate_send_frame(headers, body),
                "SUBSCRIBE" => StompClientFrame::create_subscribe_frame(headers),
                "UNSUBSCRIBE" => StompClientFrame::create_unsubscribe_frame(headers),
                "DISCONNECT" => Ok(StompClientFrame::DISCONNECT),
                _ => bail!("Unrecogized command received {}", cmd),
            },
            None => bail!("Unable to parse STOMP command... command undetected..."),
        };
    }

    fn crate_send_frame(
        headers: HashMap<String, String>,
        payload: Option<Vec<u8>>,
    ) -> Result<StompClientFrame> {
        if let Some(action) = headers.get(ACTION) {
            let send_frame = match action.as_str() {
                "CREATE" => StompClientFrame::create_send_create_frame(
                    headers,
                    String::from_utf8(payload.unwrap_or_default())?,
                ),
                "UPDATE" => StompClientFrame::create_send_update_frame(
                    headers,
                    String::from_utf8(payload.unwrap_or_default())?,
                ),
                "DELETE" => StompClientFrame::create_send_delete_frame(headers),
                _ => bail!("Urecognized action type"),
            }?;

            return Ok(StompClientFrame::SEND(send_frame));
        } else {
            bail!("Action header not specified");
        }
    }

    fn create_send_create_frame(
        headers: HashMap<String, String>,
        text: String,
    ) -> Result<SendClientFrame> {
        if let Some(destination) = headers.get(DESTINATION) {
            return Ok(SendClientFrame::CREATE {
                destination: destination.to_owned(),
                text,
            });
        } else {
            bail!("SEND frame with CREATE action requires DESTINATION to be specifed")
        }
    }

    fn create_send_update_frame(
        headers: HashMap<String, String>,
        text: String,
    ) -> Result<SendClientFrame> {
        if let Some(id) = headers.get(ID) {
            return Ok(SendClientFrame::UPDATE {
                id: id.to_owned(),
                text,
            });
        } else {
            bail!("SEND frame with UPDATE action requires ID to be specified")
        }
    }

    fn create_send_delete_frame(headers: HashMap<String, String>) -> Result<SendClientFrame> {
        if let Some(id) = headers.get(ID) {
            return Ok(SendClientFrame::DELETE { id: id.to_owned() });
        } else {
            bail!("SEND frame with DELETE action requires ID to be specifed")
        }
    }

    fn create_subscribe_frame(headers: HashMap<String, String>) -> Result<StompClientFrame> {
        if let Some(destination) = headers.get(DESTINATION) {
            return Ok(StompClientFrame::SUBSCRIBE(destination.to_owned()));
        } else {
            bail!("Destination header not found for SUBSCRIBE command");
        }
    }

    fn create_unsubscribe_frame(headers: HashMap<String, String>) -> Result<StompClientFrame> {
        if let Some(destination) = headers.get(DESTINATION) {
            return Ok(StompClientFrame::UNSUBSCRIBE(destination.to_owned()));
        } else {
            bail!("Id header not found for SUBSCRIBE command");
        }
    }
}
