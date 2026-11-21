//! `maison3d suggest` : où poser un objet dans une pièce. Chaque ligne est une commande
//! `place_prop` prête à l'emploi, suivie des raisons de son classement ; l'agent choisit.

use std::fmt::Write;

use super::CliError;
use super::query::find;
use crate::model::{House, PropKind};

/// Options : `--room <nom|x,z>` (obligatoire), `--level N`, `--facing <n|Sorte>`, `--count N`.
pub(super) fn suggest(h: &House, kind: &str, flags: &[(&str, Option<&str>)]) -> Result<String, CliError> {
    let kind = PropKind::ALL
        .into_iter()
        .find(|k| format!("{k:?}").eq_ignore_ascii_case(kind))
        .ok_or_else(|| CliError::BadOption(format!("sorte {kind} inconnue (`maison3d search` pour la trouver)")))?;
    let get = |name: &str| flags.iter().find(|(n, _)| *n == name).map(|(_, v)| v.unwrap_or(""));
    if let Some((n, _)) = flags.iter().find(|(n, _)| !["--room", "--level", "--facing", "--count"].contains(n)) {
        return Err(CliError::BadOption((*n).to_owned()));
    }
    let bad = |what: &str, v: &str| CliError::BadOption(format!("{what} {v}"));
    let level = get("--level").map_or(Ok(0), |v| v.parse().map_err(|_| bad("--level", v)))?;
    let count = get("--count").map_or(Ok(5), |v| v.parse().map_err(|_| bad("--count", v)))?;
    let key = get("--room").ok_or_else(|| CliError::BadOption("--room <nom|x,z> manquant".into()))?;
    let room = find(h, key, level)?;
    // ses limites ne sont qu'un rectangle à 1 m des murs : les places proposées tomberaient
    // dans la maison
    if room.open {
        return Err(CliError::OpenRoom(key.to_owned()));
    }
    if count == 0 {
        return Err(bad("--count", "0"));
    }
    let facing = match get("--facing") {
        None => None,
        Some(v) => Some(
            (v.parse::<usize>().ok().filter(|&i| h.props.get(i).is_some_and(|p| p.level == room.level)))
                .or_else(|| {
                    room.props.iter().copied().find(|&i| format!("{:?}", h.props[i].kind).eq_ignore_ascii_case(v))
                })
                .ok_or_else(|| {
                    bad("--facing", &format!("{v} : ni indice d'objet de l'étage ni sorte posée dans la pièce"))
                })?,
        ),
    };
    let found = h.suggest(kind, &room, facing, count);
    if found.is_empty() && kind.outdoor() {
        return Ok(format!("{kind:?} est un objet d'extérieur : le poser dehors avec place_prop.\n"));
    }
    if found.is_empty() {
        return Ok(format!(
            "Aucune place pour {kind:?} dans cette pièce : libérer de la place ou choisir une autre pièce.\n"
        ));
    }
    let mut s = String::new();
    for x in found {
        // écrire dans une String n'échoue jamais
        let _ = writeln!(
            s,
            r#"{{"op":"place_prop","kind":"{kind:?}","pos":[{},{}],"level":{},"rot":{}}}  # {:+.1} : {}"#,
            x.pos.x,
            x.pos.y,
            room.level,
            x.rot,
            x.score,
            x.why.join(", ")
        );
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::model::{RoomName, demo_house};
    use bevy::math::Vec2;

    #[test]
    fn lines_are_ready_to_apply_commands() {
        let mut h = demo_house();
        h.names.push(RoomName { name: "séjour".into(), level: 0, at: Vec2::new(5.0, 4.0) });
        let out =
            suggest(&h, "armchair", &[("--room", Some("séjour")), ("--facing", Some("0")), ("--count", Some("2"))])
                .unwrap();
        assert_eq!(out.lines().count(), 2, "{out}");
        for line in out.lines() {
            let (json, why) = line.split_once("  # ").unwrap();
            let cmd: Command = serde_json::from_str(json).unwrap();
            assert!(why.contains("avant vers Prop 0"), "{line}");
            assert!(h.clone().apply(cmd).is_ok());
        }
        assert!(suggest(&h, "Sofa", &[("--room", Some("5,4")), ("--facing", Some("Sofa"))]).is_ok());
        for flags in [
            &[][..],
            &[("--room", Some("grenier"))],
            &[("--room", Some("5,4")), ("--level", Some("x"))],
            &[("--room", Some("5,4")), ("--facing", Some("Licorne"))],
            &[("--room", Some("5,4")), ("--zone", Some("1"))],
        ] {
            assert!(suggest(&h, "Sofa", flags).is_err(), "{flags:?}");
        }
        assert!(suggest(&h, "Licorne", &[("--room", Some("5,4"))]).is_err());
        assert!(suggest(&h, "Car", &[("--room", Some("5,4"))]).unwrap().contains("objet d'extérieur"));
        assert!(suggest(&h, "Sofa", &[("--room", Some("5,4")), ("--count", Some("0"))]).is_err());
    }

    /// Salon en L : 10 × 6, dont une pièce de 4 × 3 au nord-est est séparée par deux cloisons.
    fn l_shaped() -> House {
        let cmds: Vec<Command> = serde_json::from_str(
            r#"[{"op":"room","min":[0,0],"max":[10,6],"level":0,"name":"salon"},
            {"op":"add_wall","a":[6,0],"b":[6,3],"level":0,"height":3,"thick":0.1,"color":[1,1,1],"openings":[],"texture":0},
            {"op":"add_wall","a":[6,3],"b":[10,3],"level":0,"height":3,"thick":0.1,"color":[1,1,1],"openings":[],"texture":0}]"#,
        )
        .unwrap();
        let mut h = House::default();
        for c in cmds {
            h.apply(c).unwrap();
        }
        h
    }

    #[test]
    fn proposals_in_an_l_shaped_room_stay_in_it_and_pass_check() {
        let h = l_shaped();
        assert_eq!(h.warnings(), Vec::<String>::new());
        for kind in ["Sofa", "Wardrobe", "Mirror", "Lamp", "Bed", "CeilingLamp"] {
            let out = suggest(&h, kind, &[("--room", Some("salon")), ("--count", Some("20"))]).unwrap();
            assert!(out.contains("place_prop"), "{kind} : {out}");
            for line in out.lines() {
                let cmd: Command = serde_json::from_str(line.split_once("  # ").unwrap().0).unwrap();
                let Command::PlaceProp { pos, .. } = cmd else { panic!("{line}") };
                let mut placed = h.clone();
                placed.apply(cmd).unwrap();
                assert_eq!(placed.warnings(), Vec::<String>::new(), "{line}");
                let room = placed.room_at(0, pos);
                assert!(room.is_some_and(|r| r.names == ["salon"]), "hors du salon : {line}");
            }
        }
    }

    #[test]
    fn an_open_room_is_refused() {
        let h = l_shaped();
        let err = suggest(&h, "Car", &[("--room", Some("-0.5,-0.5"))]).unwrap_err();
        assert!(matches!(err, CliError::OpenRoom(_)), "{err}");
        assert!(matches!(suggest(&h, "Sofa", &[("--room", Some("10.5,3"))]), Err(CliError::OpenRoom(_))));
    }
}
