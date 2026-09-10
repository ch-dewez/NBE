//
//
//
// THIS FILE WAS AI GENERATED
//
// I'm not yet familiar with this type of macros (modifying the enum inside) but I understand how it works
// Maybe this should be done in a derive macro but that requires proc macro which I'm even less
// familiar
//
//
//

/// Automatically generates a type-safe Scene enum and a static path lookup table.
///
/// Scene save fils are not yet supported, but you can not use them
///
/// Example usage:
///define_scenes! {
///pub enum GameScene {
///     MainMenu => "assets/scenes/main_menu.ron",
///     Gameplay => "assets/scenes/level_1.ron",
///     GameOver => "assets/scenes/game_over.ron",
///}
///
///define_scenes! {
///pub enum GameScene {
///     MainMenu,
///     Gameplay,
///     GameOver,
///}
///
#[macro_export]
macro_rules! define_scenes {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $variant:ident => $path:expr ),* $(,)?
        }
    ) => {
        // 1. Generate the pure, lightweight enum (just numbers under the hood)
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u32)]
        $vis enum $name {
            $($variant),*
        }

        // 2. Generate the static lookup table as a compile-time const function
        impl $name {
            /// Looks up the scene file path. Compiles down to an O(1) jump table.
            pub const fn file_path(self) -> &'static str {
                match self {
                    $( $name::$variant => $path, )*
                }
            }
        }

        // 3. Engine integer conversions (from previous step)
        impl From<$name> for $crate::EngineSceneId {
            #[inline(always)]
            fn from(scene: $name) -> Self {
                $crate::EngineSceneId(scene as u32)
            }
        }

        impl TryFrom<$crate::EngineSceneId> for $name {
            type Error = &'static str;
            #[inline(always)]
            fn try_from(id: $crate::EngineSceneId) -> Result<Self, Self::Error> {
                let mut current_id = 0u32;
                $(
                    if id.0 == current_id { return Ok($name::$variant); }
                    current_id += 1;
                )*
                Err("Unknown scene ID")
            }
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $variant:ident),* $(,)?
        }
    ) => {
        // 1. Generate the pure, lightweight enum (just numbers under the hood)
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u32)]
        $vis enum $name {
            $($variant),*
        }

        // 3. Engine integer conversions (from previous step)
        impl From<$name> for $crate::EngineSceneId {
            #[inline(always)]
            fn from(scene: $name) -> Self {
                $crate::EngineSceneId(scene as u32)
            }
        }

        impl TryFrom<$crate::EngineSceneId> for $name {
            type Error = &'static str;
            #[inline(always)]
            fn try_from(id: $crate::EngineSceneId) -> Result<Self, Self::Error> {
                let mut current_id = 0u32;
                $(
                    if id.0 == current_id { return Ok($name::$variant); }
                    current_id += 1;
                )*
                Err("Unknown scene ID")
            }
        }
    };
}
