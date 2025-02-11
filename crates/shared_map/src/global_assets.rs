use bevy::{color::palettes, prelude::*};

/// Practical agglomeration of assets to get a similar visual in different modules.
#[derive(Debug, Default, Clone, Resource, Reflect)]
pub struct GlobalAssets {
    pub ground_material: Handle<StandardMaterial>,
    pub rock_material: Handle<StandardMaterial>,
    pub rock_mesh: Handle<Mesh>,
    pub muck_pile_mesh: Handle<Mesh>,
    pub muck_pile_material: Handle<StandardMaterial>,
}

pub fn init_global_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let global_assets = GlobalAssets {
        ground_material: materials.add(Color::WHITE),
        rock_material: materials.add(Color::from(palettes::css::DARK_GRAY)),
        rock_mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
        muck_pile_mesh: meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE / 2f32)),
        muck_pile_material: materials.add(Color::from(palettes::css::GOLD)),
    };
    commands.insert_resource(global_assets);
}
