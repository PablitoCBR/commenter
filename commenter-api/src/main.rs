#[macro_use]
extern crate rocket;

use commenter_database::comments::Comment;
use diesel::prelude::*;
use rocket::{
    response::status::NotFound,
    serde::json::Json,
};

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/api", routes![get_comment]) // runs default on :8000
}


#[get("/comments/<id>")]
fn get_comment(id: &str) -> Result<Json<commenter_database::comments::Comment>, NotFound<String>> {
    // TODO: DB connection pool
    commenter_database::schema::comments::dsl::comments
        .find(id.to_owned())
        .select(Comment::as_select())
        .first(&mut commenter_database::establish_connection())
        .map_err(|e| NotFound(e.to_string()))
        .map(|comment| Json(comment))
}
