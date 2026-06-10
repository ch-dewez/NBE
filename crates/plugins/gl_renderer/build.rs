//
//
// MADE WITH AI!
//
//

use assets_management::assets_management::generate_asset_enum;

fn main() {
    generate_asset_enum(
        "shaders", 
        "DefaultShaders", 
        "code", 
        true // Set true to invoke include_str!
    );
}

