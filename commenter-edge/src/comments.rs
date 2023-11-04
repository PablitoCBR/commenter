include!(concat!(env!("OUT_DIR"), "/comments.rs"));

use uuid::Uuid;

impl Comment {
    pub fn new_create(destination: String, text: String) -> Comment {
        Comment {
            id: Uuid::new_v4().to_string(),
            group_id: destination,
            text,
            state: CommentState::Created.into(),
        }
    }

    pub fn new_update(id: String, text: String) -> Comment {
        Comment {
            id,
            group_id: todo!(),
            text,
            state: CommentState::Updated.into(),
        }
    }

    pub fn new_delete(id: String) -> Comment {
        Comment {
            id,
            group_id: todo!(),
            text: "".to_owned(),
            state: CommentState::Deleted.into(),
        }
    }
}