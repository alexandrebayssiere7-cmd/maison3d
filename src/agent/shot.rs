//! `maison3d shot` : rend un monde dans une fenêtre, enregistre une vue 3/4 et un plan, puis quitte.

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::camera::ScalingMode;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::window::WindowResolution;

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
                    Ok([x0, z0, x1, z1]) => o.zone = Some((Vec2::new(*x0, *z0), Vec2::new(*x1, *z1))),
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
}

impl Shot {
    fn expected(&self) -> usize {
        self.paths.iter().flatten().count()
    }
}

/// Ouvre une fenêtre, rend le monde, enregistre les images demandées et quitte.
pub(super) fn shot(h: House, png: &str, o: &ShotOpts) {
    let top = match png.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}_dessus.{ext}"),
        None => format!("{png}_dessus"),
    };
    let framing = frame(&h, o);
    let mut ed = Editor::default();
    ed.house = h;
    if let Some(l) = o.level {
        (ed.level, ed.hide_upper) = (l, true);
    }
    let [a, b] = o.views;
    let paths = [a.then(|| png.to_owned()), b.then_some(top)];
    let window = Window {
        title: "Maison 3D — capture".into(),
        resolution: WindowResolution::new(o.size.0 as f32, o.size.1 as f32).with_scale_factor_override(1.0),
        ..default()
    };
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin { primary_window: Some(window), ..default() })
                // sinon les premières images sortent sans les maillages encore en compilation
                .set(RenderPlugin { synchronous_pipeline_compilation: true, ..default() })
                // la sortie standard ne doit contenir que les chemins des images
                .set(LogPlugin { level: Level::ERROR, ..default() }),
        )
        .insert_resource(ed)
        .insert_resource(Shot { paths, framing, plan_level: o.level.unwrap_or(0), saved: 0 })
        .add_systems(Startup, render::setup_scene)
        .add_systems(Update, (render::rebuild, render::update_sun, take_shots).chain())
        .run();
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
        10 => capture(&mut commands, shot.paths[0].as_ref()),
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
        20 => capture(&mut commands, shot.paths[1].as_ref()),
        _ if shot.saved >= shot.expected() => {
            for p in shot.paths.iter().flatten() {
                println!("{p}");
            }
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}

fn capture(commands: &mut Commands, path: Option<&String>) {
    if let Some(path) = path {
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone())).observe(count_saved);
    }
}

fn count_saved(_: Trigger<ScreenshotCaptured>, mut shot: ResMut<Shot>) {
    shot.saved += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;

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
            ("--size", Some("10x10")),
            ("--level", None),
            ("--x", Some("1")),
        ] {
            assert!(opts(&[bad]).is_err(), "{bad:?}");
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
        // des montagnes autour : la caméra passe au-dessus
        h.terrain.h = vec![40.0; crate::terrain::SIDE * crate::terrain::SIDE];
        let high = frame(&h, &ShotOpts::default());
        assert!(high.eye.y > low.eye.y, "{high:?}");
        let ne = frame(&h, &ShotOpts { from: "ne".into(), ..ShotOpts::default() });
        assert!(ne.eye.x > ne.look.x && ne.eye.z < ne.look.z, "nord-est : {ne:?}");
    }

    #[test]
    fn shots_aim_capture_then_quit() {
        let mut world = World::new();
        world.init_resource::<Events<AppExit>>();
        let framing = frame(&House::default(), &ShotOpts::default());
        let paths = [Some("a.png".to_string()), None];
        world.insert_resource(Shot { paths, framing, plan_level: 2, saved: 0 });
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
}
