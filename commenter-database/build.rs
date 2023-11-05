extern crate prost_build;

fn main() {
    prost_build::Config::new()
        .type_attribute(
            "Comment",
            "#[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable)]",
        )
        .type_attribute("Comment", "#[diesel(table_name = crate::schema::comments)]")
        .type_attribute("Comment", "#[derive(serde::Serialize)]")
        .type_attribute("Comment", "#[diesel(check_for_backend(diesel::pg::Pg))]")
        .field_attribute("ID", "#[diesel(sql_type = Text)]")
        .field_attribute("TEXT", "#[diesel(sql_type = Text)]")
        .compile_protos(&["../protos/comment.proto"], &["../protos"])
        .unwrap();
}
