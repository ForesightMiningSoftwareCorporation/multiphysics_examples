pub mod global_assets;
pub mod map_def;
pub mod rock;

use bevy::prelude::*;
use global_assets::{init_global_assets, GlobalAssets};
use map_def::{MapDef, MapDefLoader};

/// Registers MapDef as an asset type.
///
/// Also adds a default [`GlobalAssets`] during [`Startup`] if not present.
pub struct MapDefPlugin;

impl Plugin for MapDefPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MapDef>();
        app.init_asset_loader::<MapDefLoader>();
        app.add_systems(
            Startup,
            init_global_assets.run_if(|res: Option<Res<GlobalAssets>>| res.is_none()),
        );
        app.add_systems(
            Update,
            (
                map_def::on_map_def_changed,
                map_def::on_map_def_handle_changed,
            )
                .chain(),
        );
    }
}
