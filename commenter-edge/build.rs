extern crate prost_build;

fn main() {
    prost_build::Config::new()
        .compile_protos(&["../protos/comment.proto"], &["../protos"])
        .unwrap();
}
