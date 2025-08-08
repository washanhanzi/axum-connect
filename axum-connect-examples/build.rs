fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();

    // Add serde support for JSON serialization
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    axum_connect_build::compile(
        &["proto/hello.proto"],
        &["proto/"],
        config
    ).unwrap();

    Ok(())
}
