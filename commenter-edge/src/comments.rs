include!(concat!(env!("OUT_DIR"), "/comments.rs"));

use anyhow::Result;
use uuid::Uuid;

impl Comment {
    pub fn new_create(destination: String, text: String) -> Comment {
        Comment {
            id: Uuid::new_v4().to_string(),
            group_id: destination,
            text: text,
            state: CommentState::Created.into(),
        }
    }

    pub async fn new_update(id: String, text: String) -> Result<Comment> {
        let stored_comment = Comment::get_stored_comment(&id).await?;

        Ok(Comment {
            id,
            group_id: stored_comment.group_id,
            text: text,
            state: CommentState::Updated.into(),
        })
    }

    pub async fn new_delete(id: String) -> Result<Comment> {
        let stored_comment = Comment::get_stored_comment(&id).await?;

        Ok(Comment {
            id,
            group_id: stored_comment.group_id,
            text: stored_comment.text,
            state: CommentState::Deleted.into(),
        })
    }

    async fn get_stored_comment(id: &str) -> Result<Comment> {
        Ok(
            reqwest::get(format!("http://localhost:8000/api/comments/{}", id))
                .await?
                .json::<Comment>()
                .await?,
        )
    }
}
