fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("treehole-space-protos/treehole.proto")?;
    Ok(())
}
