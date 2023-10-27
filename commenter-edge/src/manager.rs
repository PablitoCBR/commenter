use tokio::sync::{mpsc, RwLock};
use std::{default::Default, collections::HashMap};
use anyhow::{Result, Context};

use crate::comments::Comment;

#[derive(Default)]
pub struct Manager {
    groups: RwLock<HashMap<String, Vec<mpsc::UnboundedSender<Comment>>>>
}

impl Manager {
    async fn add(self, group_id: &str, sender: mpsc::UnboundedSender<Comment>) {
        let mut groups_write = self.groups.write().await;

        match groups_write.get_mut(group_id) {
            Some(group_senders) => group_senders.push(sender),
            None => {
                groups_write.insert(group_id.to_string(), vec![sender]);
            }
        };
    }

    async fn remove(self, group_id: &str, sender: mpsc::UnboundedSender<Comment>) {
        let mut groups_write = self.groups.write().await;

        if let Some(group) = groups_write.get_mut(group_id) {
            
        }
    }

    async fn deliver(&self, group_id: &str, comment: Comment) -> Option<Result<()>> {
        if let Some(group_senders) = self.groups.read().await.get(group_id) {
            let mut outcome = Result::Ok(());


            group_senders.iter().for_each(|sender|{
                if let Err(sending_error) = sender.send(comment.clone()) {
                    
                }
            });

            return Some(outcome);
        }

        return None;
    }
}