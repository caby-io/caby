use config::Config;

pub fn load_config() -> Config {
    Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("examples/simple/Settings"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap()
}
