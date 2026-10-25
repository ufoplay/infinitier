//! Compiles the UI, with slint-themekit registered as the `@themekit`
//! library, and bakes git build info into the binary so the running explorer
//! can report which build it is (startup log + window title).
//!
//! The git info is emitted as `VERGEN_GIT_DESCRIBE` / `VERGEN_GIT_BRANCH` /
//! `VERGEN_GIT_SHA`, read via `option_env!` in `build_data.rs`. `Emitter`
//! defaults to non-fatal: without git info it warns and emits placeholders
//! rather than failing the build.

use std::collections::HashMap;

use vergen_gix::{Emitter, Gix};

fn main() {
    let gix = Gix::builder()
        .describe(true, true, None)
        .branch(true)
        .sha(true)
        .build();
    let result = Emitter::default()
        .add_instructions(&gix)
        .and_then(|emitter| emitter.emit());
    if let Err(e) = result {
        println!("cargo:warning=git version info unavailable: {e}");
    }

    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(HashMap::from([slint_themekit::library()]));
    slint_build::compile_with_config("ui/app.slint", config).expect("compile app.slint");
}
