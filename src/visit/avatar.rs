//! Personnage articulé : construction, poses selon la posture et la marche, application aux articulations.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::{HIPS_Y, RUN_SPEED, Stance, Visitor, WALK_SPEED};
use crate::render::MainCamera;

/// Angles des articulations (radians) et hauteur du bassin ; indice 0 = gauche, 1 = droite.
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct Pose {
    pub(super) hips_y: f32,
    pub(super) lie: f32,
    pub(super) spine: f32,
    pub(super) head: f32,
    pub(super) sh: [f32; 2],
    pub(super) el: [f32; 2],
    pub(super) hip: [f32; 2],
    pub(super) kn: [f32; 2],
}

impl Pose {
    pub(super) fn blend(&mut self, t: &Pose, k: f32) {
        let l = |a: &mut f32, b: f32| *a += (b - *a) * k;
        l(&mut self.hips_y, t.hips_y);
        l(&mut self.lie, t.lie);
        l(&mut self.spine, t.spine);
        l(&mut self.head, t.head);
        for i in 0..2 {
            l(&mut self.sh[i], t.sh[i]);
            l(&mut self.el[i], t.el[i]);
            l(&mut self.hip[i], t.hip[i]);
            l(&mut self.kn[i], t.kn[i]);
        }
    }
}

#[derive(Component)]
pub struct AvatarRoot;

#[derive(Component, Clone, Copy)]
pub enum Joint {
    Hips,
    Spine,
    Head,
    Shoulder(usize),
    Elbow(usize),
    Hip(usize),
    Knee(usize),
}

// ---------------------------------------------------------------- personnage

pub fn spawn_avatar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let ball = meshes.add(Sphere::new(0.5).mesh().uv(24, 14));
    let mut mat = |r: u8, g: u8, b: u8| {
        mats.add(StandardMaterial { base_color: Color::srgb_u8(r, g, b), perceptual_roughness: 0.8, ..default() })
    };
    let (skin, shirt, pants, shoes, hair, eye) =
        (mat(225, 180, 150), mat(50, 110, 170), mat(45, 50, 70), mat(35, 30, 30), mat(70, 45, 30), mat(20, 20, 25));
    let part = |mesh: &Handle<Mesh>, m: &Handle<StandardMaterial>, c: [f32; 3], s: [f32; 3]| {
        (Mesh3d(mesh.clone()), MeshMaterial3d(m.clone()), Transform::from_translation(c.into()).with_scale(s.into()))
    };
    let joint = |j: Joint, at: [f32; 3]| (j, Transform::from_translation(at.into()), Visibility::Inherited);

    // le personnage regarde vers -Z ; toutes les articulations pivotent autour de X
    commands.spawn((AvatarRoot, Transform::default(), Visibility::Hidden)).with_children(|root| {
        root.spawn(joint(Joint::Hips, [0.0, HIPS_Y, 0.0])).with_children(|hips| {
            hips.spawn(part(&cube, &pants, [0.0, 0.0, 0.0], [0.34, 0.16, 0.2]));
            hips.spawn(joint(Joint::Spine, [0.0, 0.0, 0.0])).with_children(|sp| {
                sp.spawn(part(&cube, &shirt, [0.0, 0.3, 0.0], [0.38, 0.5, 0.21]));
                sp.spawn(part(&cube, &skin, [0.0, 0.6, 0.0], [0.09, 0.1, 0.09]));
                sp.spawn(joint(Joint::Head, [0.0, 0.72, 0.0])).with_children(|h| {
                    h.spawn(part(&ball, &skin, [0.0, 0.0, 0.0], [0.2, 0.25, 0.22]));
                    h.spawn(part(&ball, &hair, [0.0, 0.05, 0.02], [0.215, 0.2, 0.225]));
                    h.spawn(part(&ball, &eye, [-0.045, 0.02, -0.1], [0.03, 0.03, 0.02]));
                    h.spawn(part(&ball, &eye, [0.045, 0.02, -0.1], [0.03, 0.03, 0.02]));
                });
                for (i, side) in [(0, -1.0), (1, 1.0)] {
                    sp.spawn(joint(Joint::Shoulder(i), [side * 0.24, 0.52, 0.0])).with_children(|s| {
                        s.spawn(part(&cube, &shirt, [0.0, -0.14, 0.0], [0.1, 0.3, 0.1]));
                        s.spawn(joint(Joint::Elbow(i), [0.0, -0.29, 0.0])).with_children(|e| {
                            e.spawn(part(&cube, &skin, [0.0, -0.13, 0.0], [0.08, 0.27, 0.08]));
                            e.spawn(part(&ball, &skin, [0.0, -0.3, 0.0], [0.09, 0.1, 0.09]));
                        });
                    });
                }
            });
            for (i, side) in [(0, -1.0), (1, 1.0)] {
                hips.spawn(joint(Joint::Hip(i), [side * 0.1, -0.03, 0.0])).with_children(|l| {
                    l.spawn(part(&cube, &pants, [0.0, -0.22, 0.0], [0.14, 0.45, 0.15]));
                    l.spawn(joint(Joint::Knee(i), [0.0, -0.45, 0.0])).with_children(|k| {
                        k.spawn(part(&cube, &pants, [0.0, -0.21, 0.0], [0.12, 0.43, 0.13]));
                        k.spawn(part(&cube, &shoes, [0.0, -0.44, -0.05], [0.11, 0.07, 0.26]));
                    });
                });
            }
        });
    });
}

pub(super) fn hips_tf(p: &Pose) -> Transform {
    Transform::from_xyz(0.0, p.hips_y, 0.0).with_rotation(Quat::from_rotation_x(p.lie))
}

pub(super) fn root_tf(v: &Visitor) -> Transform {
    Transform::from_translation(v.root).with_rotation(Quat::from_rotation_y(v.root_yaw))
}

/// Position des yeux d'après la pose courante (suit le balancement de la marche).
pub(super) fn eyes(v: &Visitor) -> Vec3 {
    let spine = Transform::from_rotation(Quat::from_rotation_x(v.pose.spine));
    let head = Transform::from_xyz(0.0, 0.72, 0.0).with_rotation(Quat::from_rotation_x(v.pose.head));
    (root_tf(v) * hips_tf(&v.pose) * spine * head).transform_point(Vec3::new(0.0, 0.03, -0.09))
}

/// Pose visée selon la posture et la vitesse ; la marche et la course suivent `phase`.
pub(super) fn target_pose(v: &Visitor, speed: f32, t: f32) -> Pose {
    let head = (v.pitch * 0.6).clamp(-0.6, 0.6);
    match v.stance {
        Stance::Sit => Pose {
            hips_y: 0.1,
            spine: -0.05,
            head,
            sh: [0.35; 2],
            el: [0.9; 2],
            hip: [1.5; 2],
            kn: [-1.5; 2],
            ..default()
        },
        Stance::SitFloor => Pose {
            hips_y: 0.12,
            spine: 0.12,
            head,
            sh: [-0.45; 2],
            el: [0.0; 2],
            hip: [1.45, 1.3],
            kn: [-0.2, -0.6],
            ..default()
        },
        Stance::Lie => Pose {
            hips_y: 0.13,
            lie: FRAC_PI_2,
            head: (v.pitch * 0.3).clamp(-0.4, 0.4),
            sh: [0.12; 2],
            el: [0.2; 2],
            ..default()
        },
        Stance::Stand if !v.grounded => Pose {
            hips_y: HIPS_Y,
            spine: -0.1,
            head,
            sh: [-0.4, 0.5],
            el: [0.7; 2],
            hip: [0.8, 0.15],
            kn: [-1.2, -0.4],
            ..default()
        },
        Stance::Stand => {
            let run = ((speed - WALK_SPEED) / (RUN_SPEED - WALK_SPEED)).clamp(0.0, 1.0);
            let moving = (speed / WALK_SPEED).min(1.0);
            let (s, c) = v.phase.sin_cos();
            let leg = moving * (0.42 + 0.45 * run);
            let arm = moving * (0.3 + 0.55 * run);
            // le genou plie quand la jambe revient vers l'avant
            let knee = |c: f32| -moving * (0.08 + (0.45 + 1.1 * run) * c.max(0.0)) - v.dip * 3.0;
            let breathe = (t * 2.0).sin() * 0.012 * (1.0 - moving);
            Pose {
                hips_y: HIPS_Y - v.dip - moving * (0.015 + 0.045 * run) * (0.5 + 0.5 * (2.0 * v.phase).cos()),
                spine: -0.03 - 0.22 * run * moving + breathe,
                head,
                sh: [-arm * s, arm * s],
                el: [0.15 + 1.25 * run * moving; 2],
                hip: [leg * s + v.dip * 1.5, -leg * s + v.dip * 1.5],
                kn: [knee(c), knee(-c)],
                ..default()
            }
        }
    }
}

impl Visitor {
    /// En 1re personne (ou caméra collée au personnage, ou en voiture) on ne se voit pas.
    fn avatar_visible(&self) -> bool {
        self.active && self.third && self.boom_cur > 0.6 && self.car.is_none()
    }
}

/// Pose une articulation d'après la pose courante.
fn apply_joint(p: &Pose, j: Joint, t: &mut Transform) {
    let rx = Quat::from_rotation_x;
    match j {
        Joint::Hips => *t = hips_tf(p),
        Joint::Spine => t.rotation = rx(p.spine),
        Joint::Head => t.rotation = rx(p.head),
        Joint::Shoulder(i) => t.rotation = rx(p.sh[i]),
        Joint::Elbow(i) => t.rotation = rx(p.el[i]),
        Joint::Hip(i) => t.rotation = rx(p.hip[i]),
        Joint::Knee(i) => t.rotation = rx(p.kn[i]),
    }
}

/// Applique la pose aux articulations et place / montre le personnage.
pub fn animate_avatar(
    v: Res<Visitor>,
    mut root: Single<(&mut Transform, &mut Visibility), (With<AvatarRoot>, Without<MainCamera>)>,
    mut joints: Query<(&Joint, &mut Transform), (Without<AvatarRoot>, Without<MainCamera>)>,
) {
    let (tf, vis) = &mut *root;
    **tf = root_tf(&v);
    **vis = if v.avatar_visible() { Visibility::Visible } else { Visibility::Hidden };
    for (j, mut t) in &mut joints {
        apply_joint(&v.pose, *j, &mut t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stance_has_its_pose() {
        let mut v = Visitor { grounded: true, ..default() };
        let standing = target_pose(&v, 0.0, 0.0);
        assert!((standing.hips_y - HIPS_Y).abs() < 0.05);
        let running = target_pose(&v, RUN_SPEED, 0.0);
        assert!(running.spine < standing.spine, "on se penche en courant");
        v.grounded = false;
        assert!(target_pose(&v, 0.0, 0.0).kn[0] < -1.0, "jambes repliées en l air");
        for (stance, low) in [(Stance::Sit, 0.2), (Stance::SitFloor, 0.2), (Stance::Lie, 0.2)] {
            v.stance = stance;
            assert!(target_pose(&v, 0.0, 0.0).hips_y < low, "{stance:?}");
        }
        assert!((target_pose(&v, 0.0, 0.0).lie - FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn joints_follow_the_pose_and_avatar_hides_in_first_person() {
        let pose = Pose {
            hips_y: 0.5,
            spine: 0.1,
            head: 0.2,
            sh: [0.3; 2],
            el: [0.4; 2],
            hip: [0.5; 2],
            kn: [0.6; 2],
            lie: 0.0,
        };
        let joints = [
            Joint::Hips,
            Joint::Spine,
            Joint::Head,
            Joint::Shoulder(0),
            Joint::Elbow(1),
            Joint::Hip(0),
            Joint::Knee(1),
        ];
        let angles: Vec<f32> = joints
            .into_iter()
            .map(|j| {
                let mut t = Transform::default();
                apply_joint(&pose, j, &mut t);
                t.rotation.to_axis_angle().1
            })
            .collect();
        assert!((angles[1] - 0.1).abs() < 1e-5 && (angles[6] - 0.6).abs() < 1e-5, "{angles:?}");
        let mut hips = Transform::default();
        apply_joint(&pose, Joint::Hips, &mut hips);
        assert_eq!(hips.translation.y, 0.5);

        let mut v = Visitor { active: true, third: true, ..default() };
        assert!(v.avatar_visible());
        v.third = false;
        assert!(!v.avatar_visible());
    }

    /// Le personnage construit a chaque articulation une fois, et toutes suivent la pose.
    #[test]
    fn spawned_avatar_has_every_joint_and_follows_the_visitor() {
        let pose = Pose {
            hips_y: 0.9,
            lie: 0.0,
            spine: 0.1,
            head: 0.2,
            sh: [0.3, 0.35],
            el: [0.4, 0.45],
            hip: [0.5, 0.55],
            kn: [0.6, 0.65],
        };
        let v = Visitor { active: true, third: true, boom_cur: 1.0, root: Vec3::new(1.0, 0.0, 2.0), pose, ..default() };
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(v)
            .add_systems(Startup, spawn_avatar)
            .add_systems(Update, animate_avatar);
        app.update();

        let world = app.world_mut();
        let mut angles: Vec<f32> =
            world.query::<(&Joint, &Transform)>().iter(world).map(|(_, t)| t.rotation.to_axis_angle().1).collect();
        angles.sort_by(f32::total_cmp);
        let want = [0.0, 0.1, 0.2, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6, 0.65];
        assert_eq!(angles.len(), want.len(), "{angles:?}");
        assert!(angles.iter().zip(want).all(|(a, w)| (a - w).abs() < 1e-5), "{angles:?}");
        let (tf, vis) = world.query_filtered::<(&Transform, &Visibility), With<AvatarRoot>>().single(world).unwrap();
        assert_eq!((tf.translation, *vis), (Vec3::new(1.0, 0.0, 2.0), Visibility::Visible));

        app.world_mut().resource_mut::<Visitor>().third = false;
        app.update();
        let world = app.world_mut();
        let vis = world.query_filtered::<&Visibility, With<AvatarRoot>>().single(world).unwrap();
        assert_eq!(*vis, Visibility::Hidden, "caché en première personne");
    }
}
