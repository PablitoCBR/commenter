use anyhow::{bail, Ok, Result};
use std::collections::HashMap;
use std::str;
use warp::ws::Message;

// use crate::comments::{Comment, CommentState};

const DESTINATION: &str = "destination";
const ACTION: &str = "action";
const ID: &str = "id";

#[derive(Clone)]
pub struct StompFrame {
    pub command: String,
    pub headers: HashMap<String, String>,
    pub text: String,
}

#[derive(PartialEq, Debug)]
pub enum StompClientFrame {
    SEND(SendClientFrame),
    SUBSCRIBE { destination: String, id: String },
    UNSUBSCRIBE(String),
    DISCONNECT,
}

#[derive(PartialEq, Debug)]
pub enum SendClientFrame {
    CREATE { destination: String, text: String },
    UPDATE { id: String, text: String },
    DELETE { id: String },
}

// impl StompFrame {
//     pub fn new(comment: &Comment) -> StompFrame {
//         let state: CommentState = num::FromPrimitive::from_i32(comment.state).unwrap();

//         StompFrame {
//             command: "MESSAGE".to_owned(),
//             headers: HashMap::from([
//                 (DESTINATION.to_owned(), comment.group_id.clone()),
//                 (ID.to_owned(), comment.id.clone()),
//                 (ACTION.to_owned(), state.as_str_name().to_owned()),
//             ]),
//             text: comment.text.to_owned(),
//         }
//     }
// }

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
    #[inline]
    pub fn new(msg: &Message) -> Result<StompClientFrame> {
        let raw_str = str::from_utf8(msg.as_bytes())?;

        let mut command: Option<String> = None;
        let mut headers = HashMap::<String, String>::new();
        let mut body: Option<Vec<u8>> = None;

        // COMMAND[\r]\n
        // HEADER\n
        //   .
        //   .
        // HEADER\n
        // \n
        // BODY..
        // \0

        let mut last_new_line_index = 0;

        for (index, character) in raw_str.char_indices() {
            if character != '\n' {
                continue;
            }

            // we can use index as we know that \n is taking only single byte to encode in UTF-8

            let fixed_index = if index > 1 && raw_str.as_bytes()[index - 1] as char == '\r' {
                index - 1
            } else {
                index
            };

            let new_line_shift = if fixed_index == index { 1 } else { 2 }; //TODO: this has to be better handled

            if command.is_none() {
                command = Some(raw_str[..fixed_index].to_owned());
            } else if last_new_line_index != index - new_line_shift {
                if (last_new_line_index + 1) != fixed_index {
                    let header_splitted: Vec<String> = raw_str[(last_new_line_index + 1)..fixed_index]
                        .split(':')
                        .map(|el| el.to_owned())
                        .collect();
    
                    if header_splitted.len() != 2 {
                        bail!(
                            "Unable to parse header {}",
                            raw_str[(last_new_line_index + 1)..fixed_index].to_owned()
                        );
                    }
    
                    headers.insert(header_splitted[0].clone(), header_splitted[1].clone());
                }
            } else {
                body = Some(
                    raw_str[(fixed_index + new_line_shift)..raw_str.len() - 1] //TODO: -1 it temp, we should handle NULL terminator \0
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
            if let Some(id) = headers.get(ID) {
                return Ok(StompClientFrame::SUBSCRIBE {
                    destination: destination.to_owned(),
                    id: id.to_owned(),
                });
            } else {
                bail!("Id header not found for SUBSCRIBE command");
            }
        } else {
            bail!("Destination header not found for SUBSCRIBE command");
        }
    }

    fn create_unsubscribe_frame(headers: HashMap<String, String>) -> Result<StompClientFrame> {
        if let Some(id) = headers.get(ID) {
            return Ok(StompClientFrame::UNSUBSCRIBE(id.to_owned()));
        } else {
            bail!("Id header not found for SUBSCRIBE command");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use warp::filters::ws::Message;

    mod disconnect {
        use super::*;

        #[test]
        fn stomp_client_frame_should_parse_disconnection_message_with_new_line_as_eol() {
            test_stomp_client_frame_disconnection_message_parsing(false);
        }

        #[test]
        fn stomp_client_frame_should_parse_disconnection_message_with_carriage_return_included_in_eol(
        ) {
            test_stomp_client_frame_disconnection_message_parsing(true)
        }

        fn test_stomp_client_frame_disconnection_message_parsing(optional_carraige_return: bool) {
            let input = encode_stomp_frame_command_only("DISCONNECT", optional_carraige_return);
            test_stomp_client_frame_parsing(input, StompClientFrame::DISCONNECT);
        }
    }

    mod subscribe {
        use super::*;

        #[test]
        fn stomp_client_frame_should_parse_subscribe_message_with_new_line_as_eol() {
            test_stomp_client_frame_subscribe_message_parsing("topic1", "sub-1", false)
        }

        #[test]
        fn stomp_client_frame_should_parse_subscribe_message_with_carriage_return_included_in_eol()
        {
            test_stomp_client_frame_subscribe_message_parsing("topic1", "sub-1", true)
        }

        fn test_stomp_client_frame_subscribe_message_parsing(
            destination: &str,
            id: &str,
            optional_carraige_return: bool,
        ) {
            let headers = HashMap::from([(DESTINATION, destination), (ID, id)]);
            let input = encode_stomp_frame_command_with_headers(
                "SUBSCRIBE",
                headers,
                optional_carraige_return,
            );

            test_stomp_client_frame_parsing(
                input,
                StompClientFrame::SUBSCRIBE {
                    destination: destination.to_owned(),
                    id: id.to_owned(),
                },
            );
        }
    }

    mod unsubscribe {
        use super::*;

        #[test]
        fn stomp_client_frame_should_parse_unsubscribe_message_with_new_line_as_eol() {
            test_stomp_client_frame_unsubscribe_message_parsing("sub-1", false)
        }

        #[test]
        fn stomp_client_frame_should_parse_unsubscribe_message_with_carriage_return_included_in_eol(
        ) {
            test_stomp_client_frame_unsubscribe_message_parsing("sub-1", true)
        }

        fn test_stomp_client_frame_unsubscribe_message_parsing(
            id: &str,
            optional_carraige_return: bool,
        ) {
            let input = encode_stomp_frame_command_with_headers(
                "UNSUBSCRIBE",
                HashMap::from([(ID, id)]),
                optional_carraige_return,
            );

            test_stomp_client_frame_parsing(input, StompClientFrame::UNSUBSCRIBE(id.to_owned()))
        }
    }

    mod send {
        use super::*;
        use std::collections::HashMap;

        mod create {
            use super::*;

            #[test]
            fn stomp_client_frame_should_parse_send_create_message_with_new_line_as_eol() {
                test_stomp_client_frame_send_create_message_parsing(
                    "destination_1",
                    "test body",
                    false,
                )
            }

            #[test]
            fn stomp_client_frame_should_parse_send_create_message_with_carriage_return_included_in_eol(
            ) {
                test_stomp_client_frame_send_create_message_parsing(
                    "destination_2",
                    "test body 2",
                    true,
                )
            }

            fn test_stomp_client_frame_send_create_message_parsing(
                destination: &str,
                body: &str,
                optional_carraige_return: bool,
            ) {
                let input =
                    encode_send_create_stomp_frame(destination, body, optional_carraige_return);
                test_stomp_client_frame_send_parsing(
                    input,
                    SendClientFrame::CREATE {
                        destination: destination.to_owned(),
                        text: body.to_owned(),
                    },
                )
            }

            fn encode_send_create_stomp_frame(
                destination: &str,
                body: &str,
                optional_carraige_return: bool,
            ) -> String {
                let headers = HashMap::from([(DESTINATION, destination)]);
                encode_send_stop_frame("CREATE", headers, body, optional_carraige_return)
            }
        }

        mod update {
            use super::*;

            #[test]
            fn stomp_client_frame_should_parse_send_update_message_with_new_line_as_eol() {
                test_stomp_client_frame_send_update_message_parsing("101", "test body", false)
            }

            #[test]
            fn stomp_client_frame_should_parse_send_update_message_with_carriage_return_included_in_eol(
            ) {
                test_stomp_client_frame_send_update_message_parsing("303", "test body 2", true)
            }

            fn test_stomp_client_frame_send_update_message_parsing(
                id: &str,
                body: &str,
                optional_carraige_return: bool,
            ) {
                let input = encode_send_update_stomp_frame(id, body, optional_carraige_return);
                test_stomp_client_frame_send_parsing(
                    input,
                    SendClientFrame::UPDATE {
                        id: id.to_owned(),
                        text: body.to_owned(),
                    },
                )
            }

            fn encode_send_update_stomp_frame(
                id: &str,
                body: &str,
                optional_carraige_return: bool,
            ) -> String {
                let headers = HashMap::from([(ID, id)]);
                encode_send_stop_frame("UPDATE", headers, body, optional_carraige_return)
            }
        }

        mod delete {
            use super::*;

            #[test]
            fn stomp_client_frame_should_parse_send_delete_message_with_new_line_as_eol() {
                test_stomp_client_frame_send_delete_message_parsing("101", false)
            }

            #[test]
            fn stomp_client_frame_should_parse_send_delete_message_with_carriage_return_included_in_eol(
            ) {
                test_stomp_client_frame_send_delete_message_parsing("303", true)
            }

            fn test_stomp_client_frame_send_delete_message_parsing(
                id: &str,
                optional_carraige_return: bool,
            ) {
                let input = encode_send_delete_stomp_frame(id, optional_carraige_return);
                test_stomp_client_frame_send_parsing(
                    input,
                    SendClientFrame::DELETE { id: id.to_owned() },
                )
            }

            fn encode_send_delete_stomp_frame(id: &str, optional_carraige_return: bool) -> String {
                let headers = HashMap::from([(ID, id)]);
                encode_send_stop_frame("DELETE", headers, "", optional_carraige_return)
            }
        }

        fn test_stomp_client_frame_send_parsing(input: String, output: SendClientFrame) {
            test_stomp_client_frame_parsing(input, StompClientFrame::SEND(output));
        }

        fn encode_send_stop_frame<'a>(
            action: &'a str,
            mut headers: HashMap<&'a str, &'a str>,
            body: &'a str,
            optional_carraige_return: bool,
        ) -> String {
            headers.insert(ACTION, action);
            encode_stomp_frame("SEND", headers, body, optional_carraige_return)
        }
    }

    fn test_stomp_client_frame_parsing<S>(input: S, output: StompClientFrame)
    where
        S: Into<String>,
    {
        let message = Message::text(input);
        let result = StompClientFrame::new(&message);

        assert_that(&result).is_ok_containing(output);
    }

    fn encode_stomp_frame_command_only(command: &str, optional_carraige_return: bool) -> String {
        encode_stomp_frame(command, HashMap::new(), "", optional_carraige_return)
    }

    fn encode_stomp_frame_command_with_headers(
        command: &str,
        headers: HashMap<&str, &str>,
        optional_carraige_return: bool,
    ) -> String {
        encode_stomp_frame(command, headers, "", optional_carraige_return)
    }

    fn encode_stomp_frame(
        command: &str,
        headers: HashMap<&str, &str>,
        body: &str,
        optional_carraige_return: bool,
    ) -> String {
        let eol = if optional_carraige_return {
            "\r\n"
        } else {
            "\n"
        };

        let encoded_headers: String = headers
            .iter()
            .map(|(key, value)| format!("{key}:{value}"))
            .fold(String::from(""), |acc, next| format!("{acc}{eol}{next}"));

        return format!("{command}{encoded_headers}{eol}{eol}{body}\0"); // <- eol between command and header is added due to fold first iteration
    }
}