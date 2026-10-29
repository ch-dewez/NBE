use std::path::PathBuf;

use anyhow::Result;
use assets_management::assets_management::generate_asset_enum_from_paths;
use shaders_compilation::{cargo_gpu_install::spirv_builder::ModuleResult, compile_shaders};

pub fn main() -> Result<()> {
    let result = compile_shaders(
        "../../shaders/vulkan-renderer-shader/",
        "spirv-unknown-vulkan1.4",
    )?;
    let paths: Vec<(String, PathBuf)> = match result.module {
        ModuleResult::SingleModule(module) => {
            vec![("VulkanDefaultShader".into(), module)]
        }
        ModuleResult::MultiModule(tree) => tree.into_iter().collect(),
    };
    generate_asset_enum_from_paths("shaders", paths, "VulkanDefaultShaders", "bytes", false);

    Ok(())
}
