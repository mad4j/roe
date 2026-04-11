fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    // SAFETY: build scripts are single-threaded; no other threads read PROTOC at this point.
    unsafe { std::env::set_var("PROTOC", protoc) };

    tonic_build::compile_protos("proto/deploy_manager.proto")?;
    tonic_build::compile_protos("proto/managed_application.proto")?;
    tonic_build::compile_protos("proto/application_factory.proto")?;
    tonic_build::compile_protos("proto/configurable_application.proto")?;
    Ok(())
}
