fn main() {
    tonic_build::compile_protos("proto/echo.proto").unwrap();

    tonic_build::configure()
        .server_mod_attribute("attrs", "#[cfg(feature = \"server\")]")
        .client_mod_attribute("attrs", "#[cfg(feature = \"client\")]")
        .out_dir("src/")
        .compile_with_config(prost_build::Config::default(), &["proto/echo.proto"], &["proto"])
        .unwrap();
}