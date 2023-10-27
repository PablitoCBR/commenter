use serde_json;
use anyhow::Result;

use crate::stomp::StompFrame;
use crate::comments::Comment;

pub fn handle_incoming_comment(frame: StompFrame) -> Result<()> {
    let comment = serde_json::from_slice::<Comment>(&frame.body)?;

    return Ok(());
}