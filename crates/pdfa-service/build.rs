fn main() -> Result<(), Box<dyn std::error::Error>> {
    // no system protoc required: works in CI and in the container build image
    if std::env::var_os("PROTOC").is_none() {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    }
    tonic_prost_build::compile_protos("proto/pdf.proto")?;
    Ok(())
}
