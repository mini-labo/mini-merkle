extern crate napi_build;

fn main() -> Result<(), ()> {
  napi_build::setup();
  prost_build::compile_protos(&["schemas/merkletree.proto"], &["schemas/"]).unwrap();
  Ok(())
}
