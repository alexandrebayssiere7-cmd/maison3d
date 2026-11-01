//! Scène 3D : sol, lumière du soleil, et reconstruction des maillages de la maison.

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::editor::Editor;
use crate::model::{ElemRef, Part, RoofKind, Rgb, Shape};
use crate::textures;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Sun;

/// Entités générées depuis la maison, détruites à chaque reconstruction.
#[derive(Component)]
pub struct Built;

#[derive(Resource)]
pub struct Library {
    cube: Handle<Mesh>,
    cylinder: Handle<Mesh>,
    sphere: Handle<Mesh>,
    cone: Handle<Mesh>,
    glass: Handle<StandardMaterial>,
    mats: HashMap<([u8; 3], usize), Handle<StandardMaterial>>,
    /// Textures du catalogue : (nom, image, taille du motif en m). Index = id de texture - 1.
    pub textures: Vec<(&'static str, Handle<Image>, f32)>,
}

impl Library {
    fn material(&mut self, assets: &mut Assets<StandardMaterial>, c: Rgb, texture: usize) -> Handle<StandardMaterial> {
        let key = (c.map(|v| (v.clamp(0.0, 1.0) * 255.0) as u8), texture);
        let tex = texture.checked_sub(1).and_then(|i| self.textures.get(i));
        self.mats
            .entry(key)
            .or_insert_with(|| {
                let [r, g, b] = key.0;
                assets.add(StandardMaterial {
                    base_color: Color::srgb_u8(r, g, b),
                    base_color_texture: tex.map(|t| t.1.clone()),
                    // les UV des maillages sont en mètres : on les ramène à la taille du motif
                    uv_transform: Affine2::from_scale(Vec2::splat(1.0 / tex.map_or(1.0, |t| t.2))),
                    perceptual_roughness: 0.85,
                    // les toits n'ont pas de face intérieure : on dessine les deux côtés
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone()
    }
}

fn texture_image(t: &textures::Tex) -> Image {
    let (data, levels) = textures::pixels_with_mips(t);
    let n = textures::SIZE as u32;
    let first = data[..(n * n * 4) as usize].to_vec();
    let size = Extent3d { width: n, height: n, depth_or_array_layers: 1 };
    let mut img = Image::new(size, TextureDimension::D2, first, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    // mipmaps calculés sur CPU : sans eux les motifs scintillent au loin
    img.texture_descriptor.mip_level_count = levels;
    img.data = Some(data);
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 8,
        ..default()
    });
    img
}

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let textures = textures::catalog().iter().map(|t| (t.name, images.add(texture_image(t)), t.scale)).collect();
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(600.0, 600.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.36, 0.55, 0.3),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.005, 0.0),
    ));
    commands.spawn((Camera3d::default(), MainCamera, Transform::from_xyz(-10.0, 15.0, 25.0)));
    commands.spawn((Sun, DirectionalLight { shadows_enabled: true, ..default() }, Transform::default()));
    commands.insert_resource(Library {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        cylinder: meshes.add(Cylinder::new(0.5, 1.0)),
        // sphère en UV (et non icosaèdre) pour que les textures de feuillage s'enroulent proprement
        sphere: meshes.add(Sphere::new(0.5).mesh().uv(32, 18)),
        cone: meshes.add(Cone::new(0.5, 1.0)),
        glass: materials.add(StandardMaterial {
            base_color: Color::srgba(0.6, 0.8, 0.95, 0.3),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.05,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        mats: HashMap::new(),
        textures,
    });
}

/// Recrée toutes les entités de la maison quand elle a changé.
pub fn rebuild(
    mut commands: Commands,
    mut ed: ResMut<Editor>,
    mut lib: ResMut<Library>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    old: Query<Entity, With<Built>>,
) {
    if !ed.dirty {
        return;
    }
    ed.dirty = false;
    for e in &old {
        commands.entity(e).despawn();
    }
    let open = |e| ed.doors.get(&e).map_or(0.0, |d| d.0);
    let mut parts = ed.house.parts(ed.hide_upper.then_some(ed.level), &open);
    // la voiture conduite est affichée à part, avec ses roues animées
    if let Some(i) = ed.driving {
        parts.retain(|p| p.owner != ElemRef::Prop(i));
    }
    for p in &parts {
        if p.shape == Shape::Light {
            let c = Color::srgb(p.color[0], p.color[1], p.color[2]);
            commands.spawn((Built, PointLight { color: c, intensity: 300_000.0, range: 15.0, ..default() }, p.tf.with_scale(Vec3::ONE)));
        } else {
            commands.spawn((Built, part_bundle(&mut lib, &mut meshes, &mut materials, p)));
        }
    }
    ed.parts = parts;
}

/// Maillage, matériau et placement d'une pièce (hors lumières).
pub fn part_bundle(
    lib: &mut Library,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    p: &Part,
) -> (Mesh3d, MeshMaterial3d<StandardMaterial>, Transform) {
    let mut tf = p.tf;
    let mesh = match p.shape {
        Shape::Cube => {
            tf.scale = Vec3::ONE;
            meshes.add(box_mesh(p.tf))
        }
        Shape::Wheel => {
            // cylindre couché : son axe (Y) passe sur X
            tf.rotation *= Quat::from_rotation_z(FRAC_PI_2);
            tf.scale = Vec3::new(p.tf.scale.y, p.tf.scale.x, p.tf.scale.z);
            lib.cylinder.clone()
        }
        Shape::Glass | Shape::Light => lib.cube.clone(),
        Shape::Cylinder => lib.cylinder.clone(),
        Shape::Sphere => lib.sphere.clone(),
        Shape::Cone => lib.cone.clone(),
        Shape::Roof { kind, size } => meshes.add(roof_mesh(kind, size)),
    };
    let mat = match p.shape {
        Shape::Glass => lib.glass.clone(),
        _ => lib.material(materials, p.color, p.texture),
    };
    (Mesh3d(mesh), MeshMaterial3d(mat), tf)
}

/// Toit en pente, faîtage le long de X. `size` = (longueur, hauteur, largeur).
fn roof_mesh(kind: RoofKind, size: Vec3) -> Mesh {
    flat_mesh(&crate::model::roof_tris(kind, size), Vec3::new(0.0, size.y * 0.3, 0.0), Vec3::ZERO)
}

/// Boîte aux dimensions réelles de `tf.scale` (à afficher avec une échelle 1).
/// Les UV dépendent de la position dans le repère de la boîte : les morceaux d'un même mur
/// partagent ce repère, donc la texture se raccorde autour des fenêtres.
fn box_mesh(tf: Transform) -> Mesh {
    let h = tf.scale / 2.0;
    let c = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * h;
    let quads = [
        [c(1., -1., -1.), c(1., 1., -1.), c(1., 1., 1.), c(1., -1., 1.)],
        [c(-1., -1., -1.), c(-1., 1., -1.), c(-1., 1., 1.), c(-1., -1., 1.)],
        [c(-1., 1., -1.), c(1., 1., -1.), c(1., 1., 1.), c(-1., 1., 1.)],
        [c(-1., -1., -1.), c(1., -1., -1.), c(1., -1., 1.), c(-1., -1., 1.)],
        [c(-1., -1., 1.), c(1., -1., 1.), c(1., 1., 1.), c(-1., 1., 1.)],
        [c(-1., -1., -1.), c(1., -1., -1.), c(1., 1., -1.), c(-1., 1., -1.)],
    ];
    let tris: Vec<[Vec3; 3]> = quads.iter().flat_map(|[a, b, c, d]| [[*a, *b, *c], [*a, *c, *d]]).collect();
    flat_mesh(&tris, Vec3::ZERO, tf.rotation.inverse() * tf.translation)
}

/// Maillage à facettes ; chaque triangle est orienté à l'opposé de `inside`.
/// UV en mètres, projetés sur le plan le plus proche de la face (décalés de `uv_offset`).
fn flat_mesh(tris: &[[Vec3; 3]], inside: Vec3, uv_offset: Vec3) -> Mesh {
    let (mut pos, mut uvs) = (Vec::new(), Vec::new());
    for &[p0, p1, p2] in tris {
        let n = (p1 - p0).cross(p2 - p0);
        let t = if n.dot((p0 + p1 + p2) / 3.0 - inside) >= 0.0 { [p0, p1, p2] } else { [p0, p2, p1] };
        let a = n.abs();
        for p in t {
            let q = p + uv_offset;
            uvs.push(if a.y >= a.x && a.y >= a.z {
                [q.x, q.z]
            } else if a.x >= a.z {
                [q.z, -q.y]
            } else {
                [q.x, -q.y]
            });
            pos.push(p.to_array());
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_computed_flat_normals()
}

/// Position du soleil, couleur du ciel et lumière ambiante selon l'heure.
pub fn update_sun(
    ed: Res<Editor>,
    mut sun: Single<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<AmbientLight>,
) {
    let angle = (ed.hour - 6.0) / 12.0 * PI;
    let day = angle.sin().clamp(0.0, 1.0);
    let (tf, light) = &mut *sun;
    **tf = Transform::from_xyz(angle.cos(), angle.sin().max(0.05), 0.6).looking_at(Vec3::ZERO, Vec3::Y);
    light.illuminance = 11_000.0 * day;
    // lumière plus orangée au lever et au coucher
    let warm = 1.0 - day.powf(0.4);
    light.color = Color::srgb(1.0, 1.0 - 0.35 * warm, 1.0 - 0.6 * warm);
    let night = Vec3::new(0.02, 0.03, 0.08);
    let sky = Vec3::new(0.55, 0.75, 0.95).lerp(Vec3::new(0.95, 0.6, 0.4), warm * 0.6);
    let c = night.lerp(sky, day.powf(0.5));
    clear.0 = Color::srgb(c.x, c.y, c.z);
    ambient.brightness = 60.0 + 500.0 * day;
}
