//! `maison3d shot` : rend un monde hors écran, enregistre une vue 3/4 et un plan, puis quitte.
//! Aucune fenêtre ne s'ouvre : la caméra rend dans une image, l'agent ne vole pas le focus.

use bevy::app::ScheduleRunnerPlugin;
use bevy::asset::RenderAssetUsages;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;

use super::CliError;
use super::describe::building_bounds;
use crate::editor::Editor;
use crate::model::{House, elev};
use crate::render::{self, MainCamera};

/// Réglages de la capture, lus sur la ligne de commande.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ShotOpts {
    /// Étage coupé : on voit jusqu'à lui, plus haut est masqué (3/4 et plan). Absent : 3/4 entier, plan du level 0.
    pub level: Option<i32>,
    /// Point de vue de la 3/4 : "so", "se", "no" ou "ne".
    pub from: String,
    /// Zone cadrée `(min, max)` ; absente : les bâtiments.
    pub zone: Option<(Vec2, Vec2)>,
    pub size: (u32, u32),
    pub views: [bool; 2],
}

impl Default for ShotOpts {
    fn default() -> Self {
        Self { level: None, from: "so".into(), zone: None, size: (1280, 800), views: [true, true] }
    }
}

impl ShotOpts {
    pub(super) fn parse(flags: &[(&str, Option<&str>)]) -> Result<Self, CliError> {
        let mut o = Self::default();
        let bad = |flag: &str, v: &str| CliError::BadOption(format!("{flag} {v}"));
        let nums = |v: &str| v.split(',').map(str::parse::<f32>).collect::<Result<Vec<_>, _>>();
        for &(flag, value) in flags {
            let v = value.ok_or_else(|| CliError::BadOption(format!("{flag} sans valeur")))?;
            match flag {
                "--level" => o.level = Some(v.parse().map_err(|_| bad(flag, v))?),
                "--from" if ["so", "se", "no", "ne"].contains(&v) => o.from = v.into(),
                "--zone" => match nums(v).as_deref() {
                    // `parse` accepte NaN et inf : une zone doit être finie et non vide
                    Ok(&[x0, z0, x1, z1]) if [x0, z0, x1, z1].iter().all(|n| n.is_finite()) && x0 < x1 && z0 < z1 => {
                        o.zone = Some((Vec2::new(x0, z0), Vec2::new(x1, z1)));
                    }
                    _ => return Err(bad(flag, v)),
                },
                "--size" => match v.split_once('x').map(|(w, h)| (w.parse(), h.parse())) {
                    Some((Ok(w), Ok(h))) if (200..=4000).contains(&w) && (200..=4000).contains(&h) => o.size = (w, h),
                    _ => return Err(bad(flag, v)),
                },
                "--only" if v == "3/4" => o.views = [true, false],
                "--only" if v == "plan" => o.views = [false, true],
                _ => return Err(bad(flag, v)),
            }
        }
        Ok(o)
    }
}

/// Où placer la caméra : déduit du monde et des options, sans moteur (testable).
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Framing {
    pub eye: Vec3,
    pub look: Vec3,
    /// Centre et taille de la zone vue de dessus.
    pub plan: (Vec3, Vec2),
}

pub(super) fn frame(h: &House, o: &ShotOpts) -> Framing {
    let (lo, hi) = o.zone.or_else(|| building_bounds(h)).unwrap_or((Vec2::splat(-10.0), Vec2::splat(10.0)));
    let c = (lo + hi) / 2.0;
    let ground = h.ground(c).max(-5.0);
    // hauteur de ce qui est montré : murs et toits (coupés à l'étage choisi)
    let shown = |l: i32| o.level.is_none_or(|m| l <= m);
    let tops = (h.walls.iter().filter(|w| shown(w.level)).map(|w| elev(w.level) + w.height))
        .chain(h.roofs.iter().filter(|r| shown(r.level)).map(|r| elev(r.level) + r.height));
    let top = tops.fold(3.0f32, f32::max);
    let look = Vec3::new(c.x, ground + top / 2.0, c.y);
    let size = hi - lo;
    let radius = 0.5 * (size.length_squared() + top * top).sqrt().max(8.0);
    let (sx, sz) = match o.from.as_str() {
        "se" => (1.0, 1.0),
        "no" => (-1.0, -1.0),
        "ne" => (1.0, -1.0),
        _ => (-1.0, 1.0),
    };
    let dir = Vec3::new(0.56 * sx, 0.56, 0.62 * sz).normalize();
    // champ vertical de 45° : la sphère englobante tient dans l'image
    let mut eye = look + dir * radius / (std::f32::consts::FRAC_PI_8).sin() * 1.05;
    // remonte la caméra tant qu'une colline masque la vue
    for _ in 0..40 {
        let hidden = (1..24).any(|k| {
            let p = eye.lerp(look, k as f32 / 24.0);
            h.ground(Vec2::new(p.x, p.z)) + 1.0 > p.y
        });
        if !hidden {
            break;
        }
        eye.y += 5.0;
    }
    Framing { eye, look, plan: (Vec3::new(c.x, ground, c.y), size) }
}

#[derive(Resource)]
pub(super) struct Shot {
    /// Chemins de la 3/4 et du plan ; `None` pour une vue non demandée.
    pub paths: [Option<String>; 2],
    pub framing: Framing,
    pub plan_level: i32,
    pub saved: usize,
    /// Image dans laquelle rend la caméra (pas de fenêtre).
    pub target: Handle<Image>,
    pub size: (u32, u32),
}

impl Shot {
    fn expected(&self) -> usize {
        self.paths.iter().flatten().count()
    }
}

/// Chemins des deux vues demandées : perspective sous `png`, dessus sous `<nom>_dessus.<ext>`.
fn shot_paths(png: &str, [persp, plan]: [bool; 2]) -> [Option<String>; 2] {
    let top = match png.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}_dessus.{ext}"),
        None => format!("{png}_dessus"),
    };
    [persp.then(|| png.to_owned()), plan.then_some(top)]
}

/// Rend le monde hors écran, enregistre les images demandées et quitte.
pub(super) fn shot(h: House, png: &str, o: &ShotOpts) {
    let framing = frame(&h, o);
    let mut ed = Editor::default();
    ed.house = h;
    if let Some(l) = o.level {
        (ed.level, ed.hide_upper) = (l, true);
    }
    let paths = shot_paths(png, o.views);
    let shot =
        Shot { paths, framing, plan_level: o.level.unwrap_or(0), saved: 0, target: Handle::default(), size: o.size };
    App::new()
        .add_plugins(
            DefaultPlugins
                // pas de fenêtre ni de boucle d'événements : rendu hors écran, la boucle tourne seule
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    close_when_requested: false,
                })
                .disable::<WinitPlugin>()
                // sinon les premières images sortent sans les maillages encore en compilation
                .set(RenderPlugin { synchronous_pipeline_compilation: true, ..default() })
                // la sortie standard ne doit contenir que les chemins des images
                .set(LogPlugin { level: Level::ERROR, ..default() }),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::ZERO))
        .insert_resource(ed)
        .insert_resource(shot)
        .add_systems(Startup, (render::setup_scene, render_to_image).chain())
        .add_systems(Update, (render::rebuild, render::update_sun, take_shots).chain())
        .run();
}

/// Image vide aux dimensions demandées, utilisable comme cible de rendu et lisible pour la capture.
fn target_image((width, height): (u32, u32)) -> Image {
    let size = Extent3d { width, height, depth_or_array_layers: 1 };
    let format = TextureFormat::Rgba8UnormSrgb;
    let mut img = Image::new_fill(size, TextureDimension::D2, &[0, 0, 0, 255], format, RenderAssetUsages::default());
    img.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::COPY_SRC
        | TextureUsages::RENDER_ATTACHMENT;
    img
}

/// Fait rendre la caméra principale dans une image plutôt que dans une fenêtre.
fn render_to_image(
    mut images: ResMut<Assets<Image>>,
    mut shot: ResMut<Shot>,
    mut cam: Single<&mut Camera, With<MainCamera>>,
) {
    shot.target = images.add(target_image(shot.size));
    cam.target = RenderTarget::Image(shot.target.clone().into());
}

pub(super) fn take_shots(
    mut commands: Commands,
    mut frame: Local<u32>,
    shot: Res<Shot>,
    mut ed: ResMut<Editor>,
    cam: Single<(Entity, &mut Transform), With<MainCamera>>,
    mut exit: EventWriter<AppExit>,
) {
    *frame += 1;
    let (e, mut tf) = cam.into_inner();
    let f = &shot.framing;
    match *frame {
        // quelques images pour que les ombres et l'exposition se posent
        5 => *tf = Transform::from_translation(f.eye).looking_at(f.look, Vec3::Y),
        10 => capture(&mut commands, &shot.target, shot.paths[0].as_ref()),
        15 => {
            // vue de dessus orthographique, le nord (-z) en haut : les distances se lisent à l'échelle
            let (center, size) = f.plan;
            *tf = Transform::from_translation(center + Vec3::Y * 200.0).looking_at(center, Vec3::NEG_Z);
            let m = size * 1.15 + 2.0;
            let ortho = OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin { min_width: m.x, min_height: m.y },
                far: 1000.0,
                ..OrthographicProjection::default_3d()
            };
            commands.entity(e).insert(Projection::from(ortho));
            // plan de l'étage choisi : ce qui est au-dessus est masqué
            (ed.level, ed.hide_upper, ed.dirty) = (shot.plan_level, true, true);
        }
        20 => capture(&mut commands, &shot.target, shot.paths[1].as_ref()),
        _ if shot.saved >= shot.expected() => {
            for p in shot.paths.iter().flatten() {
                println!("{p}");
            }
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}

fn capture(commands: &mut Commands, target: &Handle<Image>, path: Option<&String>) {
    if let Some(path) = path {
        commands.spawn(Screenshot::image(target.clone())).observe(save_to_disk(path.clone())).observe(count_saved);
    }
}

fn count_saved(_: Trigger<ScreenshotCaptured>, mut shot: ResMut<Shot>) {
    shot.saved += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::terrain::{SIDE, Terrain};

    fn opts(flags: &[(&'static str, Option<&'static str>)]) -> Result<ShotOpts, CliError> {
        ShotOpts::parse(flags)
    }

    #[test]
    fn options_parse_and_reject_bad_values() {
        let o = opts(&[
            ("--level", Some("1")),
            ("--from", Some("ne")),
            ("--zone", Some("0,0,10,8")),
            ("--size", Some("800x600")),
        ])
        .unwrap();
        assert_eq!((o.level, o.from.as_str(), o.size), (Some(1), "ne", (800, 600)));
        assert_eq!(o.zone, Some((Vec2::ZERO, Vec2::new(10.0, 8.0))));
        assert_eq!(opts(&[("--only", Some("plan"))]).unwrap().views, [false, true]);
        for bad in [
            ("--from", Some("haut")),
            ("--zone", Some("1,2")),
            ("--zone", Some("NaN,0,10,8")),
            ("--zone", Some("0,0,inf,8")),
            ("--zone", Some("10,0,0,8")),
            ("--zone", Some("0,8,10,8")),
            ("--size", Some("10x10")),
            ("--level", None),
            ("--x", Some("1")),
        ] {
            assert!(matches!(opts(&[bad]), Err(CliError::BadOption(_))), "{bad:?}");
        }
    }

    #[test]
    fn framing_covers_tall_buildings_and_clears_hills() {
        let mut h = House::default();
        h.apply(
            serde_json::from_str::<Command>(r#"{"op":"room","min":[0,0],"max":[8,8],"level":0,"height":12}"#).unwrap(),
        )
        .unwrap();
        let low = frame(&h, &ShotOpts::default());
        assert!(low.look.y > 5.0, "vise le milieu de la hauteur : {low:?}");
        // une longue route ne dézoome plus : seuls les bâtiments comptent
        h.apply(
            serde_json::from_str::<Command>(
                r#"{"op":"add_road","a":[-70,20],"b":[70,20],"width":6,"color":[1,1,1],"texture":0}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(frame(&h, &ShotOpts::default()).eye, low.eye);
        // une colline entre la caméra (sud-ouest) et la maison : la caméra passe au-dessus
        h.terrain.h = (0..SIDE * SIDE)
            .map(|k| {
                let p = Terrain::vertex(k % SIDE, k / SIDE);
                if p.x < -6.0 && p.y > 14.0 { 60.0 } else { 0.0 }
            })
            .collect();
        let high = frame(&h, &ShotOpts::default());
        assert_eq!(high.look, low.look, "la cible ne bouge pas, seule la caméra monte");
        assert!(high.eye.y > low.eye.y + 20.0, "{high:?}");
        let clear = (1..24).all(|k| {
            let p = high.eye.lerp(high.look, k as f32 / 24.0);
            h.ground(Vec2::new(p.x, p.z)) + 1.0 <= p.y
        });
        assert!(clear, "plus rien ne masque la vue");
        let ne = frame(&h, &ShotOpts { from: "ne".into(), ..ShotOpts::default() });
        assert!(ne.eye.x > ne.look.x && ne.eye.z < ne.look.z, "nord-est : {ne:?}");
    }

    #[test]
    fn target_image_is_a_readable_render_target() {
        let img = target_image((320, 200));
        assert_eq!((img.width(), img.height()), (320, 200));
        let usage = img.texture_descriptor.usage;
        assert!(usage.contains(TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC), "{usage:?}");
    }

    #[test]
    fn shots_aim_capture_then_quit() {
        let mut world = World::new();
        world.init_resource::<Events<AppExit>>();
        let framing = frame(&House::default(), &ShotOpts::default());
        let paths = [Some("a.png".to_string()), None];
        let (target, size) = (Handle::default(), (64, 32));
        world.insert_resource(Shot { paths, framing, plan_level: 2, saved: 0, target, size });
        world.insert_resource(Editor::default());
        let cam = world.spawn((MainCamera, Transform::default())).id();
        let id = world.register_system(take_shots);
        for _ in 0..25 {
            world.run_system(id).unwrap();
        }
        assert!(world.get::<Projection>(cam).is_some(), "vue de dessus orthographique");
        assert_eq!(world.resource::<Editor>().level, 2, "plan de l'étage demandé");
        assert!(world.resource::<Events<AppExit>>().is_empty(), "attend l'image demandée");
        world.resource_mut::<Shot>().saved = 1;
        world.run_system(id).unwrap();
        assert!(!world.resource::<Events<AppExit>>().is_empty());
    }

    #[test]
    fn plan_view_is_saved_next_to_the_perspective() {
        let both = shot_paths("out/maison.png", [true, true]);
        assert_eq!(both, [Some("out/maison.png".into()), Some("out/maison_dessus.png".into())]);
        assert_eq!(shot_paths("maison", [false, true]), [None, Some("maison_dessus".into())]);
        assert_eq!(shot_paths("maison.png", [true, false]), [Some("maison.png".into()), None]);
    }
}
