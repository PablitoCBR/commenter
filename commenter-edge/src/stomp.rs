use std::collections::HashMap;
use std::str;
use warp::ws::Message;
use anyhow::{Result, bail};

pub struct StompFrame {
    pub command: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>
}

impl StompFrame {
    pub fn new(msg: Message) -> Result<StompFrame> {
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
            }
            else if last_new_line_index != index - 1 {
                let header_splitted: Vec<String> = raw_str[(last_new_line_index + 1)..index]
                    .split(':')
                    .map(|el| el.to_string())
                    .collect();

                if header_splitted.len() != 2 {
                    bail!("Unable to parse header {}", raw_str[(last_new_line_index + 1)..index].to_string());
                }

                headers.insert(header_splitted[0].clone(), header_splitted[1].clone());
            } else {
                body = Some(raw_str[(last_new_line_index + 1)..].as_bytes().into_iter().map(|el| el.clone()).collect());
            }

            last_new_line_index = index;
        }

        if command.is_none() {
            bail!("Unable to parse STOMP command... command undetected...");
        }

        if body.is_none() {
            bail!("Unable to parse STOMP frame... body undetected...");
        }

        return Ok(StompFrame {
            command: command.unwrap(),
            body: body.unwrap(),
            headers
        });
    }

    // fn new<T>(command: String, headers: HashMap<String, String>, data: T ) -> StompFrame {

    // }
}

