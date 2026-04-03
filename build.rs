fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/deploy_manager.proto")?;
    tonic_build::compile_protos("proto/managed_application.proto")?;
    tonic_build::compile_protos("proto/application_manager.proto")?;
    Ok(())
}
