use bevy::{color::palettes, prelude::*};

/// Practical agglomeration of assets to get a similar visual in different modules.
#[derive(Debug, Default, Clone, Resource, Reflect)]
pub struct GlobalAssets {
    pub ground_material: Handle<StandardMaterial>,
    pub rock_material: Handle<StandardMaterial>,
    pub rock_mesh: Handle<Mesh>,
    pub rock_half_size: f32,
    pub muck_pile_mesh: Handle<Mesh>,
    pub muck_pile_material: Handle<StandardMaterial>,
}

pub fn init_global_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let rock_half_size = 0.6;
    let global_assets = GlobalAssets {
        ground_material: materials.add(Color::WHITE),
        rock_material: materials.add(Color::from(palettes::css::DARK_GRAY)),
        rock_mesh: meshes.add(Cuboid::new(
            rock_half_size * 2f32,
            rock_half_size * 2f32,
            rock_half_size * 2f32,
        )),
        rock_half_size,
        muck_pile_mesh: meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE / 2f32)),
        muck_pile_material: materials.add(Color::from(palettes::css::GOLD)),
    };
    commands.insert_resource(global_assets);
}
