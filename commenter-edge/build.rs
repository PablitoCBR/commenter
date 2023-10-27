extern crate prost_build;

fn main() {
    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&["../protos/comment.proto"], &["../protos"])
        .unwrap();
}
