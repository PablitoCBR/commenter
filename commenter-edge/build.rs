extern crate prost_build;

fn main() {
    prost_build::Config::new()
        .type_attribute("Comment", "#[derive(serde::Deserialize)]")
        .enum_attribute("CommentState", "#[derive(num_derive::FromPrimitive)]")
        .compile_protos(&["../protos/comment.proto"], &["../protos"])
        .unwrap();
}
