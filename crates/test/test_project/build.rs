use assets_management::assets_management::generate_asset_enum;

fn main() {
    generate_asset_enum(
        "assets", 
        "Assets", 
        "bytes", 
        false
    );
}

