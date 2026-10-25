//! Compiles the UI, with slint-themekit registered as the `@themekit` library.

use std::collections::HashMap;

fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(HashMap::from([slint_themekit::library()]));
    slint_build::compile_with_config("ui/app.slint", config).expect("compile app.slint");
}
