fn main() {
    tonic_build::compile_protos("../proto/service.proto").unwrap();
}