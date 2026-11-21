//! Questions qu'un agent pose sur le monde : quelles pièces, ce que contient l'une d'elles, ce
//! qu'il y a autour d'un point. Une pièce est l'espace fermé par les murs d'un étage, cloisons
//! comprises : on la trouve en remplissant une grille de 10 cm depuis un point. Les réponses
//! donnent les faces intérieures des murs et la position des ouvertures, pour ne pas les
//! recalculer à partir des segments.

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

use super::{ElemRef, House, OpeningKind, Wall};

/// Nom d'une pièce : l'espace fermé par les murs autour de `at`, à l'étage `level`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomName {
    pub name: String,
    pub level: i32,
    pub at: Vec2,
}

/// Côté de la pièce où se trouve un mur.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomSide {
    North,
    East,
    South,
    West,
    /// Cloison qui avance dans la pièce, ou mur en biais.
    Inside,
}

impl RoomSide {
    pub fn name(self) -> &'static str {
        match self {
            Self::North => "nord",
            Self::East => "est",
            Self::South => "sud",
            Self::West => "ouest",
            Self::Inside => "dedans",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RoomWall {
    pub wall: usize,
    pub side: RoomSide,
    /// Coordonnée de la face tournée vers la pièce : z pour un mur nord ou sud, x pour est ou
    /// ouest. `None` pour un mur `Inside`.
    pub face: Option<f32>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RoomOpening {
    pub wall: usize,
    pub index: usize,
    pub kind: OpeningKind,
    pub side: RoomSide,
    /// Centre de l'ouverture sur l'axe du mur.
    pub pos: Vec2,
    pub width: f32,
    /// Au ras du sol : une porte (comme dans `check`).
    pub door: bool,
}

/// Une pièce et ce qu'elle contient.
#[derive(Clone, PartialEq, Debug)]
pub struct RoomInfo {
    pub level: i32,
    /// Noms posés dans la pièce ; plus d'un est signalé par `check`.
    pub names: Vec<String>,
    /// Un point à l'intérieur, à reprendre dans `name_room`.
    pub at: Vec2,
    /// Pas fermée par les murs (dehors, ou un mur manque) : l'emprise s'arrête à 1 m des murs.
    pub open: bool,
    /// Surface au sol en m², à 10 cm près.
    pub area: f32,
    /// Rectangle entre les faces intérieures des murs nord, sud, est et ouest.
    pub inner: (Vec2, Vec2),
    pub walls: Vec<RoomWall>,
    pub openings: Vec<RoomOpening>,
    pub props: Vec<usize>,
    /// Surface au sol prise par les meubles (sans tapis, objets muraux ni plafonniers), en m².
    pub occupied: f32,
}

/// Pas de la grille.
const CELL: f32 = 0.1;
/// Case libre.
const FREE: u32 = u32::MAX;

/// Grille d'un étage : chaque case porte le mur qui l'occupe, ou `FREE`.
struct Grid {
    origin: Vec2,
    nx: usize,
    nz: usize,
    owner: Vec<u32>,
}

impl Grid {
    /// Grille des murs de l'étage, avec 1 m de marge ; `None` sans mur.
    fn new(h: &House, level: i32) -> Option<Self> {
        let walls: Vec<(usize, &Wall)> = h.walls.iter().enumerate().filter(|(_, w)| w.level == level).collect();
        let (lo, hi) = walls.iter().fold(None, |acc: Option<(Vec2, Vec2)>, (_, w)| {
            let (a, b) = (w.a.min(w.b), w.a.max(w.b));
            Some(acc.map_or((a, b), |(lo, hi)| (lo.min(a), hi.max(b))))
        })?;
        let origin = lo - 1.0;
        let n = ((hi + 1.0 - origin) / CELL).ceil();
        let mut g = Self { origin, nx: n.x as usize, nz: n.y as usize, owner: vec![] };
        g.owner = vec![FREE; g.nx * g.nz];
        for (i, w) in walls {
            g.draw(i, w);
        }
        Some(g)
    }

    /// Marque les cases que le mur couvre ; au moins 7,5 cm de part et d'autre de l'axe pour
    /// qu'un mur fin ou en biais ne laisse pas passer le remplissage entre deux cases.
    fn draw(&mut self, i: usize, w: &Wall) {
        let r = (w.thick / 2.0).max(CELL * 0.75);
        let (lo, hi) = (self.index(w.a.min(w.b) - r), self.index(w.a.max(w.b) + r));
        for z in lo.1..=hi.1 {
            for x in lo.0..=hi.0 {
                if segment_distance(self.center(x, z), w.a, w.b) <= r {
                    self.owner[z * self.nx + x] = i as u32;
                }
            }
        }
    }

    /// Case qui contient `p`, ramenée dans la grille.
    fn index(&self, p: Vec2) -> (usize, usize) {
        let c = ((p - self.origin) / CELL).floor();
        (c.x.clamp(0.0, (self.nx - 1) as f32) as usize, c.y.clamp(0.0, (self.nz - 1) as f32) as usize)
    }

    fn cell(&self, p: Vec2) -> Option<usize> {
        let c = ((p - self.origin) / CELL).floor();
        let inside = c.x >= 0.0 && c.y >= 0.0 && (c.x as usize) < self.nx && (c.y as usize) < self.nz;
        inside.then(|| c.y as usize * self.nx + c.x as usize)
    }

    /// Free cell under `p`, `None` in a wall or off the grid: the room a point belongs to.
    fn free_cell(&self, p: Vec2) -> Option<usize> {
        self.cell(p).filter(|&c| self.owner[c] == FREE)
    }

    fn center(&self, x: usize, z: usize) -> Vec2 {
        self.origin + (Vec2::new(x as f32, z as f32) + 0.5) * CELL
    }

    fn neighbours(&self, i: usize) -> impl Iterator<Item = usize> {
        let (x, z, nx) = (i % self.nx, i / self.nx, self.nx);
        [
            (x > 0).then(|| i - 1),
            (x + 1 < nx).then(|| i + 1),
            (z > 0).then(|| i - nx),
            (z + 1 < self.nz).then(|| i + nx),
        ]
        .into_iter()
        .flatten()
    }

    /// Cases libres reliées à `start`, et si elles touchent le bord de la grille.
    fn fill(&self, start: usize, seen: &mut [bool]) -> (Vec<usize>, bool) {
        let (mut cells, mut stack, mut open) = (vec![], vec![start], false);
        seen[start] = true;
        while let Some(i) = stack.pop() {
            cells.push(i);
            let (x, z) = (i % self.nx, i / self.nx);
            open |= x == 0 || z == 0 || x + 1 == self.nx || z + 1 == self.nz;
            for n in self.neighbours(i) {
                if !seen[n] && self.owner[n] == FREE {
                    seen[n] = true;
                    stack.push(n);
                }
            }
        }
        (cells, open)
    }
}

impl House {
    /// Pièces fermées de chaque étage, plus les espaces ouverts qui portent un nom.
    pub fn rooms(&self) -> Vec<RoomInfo> {
        let mut levels: Vec<i32> = self.walls.iter().map(|w| w.level).collect();
        levels.sort_unstable();
        levels.dedup();
        let mut out = vec![];
        for l in levels {
            let Some(g) = Grid::new(self, l) else { continue };
            let mut seen = vec![false; g.owner.len()];
            for start in 0..g.owner.len() {
                if seen[start] || g.owner[start] != FREE {
                    continue;
                }
                let (cells, open) = g.fill(start, &mut seen);
                let room = self.describe_cells(l, &g, &cells, open);
                // les fentes entre deux murs accolés ne sont pas des pièces
                if (!open && room.area >= 1.0) || !room.names.is_empty() {
                    out.push(room);
                }
            }
        }
        out
    }

    /// Pièce de l'étage `level` qui contient `p`, `None` si `p` est dans un mur ou si l'étage
    /// n'a pas de mur.
    pub fn room_at(&self, level: i32, p: Vec2) -> Option<RoomInfo> {
        let g = Grid::new(self, level)?;
        let start = g.free_cell(p)?;
        let (cells, open) = g.fill(start, &mut vec![false; g.owner.len()]);
        Some(self.describe_cells(level, &g, &cells, open))
    }

    /// Le point est-il dans `room`, et pas seulement dans son rectangle `inner` (pièce en L) ?
    /// La grille n'est remplie qu'une fois, pour tester beaucoup de points.
    pub(super) fn in_room(&self, room: &RoomInfo) -> impl Fn(Vec2) -> bool {
        let g = Grid::new(self, room.level);
        let mut seen = vec![false; g.as_ref().map_or(0, |g| g.owner.len())];
        if let Some((g, start)) = g.as_ref().and_then(|g| Some((g, g.cell(room.at)?))) {
            g.fill(start, &mut seen);
        }
        move |p| g.as_ref().and_then(|g| g.cell(p)).is_some_and(|c| seen[c])
    }

    /// Pièce nommée `name`, sans tenir compte de la casse.
    pub fn find_room(&self, name: &str) -> Option<RoomInfo> {
        let n = self.names.iter().find(|n| same_name(&n.name, name))?;
        self.room_at(n.level, n.at)
    }

    fn describe_cells(&self, level: i32, g: &Grid, cells: &[usize], open: bool) -> RoomInfo {
        let mut inside = vec![false; g.owner.len()];
        for &c in cells {
            inside[c] = true;
        }
        let centers: Vec<Vec2> = cells.iter().map(|&c| g.center(c % g.nx, c / g.nx)).collect();
        let (lo, hi) = centers.iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), c| (lo.min(*c), hi.max(*c)));
        let (lo, hi) = (lo - CELL / 2.0, hi + CELL / 2.0);
        let mid = (lo + hi) / 2.0;
        let at = *centers.iter().min_by(|a, b| a.distance(mid).total_cmp(&b.distance(mid))).unwrap_or(&mid);
        // murs qui bordent la pièce sur 30 cm au moins : pas ceux qui la touchent par un bout
        let mut touching: Vec<usize> = (cells.iter())
            .flat_map(|&c| g.neighbours(c))
            .filter_map(|n| (g.owner[n] != FREE).then_some(g.owner[n] as usize))
            .collect();
        touching.sort_unstable();
        let mut ids = touching.clone();
        ids.dedup();
        ids.retain(|i| touching.iter().filter(|j| *j == i).count() >= 3);
        let walls: Vec<RoomWall> = ids
            .into_iter()
            .map(|i| {
                let side = side(&self.walls[i], lo, hi);
                RoomWall { wall: i, side, face: face(&self.walls[i], side) }
            })
            .collect();
        let edge = |side: RoomSide, default: f32, pick: fn(f32, f32) -> f32| {
            walls.iter().filter(|w| w.side == side).filter_map(|w| w.face).reduce(pick).unwrap_or(default)
        };
        let inner = (
            Vec2::new(edge(RoomSide::West, lo.x, f32::max), edge(RoomSide::North, lo.y, f32::max)),
            Vec2::new(edge(RoomSide::East, hi.x, f32::min), edge(RoomSide::South, hi.y, f32::min)),
        );
        let props: Vec<usize> = (self.props.iter().enumerate())
            .filter(|(_, p)| p.level == level && g.cell(p.pos).is_some_and(|c| inside[c]))
            .map(|(i, _)| i)
            .collect();
        let names = (self.name_cells(level, g))
            .filter(|(_, c)| c.is_some_and(|c| inside[c]))
            .map(|(name, _)| name.to_owned())
            .collect();
        RoomInfo {
            level,
            names,
            at,
            open,
            area: cells.len() as f32 * CELL * CELL,
            inner,
            openings: self.openings_of(&walls, lo, hi),
            occupied: props
                .iter()
                .map(|&i| &self.props[i])
                .filter(|p| on_ground(p.kind))
                .map(|p| footprint(&p.corners()))
                .sum(),
            walls,
            props,
        }
    }

    /// Ouvertures des murs de la pièce qui donnent sur elle : un mur commun à deux pièces porte
    /// les fenêtres de l'une et de l'autre.
    fn openings_of(&self, walls: &[RoomWall], lo: Vec2, hi: Vec2) -> Vec<RoomOpening> {
        let mut out = vec![];
        for rw in walls {
            let w = &self.walls[rw.wall];
            let margin = w.thick / 2.0 + CELL;
            for (j, o) in w.openings.iter().enumerate() {
                let pos = w.a + w.dir() * o.t;
                if pos.cmpge(lo - margin).all() && pos.cmple(hi + margin).all() {
                    let (kind, width, door) = (o.kind, o.width, o.is_door());
                    out.push(RoomOpening { wall: rw.wall, index: j, kind, side: rw.side, pos, width, door });
                }
            }
        }
        out
    }

    /// Éléments du `level` à moins de `radius` de `p`, du plus proche au plus loin.
    pub fn near(&self, level: i32, p: Vec2, radius: f32) -> Vec<(ElemRef, f32)> {
        let walls = self.walls.iter().enumerate().filter(|(_, w)| w.level == level);
        let mut hits: Vec<(ElemRef, f32)> = walls
            .clone()
            .map(|(i, w)| (ElemRef::Wall(i), segment_distance(p, w.a, w.b)))
            .chain(walls.flat_map(|(i, w)| {
                w.openings
                    .iter()
                    .enumerate()
                    .map(move |(j, o)| (ElemRef::Opening(i, j), p.distance(w.a + w.dir() * o.t)))
            }))
            .chain(
                (self.props.iter().enumerate())
                    .filter(|(_, q)| q.level == level)
                    .map(|(i, q)| (ElemRef::Prop(i), p.distance(q.pos))),
            )
            .filter(|(_, d)| *d <= radius)
            .collect();
        hits.sort_by(|a, b| a.1.total_cmp(&b.1));
        hits
    }

    /// Avertissements : un nom posé dans un mur, ou plusieurs noms pour une même pièce.
    pub(super) fn name_warnings(&self, out: &mut Vec<String>) {
        let mut levels: Vec<i32> = self.names.iter().map(|n| n.level).collect();
        levels.sort_unstable();
        levels.dedup();
        for l in levels {
            let (lost, rooms) = self.names_by_room(l);
            for name in lost {
                out.push(format!(
                    "Le nom « {name} » est posé dans un mur ou sur un étage sans mur : le reposer (name_room)"
                ));
            }
            for names in rooms {
                if let [first @ .., last] = &names[..]
                    && !first.is_empty()
                {
                    let first: Vec<String> = first.iter().map(|n| format!("« {n} »")).collect();
                    out.push(format!(
                        "{} et « {last} » nomment la même pièce : il manque une cloison ?",
                        first.join(", ")
                    ));
                }
            }
        }
    }

    /// Noms de l'étage `level` posés hors de toute pièce, puis les noms de chaque pièce nommée.
    /// Même découpage que `rooms`, mais une grille par étage et un remplissage par pièce nommée
    /// (pas un par nom), sans décrire les pièces : `check` reste rapide avec beaucoup de noms.
    fn names_by_room(&self, level: i32) -> (Vec<&str>, Vec<Vec<&str>>) {
        let Some(g) = Grid::new(self, level) else {
            return (self.names.iter().filter(|n| n.level == level).map(|n| n.name.as_str()).collect(), vec![]);
        };
        let (mut lost, mut rooms) = (vec![], vec![]);
        let (mut seen, mut room_of) = (vec![false; g.owner.len()], vec![0; g.owner.len()]);
        for (name, start) in self.name_cells(level, &g) {
            let Some(start) = start else {
                lost.push(name);
                continue;
            };
            if !seen[start] {
                for c in g.fill(start, &mut seen).0 {
                    room_of[c] = rooms.len();
                }
                rooms.push(vec![]);
            }
            rooms[room_of[start]].push(name);
        }
        (lost, rooms)
    }

    /// Names of level `level` with the free cell they sit on, `None` in a wall or off the grid.
    /// The single rule for which room holds a name, shared by `describe_cells` and `names_by_room`.
    fn name_cells<'a>(&'a self, level: i32, g: &Grid) -> impl Iterator<Item = (&'a str, Option<usize>)> {
        (self.names.iter()).filter(move |n| n.level == level).map(|n| (n.name.as_str(), g.free_cell(n.at)))
    }
}

/// Côté de la pièce `lo`–`hi` où se trouve le mur.
fn side(w: &Wall, lo: Vec2, hi: Vec2) -> RoomSide {
    let tol = w.thick / 2.0 + CELL * 1.5;
    let pick = |v: f32, min: f32, max: f32, before: RoomSide, after: RoomSide| {
        if v <= min + tol {
            before
        } else if v >= max - tol {
            after
        } else {
            RoomSide::Inside
        }
    };
    if (w.a.y - w.b.y).abs() < 1e-3 {
        pick(w.a.y, lo.y, hi.y, RoomSide::North, RoomSide::South)
    } else if (w.a.x - w.b.x).abs() < 1e-3 {
        pick(w.a.x, lo.x, hi.x, RoomSide::West, RoomSide::East)
    } else {
        RoomSide::Inside
    }
}

fn face(w: &Wall, side: RoomSide) -> Option<f32> {
    let h = w.thick / 2.0;
    match side {
        RoomSide::North => Some(w.a.y + h),
        RoomSide::South => Some(w.a.y - h),
        RoomSide::West => Some(w.a.x + h),
        RoomSide::East => Some(w.a.x - h),
        RoomSide::Inside => None,
    }
}

/// L'objet prend-il de la place au sol ?
pub(super) fn on_ground(k: super::PropKind) -> bool {
    use super::annotations::Placement::{Ceiling, OnTop, OnWall};
    !k.flat() && !k.annotation().is_some_and(|a| matches!(a.placement, OnWall | OnTop | Ceiling))
}

/// Aire du rectangle `c` (coins dans l'ordre du contour).
pub(super) fn footprint(c: &[Vec2; 4]) -> f32 {
    c[0].distance(c[1]) * c[1].distance(c[2])
}

/// Même nom, sans tenir compte de la casse, des accents ni des espaces autour (« Séjour » = « sejour »).
pub fn same_name(a: &str, b: &str) -> bool {
    super::annotations::fold(a.trim()) == super::annotations::fold(b.trim())
}

fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Opening, Prop, PropKind, WHITE, room_walls};

    /// Enveloppe 8 × 4 coupée en x = 5 par une cloison avec porte : cuisine à l'ouest, salon à
    /// l'est. Fenêtre de la cuisine et fenêtre du salon sur le même mur nord.
    fn flat() -> House {
        let mut h =
            House { walls: room_walls(Vec2::ZERO, Vec2::new(8.0, 4.0), 0, 3.0, 0.2, WHITE), ..House::default() };
        h.walls[0].openings.push(Opening::new(OpeningKind::Window, 2.0));
        h.walls[0].openings.push(Opening::new(OpeningKind::Window, 6.5));
        h.walls[2].openings.push(Opening::new(OpeningKind::Door, 1.0));
        let mut partition = Wall { a: Vec2::new(5.0, 0.0), b: Vec2::new(5.0, 4.0), ..h.walls[0].clone() };
        partition.thick = 0.1;
        partition.openings = vec![Opening::new(OpeningKind::Door, 2.0)];
        h.walls.push(partition);
        h.names.push(RoomName { name: "Cuisine".into(), level: 0, at: Vec2::new(2.0, 2.0) });
        h.props.push(Prop::new(PropKind::Fridge, Vec2::new(0.5, 0.5), 0, 0.0));
        h.props.push(Prop::new(PropKind::Rug, Vec2::new(2.0, 2.0), 0, 0.0));
        h.props.push(Prop::new(PropKind::Sofa, Vec2::new(6.5, 2.0), 0, 0.0));
        h
    }

    #[test]
    fn a_partition_splits_the_envelope_into_two_rooms() {
        let h = flat();
        let rooms = h.rooms();
        assert_eq!(rooms.len(), 2, "{rooms:?}");
        let kitchen = rooms.iter().find(|r| r.names == ["Cuisine"]).unwrap();
        assert_eq!(kitchen.inner, (Vec2::splat(0.1), Vec2::new(4.95, 3.9)));
        assert!((kitchen.area - 4.85 * 3.8).abs() < 0.5, "{}", kitchen.area);
        assert_eq!(kitchen.props, [0, 1]);
        let living = rooms.iter().find(|r| r.names.is_empty()).unwrap();
        assert_eq!(living.props, [2]);
        assert!(!kitchen.open && !living.open);
    }

    #[test]
    fn room_lists_sides_faces_and_its_own_openings() {
        let r = flat().find_room(" CUISINE ").unwrap();
        assert_eq!(flat().find_room("cuisiné"), flat().find_room("cuisine"));
        let sides: Vec<(usize, RoomSide)> = r.walls.iter().map(|w| (w.wall, w.side)).collect();
        assert_eq!(sides, [(0, RoomSide::North), (2, RoomSide::South), (3, RoomSide::West), (4, RoomSide::East)]);
        assert_eq!(r.walls[0].face, Some(0.1));
        assert_eq!(r.walls[3].face, Some(4.95));
        // la fenêtre en x = 6.5 est celle du salon
        let ops: Vec<(usize, usize, bool)> = r.openings.iter().map(|o| (o.wall, o.index, o.door)).collect();
        assert_eq!(ops, [(0, 0, false), (4, 0, true)]);
        assert_eq!(r.openings[0].pos, Vec2::new(2.0, 0.0));
        // le tapis ne compte pas dans la place prise
        let fridge = PropKind::Fridge.size();
        assert!((r.occupied - fridge.x * fridge.y).abs() < 1e-3, "{}", r.occupied);
        assert_eq!(flat().find_room("grenier"), None);
    }

    #[test]
    fn outside_is_open_and_walls_are_not_rooms() {
        let h = flat();
        let out = h.room_at(0, Vec2::new(-0.5, -0.5)).unwrap();
        assert!(out.open && out.props.is_empty());
        assert_eq!(h.room_at(0, Vec2::new(0.0, 2.0)), None, "dans le mur ouest");
        assert_eq!(h.room_at(1, Vec2::ONE), None, "étage sans mur");
        assert_eq!(h.room_at(0, Vec2::splat(50.0)), None, "hors de la grille");
    }

    #[test]
    fn a_diagonal_or_jutting_wall_is_inside() {
        let mut h = flat();
        h.walls.push(Wall { a: Vec2::new(3.0, 0.5), b: Vec2::new(4.0, 1.5), ..h.walls[0].clone() });
        h.walls.push(Wall { a: Vec2::new(0.0, 2.0), b: Vec2::new(1.0, 2.0), openings: vec![], ..h.walls[0].clone() });
        let r = h.find_room("cuisine").unwrap();
        let inside: Vec<usize> = r.walls.iter().filter(|w| w.side == RoomSide::Inside).map(|w| w.wall).collect();
        assert_eq!(inside, [5, 6]);
    }

    #[test]
    fn a_wall_touching_by_its_end_does_not_border_the_room() {
        let mut h = flat();
        // cloison du salon, accrochée à la cloison commune en x = 5
        h.walls.push(Wall { a: Vec2::new(5.0, 2.0), b: Vec2::new(8.0, 2.0), openings: vec![], ..h.walls[4].clone() });
        let r = h.find_room("cuisine").unwrap();
        assert!(r.walls.iter().all(|w| w.wall != 5), "{:?}", r.walls);
    }

    #[test]
    fn near_sorts_by_distance_and_stays_on_the_level() {
        let mut h = flat();
        h.props.push(Prop::new(PropKind::Sofa, Vec2::new(0.5, 0.5), 1, 0.0));
        let near = h.near(0, Vec2::new(0.5, 0.5), 0.6);
        assert_eq!(near[0], (ElemRef::Prop(0), 0.0));
        assert!(near.contains(&(ElemRef::Wall(0), 0.5)) && !near.iter().any(|(e, _)| *e == ElemRef::Prop(3)));
        assert!(h.near(0, Vec2::new(2.0, 0.0), 0.1).contains(&(ElemRef::Opening(0, 0), 0.0)));
    }

    #[test]
    fn misplaced_or_doubled_names_are_reported() {
        let mut h = flat();
        h.names.push(RoomName { name: "coin repas".into(), level: 0, at: Vec2::new(3.0, 3.0) });
        h.names.push(RoomName { name: "mur".into(), level: 0, at: Vec2::new(0.0, 2.0) });
        let w = h.warnings();
        assert!(w.iter().any(|m| m.contains("« Cuisine » et « coin repas »")), "{w:?}");
        assert!(w.iter().any(|m| m.contains("« mur » est posé dans un mur")), "{w:?}");
    }

    /// Grande pièce dont le centre tombe dans une petite pièce : le point `at` de la grande pièce
    /// dépend de l'ordre du remplissage, il ne doit pas décider si deux noms la partagent.
    #[test]
    fn two_names_in_one_big_room_are_reported() {
        let mut h =
            House { walls: room_walls(Vec2::splat(-20.0), Vec2::splat(20.0), 0, 3.0, 0.2, WHITE), ..House::default() };
        h.walls.extend(room_walls(Vec2::ZERO, Vec2::new(10.0, 6.0), 0, 3.0, 0.2, WHITE));
        h.names.push(RoomName { name: "jardin".into(), level: 0, at: Vec2::new(-10.0, -10.0) });
        h.names.push(RoomName { name: "cour".into(), level: 0, at: Vec2::new(-9.0, 15.0) });
        let w = h.warnings();
        assert_eq!(w, ["« jardin » et « cour » nomment la même pièce : il manque une cloison ?"]);
    }

    #[test]
    fn three_names_in_one_room_give_one_warning() {
        let mut h = flat();
        h.names.push(RoomName { name: "coin repas".into(), level: 0, at: Vec2::new(3.0, 3.0) });
        h.names.push(RoomName { name: "office".into(), level: 0, at: Vec2::new(1.0, 1.0) });
        h.names.push(RoomName { name: "salon".into(), level: 0, at: Vec2::new(6.5, 2.0) });
        h.names.push(RoomName { name: "grenier".into(), level: 2, at: Vec2::new(6.5, 2.0) });
        h.names.push(RoomName { name: "dehors".into(), level: 0, at: Vec2::new(-30.0, 2.0) });
        let w = h.warnings();
        assert_eq!(
            w,
            [
                "Le nom « dehors » est posé dans un mur ou sur un étage sans mur : le reposer (name_room)",
                "« Cuisine », « coin repas » et « office » nomment la même pièce : il manque une cloison ?",
                "Le nom « grenier » est posé dans un mur ou sur un étage sans mur : le reposer (name_room)",
            ]
        );
    }

    /// `rooms` and `check` must put the same names in the same rooms, and lose the same ones.
    #[test]
    fn rooms_and_name_warnings_group_names_alike() {
        let name = |name: &str, level, at| RoomName { name: name.into(), level, at };
        let mut on_partition = flat();
        on_partition.names.push(name("cloison", 0, Vec2::new(5.0, 2.0)));
        let mut floors = flat();
        floors.walls.extend(room_walls(Vec2::ZERO, Vec2::new(8.0, 4.0), 1, 3.0, 0.2, WHITE));
        floors.names.extend([
            name("chambre", 1, Vec2::new(4.0, 2.0)),
            name("bureau", 1, Vec2::new(1.0, 1.0)),
            name("grenier", 2, Vec2::new(1.0, 1.0)),
            name("coin repas", 0, Vec2::new(3.0, 3.0)),
            name("terrasse", 0, Vec2::new(-0.5, 2.0)),
            name("loin", 0, Vec2::new(-30.0, 2.0)),
        ]);
        let mut big =
            House { walls: room_walls(Vec2::splat(-20.0), Vec2::splat(20.0), 0, 3.0, 0.2, WHITE), ..House::default() };
        big.walls.extend(room_walls(Vec2::ZERO, Vec2::new(10.0, 6.0), 0, 3.0, 0.2, WHITE));
        big.names.extend([
            name("jardin", 0, Vec2::new(-10.0, -10.0)),
            name("cour", 0, Vec2::new(-9.0, 15.0)),
            name("atelier", 0, Vec2::new(5.0, 3.0)),
        ]);
        for h in [flat(), on_partition, floors, big] {
            let rooms = h.rooms();
            let mut levels: Vec<i32> = h.names.iter().map(|n| n.level).collect();
            levels.sort_unstable();
            levels.dedup();
            for l in levels {
                let (lost, mut grouped) = h.names_by_room(l);
                let mut from_rooms: Vec<Vec<&str>> = (rooms.iter())
                    .filter(|r| r.level == l && !r.names.is_empty())
                    .map(|r| r.names.iter().map(String::as_str).collect())
                    .collect();
                grouped.sort();
                from_rooms.sort();
                assert_eq!(grouped, from_rooms, "level {l}");
                let placed = from_rooms.concat();
                let unplaced: Vec<&str> = (h.names.iter())
                    .filter(|n| n.level == l && !placed.contains(&n.name.as_str()))
                    .map(|n| n.name.as_str())
                    .collect();
                assert_eq!(lost, unplaced, "level {l}");
            }
        }
    }
}
