use anyhow::Result;
use cargo_gpu_install::install::Install;
use cargo_gpu_install::spirv_builder::{CompileResult, ShaderPanicStrategy, SpirvMetadata};
use std::env;
use std::path::{Path, PathBuf};

pub extern crate cargo_gpu_install;

/// shader_crate: path to crate from the directory of the build.rs calling
pub fn compile_shaders<T: AsRef<Path>>(shader_crate: T, target: &str) -> Result<CompileResult> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let mut crate_path = manifest_dir.clone();
    crate_path.push(shader_crate);

    println!(
        "cargo:rerun-if-changed={}",
        crate_path.join("src").display()
    );

    let install = Install::from_shader_crate(crate_path.clone())
        .within_build_script()
        .run()?;
    let mut builder = install.to_spirv_builder(crate_path, target);
    builder.build_script.defaults = true;
    builder.shader_panic_strategy = ShaderPanicStrategy::SilentExit;
    builder.spirv_metadata = SpirvMetadata::Full;

    let compile_result = builder.build()?;
    // let artifact = compile_result.module.unwrap_single();

    // println!("cargo::rustc-env=SHADER_WGSL_PATH={}", artifact.display());
    Ok(compile_result)
}
