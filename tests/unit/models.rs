use wovensnake::core::config::Config;
use std::collections::HashMap;

#[test]
fn test_config_model() {
    let conf = Config {
        name: "test".into(),
        version: "1.0".into(),
        python_version: "3.8".into(),
        virtual_environment: "env".into(),
        dependencies: HashMap::from([("pip".into(), "20.0".into())]),
    };
    
    // Pure logic assertions
    assert_eq!(conf.name, "test");
    assert!(conf.dependencies.contains_key("pip"));
}
