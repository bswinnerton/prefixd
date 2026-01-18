fn main() -> Result<(), Box<dyn std::error::Error>> {
    // GoBGP v4.2.0 proto files
    tonic_prost_build::configure()
        .build_server(false) // We only need client
        .compile_protos(
            &[
                "proto/gobgp.proto",
                "proto/attribute.proto",
                "proto/capability.proto",
                "proto/common.proto",
                "proto/extcom.proto",
                "proto/nlri.proto",
            ],
            &["proto/"],
        )?;
    Ok(())
}
