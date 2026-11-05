//! `maison3d query` : réponses courtes sur une pièce ou un point, pour ne pas relire tout `describe`.

use std::fmt::Write;

use bevy::math::Vec2;

use super::CliError;
use super::describe::pt;
use crate::model::{ElemRef, House, RoomInfo, RoomSide, same_name};

// les `let _ = writeln!` de ce fichier écrivent dans une String : ils n'échouent jamais

/// Une ligne par pièce : nom, étage, point intérieur, surface, objets et portes.
pub(super) fn rooms(h: &House) -> String {
    let mut s = String::new();
    for r in h.rooms() {
        let doors = r.openings.iter().filter(|o| o.door).count();
        let _ = writeln!(
            s,
            "{}, level {} : autour de {}, {:.1} m², {} objet(s), {doors} porte(s){}",
            title(&r),
            r.level,
            pt(r.at),
            r.area,
            r.props.len(),
            if r.open { ", ouverte" } else { "" },
        );
    }
    if s.is_empty() { "Aucune pièce : aucun mur ne ferme d'espace.\n".into() } else { s }
}

/// Murs par côté avec leur face intérieure, ouvertures, objets posés et place prise au sol.
/// `key` est un nom de pièce ou un point « x,z » de l'étage `--level`.
pub(super) fn room(h: &House, key: &str, flags: &[(&str, Option<&str>)]) -> Result<String, CliError> {
    let (level, _) = options(flags)?;
    let r = find(h, key, level)?;
    let mut s = String::new();
    let (lo, hi) = r.inner;
    let _ = writeln!(
        s,
        "{}, level {}{} : {:.1} m², meubles {:.1} m²\nentre les murs : {} → {} ({:.2} × {:.2} m)",
        title(&r),
        r.level,
        if r.open { ", ouverte (un mur manque ou dehors)" } else { "" },
        r.area,
        r.occupied,
        pt(lo),
        pt(hi),
        hi.x - lo.x,
        hi.y - lo.y,
    );
    s.push_str("Murs :\n");
    for w in &r.walls {
        let wall = &h.walls[w.wall];
        let face = match w.face {
            // arrondi comme `pt`, pour que la face et le rectangle intérieur affichent la même valeur
            Some(f) => format!(
                "face intérieure {} = {}",
                if matches!(w.side, RoomSide::North | RoomSide::South) { "z" } else { "x" },
                (f * 100.0).round() / 100.0
            ),
            None => format!("{} → {}", pt(wall.a), pt(wall.b)),
        };
        let _ = writeln!(s, "  {:<6} Wall {} : {face}, ép {} m", w.side.name(), w.wall, wall.thick);
    }
    if !r.openings.is_empty() {
        s.push_str("Ouvertures :\n");
    }
    for o in &r.openings {
        let _ = writeln!(
            s,
            "  Opening [{}, {}] : {:?} ({}) mur {}, centre {}, larg {} m",
            o.wall,
            o.index,
            o.kind,
            o.kind.name(),
            o.side.name(),
            pt(o.pos),
            o.width
        );
    }
    let _ = writeln!(s, "Objets ({}) :", r.props.len());
    for &i in &r.props {
        let p = &h.props[i];
        let _ = writeln!(s, "  Prop {i} : {:?} ({}) en {}, rot {}°", p.kind, p.kind.name(), pt(p.pos), p.rot);
    }
    Ok(s)
}

/// Éléments autour du point `at` (« x,z »), du plus proche au plus loin.
pub(super) fn near(h: &House, at: &str, flags: &[(&str, Option<&str>)]) -> Result<String, CliError> {
    let p = point(at).ok_or_else(|| CliError::BadOption(format!("point {at} (attendu x,z)")))?;
    let (level, radius) = options(flags)?;
    let mut s = String::new();
    if let Some(r) = h.room_at(level, p) {
        let _ = writeln!(s, "dans : {}", title(&r));
    }
    for (e, d) in h.near(level, p, radius) {
        let what = match e {
            ElemRef::Wall(i) => format!("{} → {}", pt(h.walls[i].a), pt(h.walls[i].b)),
            ElemRef::Opening(w, o) => format!("{:?}", h.walls[w].openings[o].kind),
            ElemRef::Prop(i) => format!("{:?} en {}", h.props[i].kind, pt(h.props[i].pos)),
            _ => String::new(),
        };
        let _ = writeln!(s, "  {d:.2} m  {e:?} : {what}");
    }
    Ok(if s.is_empty() { format!("Rien à moins de {radius} m de {} au level {level}.\n", pt(p)) } else { s })
}

/// `--level` (0 par défaut) et `--radius` (2 m par défaut).
fn options(flags: &[(&str, Option<&str>)]) -> Result<(i32, f32), CliError> {
    let bad = |what: String| CliError::BadOption(what);
    let (mut level, mut radius) = (0, 2.0);
    for &(flag, v) in flags {
        let v = v.ok_or_else(|| bad(format!("{flag} sans valeur")))?;
        match flag {
            "--level" => level = v.parse().map_err(|_| bad(format!("--level {v}")))?,
            "--radius" => radius = v.parse().map_err(|_| bad(format!("--radius {v}")))?,
            _ => return Err(bad(flag.to_owned())),
        }
    }
    Ok((level, radius))
}

/// Pièce désignée par un nom ou par un point « x,z » de l'étage `level`, avec un message qui
/// dit pourquoi elle n'est pas trouvée.
pub(super) fn find(h: &House, key: &str, level: i32) -> Result<RoomInfo, CliError> {
    if let Some(p) = point(key) {
        return h.room_at(level, p).ok_or_else(|| {
            CliError::NoRoom(format!("en {} au level {level} : point dans un mur, ou étage sans mur", pt(p)))
        });
    }
    if let Some(r) = h.find_room(key) {
        return Ok(r);
    }
    if let Some(n) = h.names.iter().find(|n| same_name(&n.name, key)) {
        return Err(CliError::NoRoom(format!(
            "« {} » : son point {} est dans un mur, le reposer avec name_room",
            n.name,
            pt(n.at)
        )));
    }
    let known: Vec<&str> = h.names.iter().map(|n| n.name.as_str()).collect();
    Err(CliError::UnknownRoom(key.to_owned(), known.join(", ")))
}

/// « x,z » → point.
fn point(s: &str) -> Option<Vec2> {
    match s.split(',').map(|v| v.trim().parse::<f32>()).collect::<Result<Vec<_>, _>>().as_deref() {
        Ok([x, z]) => Some(Vec2::new(*x, *z)),
        _ => None,
    }
}

fn title(r: &RoomInfo) -> String {
    if r.names.is_empty() {
        return "Pièce sans nom".into();
    }
    r.names.iter().map(|n| format!("« {n} »")).collect::<Vec<_>>().join(" et ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{RoomName, demo_house};

    fn named_demo() -> House {
        let mut h = demo_house();
        h.names.push(RoomName { name: "séjour".into(), level: 0, at: Vec2::new(5.0, 4.0) });
        h
    }

    #[test]
    fn rooms_room_and_near_describe_the_demo() {
        let h = named_demo();
        let list = rooms(&h);
        assert!(list.contains("« séjour », level 0 : autour de"), "{list}");
        let r = room(&h, "SÉJOUR", &[]).unwrap();
        assert!(r.contains("nord   Wall 0 : face intérieure z = 0.13, ép 0.25 m"), "{r}");
        assert!(r.contains("Ouvertures :") && r.contains("Prop 0 :"), "{r}");
        assert_eq!(room(&h, "5,4", &[]).unwrap(), r);
        let unknown = room(&h, "grenier", &[]).unwrap_err();
        assert!(matches!(unknown, CliError::UnknownRoom(_, _)) && unknown.to_string().contains("séjour"), "{unknown}");
        assert!(matches!(room(&h, "5,4", &[("--level", Some("3"))]), Err(CliError::NoRoom(_))));
        let mut walled = h.clone();
        walled.names.push(RoomName { name: "mur".into(), level: 0, at: Vec2::ZERO });
        assert!(room(&walled, "mur", &[]).unwrap_err().to_string().contains("dans un mur"));
        assert!(rooms(&House::default()).starts_with("Aucune pièce"));
        assert!(crate::agent::describe(&h).contains("Pièce « séjour » : autour de [5, 4]"));
        let n = near(&h, "0,0", &[("--radius", Some("0.5"))]).unwrap();
        assert!(n.contains("Wall(0)"), "{n}");
        assert!(near(&h, "1,1", &[]).unwrap().starts_with("dans : « séjour »"));
        assert!(near(&h, "0,0", &[("--level", Some("5"))]).unwrap().starts_with("Rien"));
        for (at, flags) in [
            ("0", &[][..]),
            ("0,0", &[("--radius", None)]),
            ("0,0", &[("--level", Some("x"))]),
            ("0,0", &[("--zone", Some("1"))]),
        ] {
            assert!(matches!(near(&h, at, flags), Err(CliError::BadOption(_))), "{at} {flags:?}");
        }
    }
}
