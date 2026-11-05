//! Mode ligne de commande pour piloter le monde sans souris (agents, scripts) :
//!
//! ```text
//! maison3d apply <monde.json> <commandes.json> [--new] [--quiet]
//!                                       applique une liste de commandes (tout ou rien)
//! maison3d describe <monde.json>        résumé texte du monde
//! maison3d check <monde.json>           avertissements de cohérence
//! maison3d schema [--brief]             schéma JSON des commandes (ou résumé d'une ligne par commande)
//! maison3d help <op>                    détail d'une commande et de ses champs
//! maison3d textures                     indices des matières
//! maison3d catalog                      objets, ouvertures et toits avec leurs dimensions
//! maison3d guide                        mode d'emploi pour un agent (docs/guide-agent.md)
//! maison3d shot <monde.json> <image.png> [--level N] [--from so|se|no|ne] [--zone x0,z0,x1,z1]
//!                                        [--size LxH] [--only 3/4|plan]
//! ```

use std::fmt::Write;

use bevy::math::Vec3;

use crate::command::{Command, EditError};
use crate::model::{ElemRef, House, OpeningKind, PropKind, RoofKind, WorldError};

mod describe;
mod shot;

pub use describe::describe;

/// Traite une sous-commande ; `None` s'il n'y en a pas (lancer l'éditeur).
pub fn cli() -> Option<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return None;
    }
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    Some(match run(&args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            1
        }
    })
}

#[derive(Debug)]
enum CliError {
    Usage,
    BadOption(String),
    NoWorld(String),
    UnknownOp(String),
    World(WorldError),
    Commands { path: String, err: WorldError },
    Refused { index: usize, cmd: String, err: EditError },
}

const USAGE: &str = "usage : maison3d apply <monde.json> <commandes.json> [--new] [--quiet] | describe <monde.json> | \
check <monde.json> | schema [--brief] | help <op> | textures | catalog | guide | \
shot <monde.json> <image.png> [--level N] [--from so|se|no|ne] [--zone x0,z0,x1,z1] [--size LxH] [--only 3/4|plan]";

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Usage => write!(f, "{USAGE}"),
            Self::BadOption(o) => write!(f, "option invalide : {o}\n{USAGE}"),
            Self::NoWorld(p) => write!(f, "monde introuvable : {p} (ajouter --new pour le créer)"),
            Self::UnknownOp(op) => write!(f, "commande inconnue : {op} (voir `maison3d schema --brief`)"),
            Self::World(e) => write!(f, "{e}"),
            Self::Commands { path, err } => write!(f, "{path} : {err}"),
            Self::Refused { index, cmd, err } => write!(f, "#{index} refusée, monde inchangé : {err}\n  {cmd}"),
        }
    }
}

impl From<WorldError> for CliError {
    fn from(e: WorldError) -> Self {
        Self::World(e)
    }
}

/// Options sans valeur ; toutes les autres en prennent une.
const SWITCHES: [&str; 3] = ["--new", "--quiet", "--brief"];

/// Sépare les arguments positionnels des options `--nom [valeur]`.
fn split<'a>(args: &[&'a str]) -> (Vec<&'a str>, Vec<(&'a str, Option<&'a str>)>) {
    let (mut pos, mut flags) = (vec![], vec![]);
    let mut it = args.iter().copied();
    while let Some(a) = it.next() {
        if !a.starts_with("--") {
            pos.push(a);
        } else if SWITCHES.contains(&a) {
            flags.push((a, None));
        } else {
            flags.push((a, it.next()));
        }
    }
    (pos, flags)
}

fn run(args: &[&str]) -> Result<(), CliError> {
    let (pos, flags) = split(args);
    let has = |f: &str| flags.iter().any(|(n, _)| *n == f);
    let only = |allowed: &[&str]| match flags.iter().find(|(n, _)| !allowed.contains(n)) {
        Some((n, _)) => Err(CliError::BadOption((*n).to_owned())),
        None => Ok(()),
    };
    match pos.as_slice() {
        ["apply", world, cmds] => {
            only(&["--new", "--quiet"])?;
            apply(world, cmds, has("--new"), has("--quiet"))
        }
        ["shot", world, png] => {
            let o = shot::ShotOpts::parse(&flags)?;
            shot::shot(House::load(world)?, png, &o);
            Ok(())
        }
        ["schema"] => {
            only(&["--brief"])?;
            println!("{}", if has("--brief") { schema_brief() } else { schema_text() });
            Ok(())
        }
        rest => {
            only(&[])?;
            run_simple(rest)
        }
    }
}

/// Sous-commandes sans option.
fn run_simple(args: &[&str]) -> Result<(), CliError> {
    match args {
        ["describe", world] => print!("{}", describe(&House::load(world)?)),
        ["check", world] => print!("{}", warnings_text(&House::load(world)?.warnings(), usize::MAX)),
        ["help", op] => print!("{}", help(op).ok_or_else(|| CliError::UnknownOp((*op).to_owned()))?),
        ["textures"] => {
            println!("0\taucune");
            for (i, t) in crate::textures::catalog().iter().enumerate() {
                println!("{}\t{}", i + 1, t.name);
            }
        }
        ["catalog"] => println!("{}", serde_json::to_string_pretty(&catalog()).map_err(WorldError::from)?),
        ["guide"] => print!("{}", include_str!("../../docs/guide-agent.md")),
        _ => return Err(CliError::Usage),
    }
    Ok(())
}

/// Applique toutes les commandes ou aucune. Un monde absent n'est créé qu'avec `--new`.
fn apply(world: &str, cmds: &str, new: bool, quiet: bool) -> Result<(), CliError> {
    let exists = std::path::Path::new(world).exists();
    if !exists && !new {
        return Err(CliError::NoWorld(world.into()));
    }
    let mut h = if exists { House::load(world)? } else { House::default() };
    let bad = |err: WorldError| CliError::Commands { path: cmds.into(), err };
    let text = std::fs::read_to_string(cmds).map_err(|e| bad(e.into()))?;
    let list: Vec<Command> = serde_json::from_str(&text).map_err(|e| bad(e.into()))?;
    for (i, c) in list.into_iter().enumerate() {
        let what = serde_json::to_string(&c).unwrap_or_else(|_| format!("{c:?}"));
        let before = Counts::of(&h);
        match h.apply(c) {
            Ok(e) => println!("#{i} ok{}", changes(&before, &Counts::of(&h), e)),
            Err(err) => return Err(CliError::Refused { index: i, cmd: what, err }),
        }
    }
    h.save(world)?;
    print!("{}", warnings_text(&h.warnings(), 30));
    if !quiet {
        print!("\n{}", describe(&h));
    }
    Ok(())
}

/// Taille de chaque liste du monde, pour dire ce qu'une commande a créé ou supprimé.
#[derive(PartialEq)]
struct Counts([(&'static str, usize); 6]);

impl Counts {
    fn of(h: &House) -> Self {
        Self([
            ("Wall", h.walls.len()),
            ("Slab", h.slabs.len()),
            ("Roof", h.roofs.len()),
            ("Prop", h.props.len()),
            ("Pool", h.pools.len()),
            ("Road", h.roads.len()),
        ])
    }
}

/// ` → créé : Wall 4 à 7, Slab 1` ; ` → supprimé : …` ; ` → Prop 3` pour une modification.
fn changes(before: &Counts, after: &Counts, result: Option<ElemRef>) -> String {
    let mut parts = vec![];
    for ((name, b), (_, a)) in before.0.iter().zip(&after.0) {
        match a.cmp(b) {
            std::cmp::Ordering::Greater if a - b == 1 => parts.push(format!("créé {name} {b}")),
            std::cmp::Ordering::Greater => parts.push(format!("créés {name} {b} à {}", a - 1)),
            std::cmp::Ordering::Less => parts.push(format!("supprimé, les {name} suivants sont décalés de {}", b - a)),
            std::cmp::Ordering::Equal => {}
        }
    }
    match (parts.is_empty(), result) {
        (true, Some(ElemRef::Opening(w, o))) => format!(" → Opening [{w}, {o}]"),
        (true, Some(e)) => format!(" → {e:?}"),
        (true, None) => String::new(),
        (false, _) => format!(" → {}", parts.join(", ")),
    }
}

// les `let _ = writeln!` de ce fichier écrivent dans une String : ils n'échouent jamais
fn warnings_text(w: &[String], max: usize) -> String {
    if w.is_empty() {
        return "Aucun problème détecté.\n".into();
    }
    let mut s = format!("Avertissements ({}) :\n", w.len());
    for m in w.iter().take(max) {
        let _ = writeln!(s, "  - {m}");
    }
    if w.len() > max {
        let _ = writeln!(s, "  … et {} autres (`maison3d check` pour tout voir)", w.len() - max);
    }
    s
}

fn schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(Vec<Command>)).unwrap_or_default()
}

fn schema_text() -> String {
    serde_json::to_string_pretty(&schemars::schema_for!(Vec<Command>)).unwrap_or_default()
}

fn variants(schema: &serde_json::Value) -> Vec<serde_json::Value> {
    schema["$defs"]["Command"]["oneOf"].as_array().cloned().unwrap_or_default()
}

/// Type lisible d'un champ du schéma : `[x, z]`, `entier`, `PropKind`…
fn type_name(p: &serde_json::Value) -> String {
    if let Some(r) = p["$ref"].as_str() {
        return r.rsplit('/').next().unwrap_or(r).to_owned();
    }
    let ty = |t: &str| p["type"] == t || p["type"].as_array().is_some_and(|a| a.iter().any(|v| v == t));
    match p["maxItems"].as_u64() {
        _ if !ty("array") => {
            let base =
                ["integer", "number", "boolean", "object", "string"].into_iter().find(|t| ty(t)).unwrap_or("valeur");
            match base {
                "integer" => "entier",
                "number" => "nombre",
                "boolean" => "booléen",
                b => b,
            }
            .to_owned()
        }
        Some(2) => "[x, z]".into(),
        Some(3) => "[r, g, b]".into(),
        _ => format!("[{}]", type_name(&p["items"])),
    }
}

/// Une ligne par commande : nom, champs (`?` = facultatif) et première phrase de sa description.
fn schema_brief() -> String {
    let mut s = String::from("Commandes (`maison3d help <op>` pour le détail ; ? = facultatif) :\n");
    for v in variants(&schema()) {
        let op = v["properties"]["op"]["const"].as_str().unwrap_or("?");
        let required: Vec<&str> = v["required"].as_array().into_iter().flatten().filter_map(|r| r.as_str()).collect();
        let fields: Vec<String> = (v["properties"].as_object().into_iter().flatten())
            .filter(|(k, _)| *k != "op")
            .map(|(k, p)| format!("{k}{}: {}", if required.contains(&k.as_str()) { "" } else { "?" }, type_name(p)))
            .collect();
        let doc = v["description"].as_str().unwrap_or("").replace('\n', " ");
        let first = doc.split(". ").next().unwrap_or(&doc);
        let _ = writeln!(s, "\n{op}({})\n    {first}", fields.join(", "));
    }
    s
}

/// Description complète d'une commande et de chacun de ses champs.
fn help(op: &str) -> Option<String> {
    let schema = schema();
    let v = variants(&schema).into_iter().find(|v| v["properties"]["op"]["const"] == op)?;
    let required: Vec<&str> = v["required"].as_array().into_iter().flatten().filter_map(|r| r.as_str()).collect();
    let mut s = format!("{op}\n\n{}\n\nChamps :\n", v["description"].as_str().unwrap_or(""));
    for (k, p) in v["properties"].as_object().into_iter().flatten().filter(|(k, _)| *k != "op") {
        let need = if required.contains(&k.as_str()) { "obligatoire" } else { "facultatif" };
        let doc = p["description"].as_str().map(str::to_owned).or_else(|| {
            let name = p["$ref"].as_str()?.rsplit('/').next()?;
            schema["$defs"][name]["description"].as_str().map(str::to_owned)
        });
        let _ = writeln!(s, "  - {k} ({}, {need}) : {}", type_name(p), doc.unwrap_or_default().replace('\n', " "));
    }
    Some(s)
}

/// Dimensions par défaut de tout ce que les commandes `place_*` savent poser.
fn catalog() -> serde_json::Value {
    // en f64 pour que le JSON affiche 1.6 et non 1.600000023841858
    let r2 = |v: f32| (f64::from(v) * 100.0).round() / 100.0;
    let props: Vec<_> = PropKind::ALL
        .iter()
        .map(|&k| {
            let s: Vec3 = k.size();
            serde_json::json!({
                "kind": k, "name": k.name(), "size": [r2(s.x), r2(s.y), r2(s.z)],
                "outdoor": k.outdoor(), "has_height": k.has_height(),
            })
        })
        .collect();
    let openings: Vec<_> = OpeningKind::ALL
        .iter()
        .map(|&k| {
            let (width, height, sill) = k.default_dims();
            serde_json::json!({ "kind": k, "name": k.name(), "width": r2(width), "height": r2(height), "sill": r2(sill) })
        })
        .collect();
    let roofs: Vec<_> = RoofKind::ALL.iter().map(|&k| serde_json::json!({ "kind": k, "name": k.name() })).collect();
    serde_json::json!({
        "note": "size = [largeur x, profondeur z, hauteur] en mètres à rot 0, avant vers +z ; outdoor = posé sur le relief ; has_height = hauteur réglable (champ height de add_prop)",
        "props": props, "openings": openings, "roofs": roofs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Road, WHITE_TEX, demo_house};
    use bevy::math::Vec2;

    #[test]
    fn catalog_sizes_come_from_geometry() {
        let bed = PropKind::Bed.size();
        assert!((bed.x - 1.6).abs() < 0.05 && (bed.y - 2.0).abs() < 0.1, "lit : {bed}");
        assert!(PropKind::Car.size().y > 4.0, "voiture plus longue que large");
        let c = catalog();
        assert_eq!(c["props"].as_array().map(Vec::len), Some(PropKind::ALL.len()));
        assert_eq!(c["openings"][0]["kind"], "Door");
        assert!(PropKind::ALL.iter().all(|&k| k.size().min_element() > 0.0));
    }

    /// Dossier temporaire propre à chaque test (les tests tournent en parallèle).
    fn tmp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("maison3d_agent_{name}"));
        let _ = std::fs::remove_dir_all(&dir); // absent au premier lancement : sans danger
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn s(p: &std::path::Path) -> &str {
        p.to_str().unwrap()
    }

    #[test]
    fn apply_needs_new_to_create_then_is_all_or_nothing() {
        let dir = tmp("apply");
        let (world, cmds) = (dir.join("monde.json"), dir.join("cmds.json"));
        std::fs::write(&cmds, r#"[{"op": "room", "min": [0, 0], "max": [4, 3], "level": 0}]"#).unwrap();
        assert!(matches!(run(&["apply", s(&world), s(&cmds)]), Err(CliError::NoWorld(_))));
        run(&["apply", s(&world), s(&cmds), "--new", "--quiet"]).unwrap();
        let saved = House::load(&world).unwrap();
        assert_eq!((saved.walls.len(), saved.slabs.len()), (4, 1));
        assert!(!dir.join("monde.json.tmp").exists());

        // la 2e commande échoue : la 1re ne doit pas être enregistrée non plus
        std::fs::write(&cmds, r#"[{"op": "place_prop", "kind": "Bed", "pos": [1, 1], "level": 0}, {"op": "remove", "target": {"Wall": 42}}]"#).unwrap();
        let err = run(&["apply", s(&world), s(&cmds)]).unwrap_err();
        assert!(
            matches!(err, CliError::Refused { index: 1, err: EditError::NotFound(ElemRef::Wall(42), 4), .. }),
            "{err}"
        );
        let text = err.to_string();
        assert!(text.starts_with("#1 refusée") && text.contains(r#"{"op":"remove","target":{"Wall":42}}"#), "{text}");
        assert_eq!(House::load(&world).unwrap(), saved);
        let _ = std::fs::remove_dir_all(dir); // nettoyage best effort
    }

    #[test]
    fn apply_reports_unreadable_inputs() {
        let dir = tmp("errors");
        let (world, cmds) = (dir.join("monde.json"), dir.join("cmds.json"));
        let err = run(&["apply", s(&world), s(&cmds), "--new"]).unwrap_err();
        assert!(matches!(err, CliError::Commands { err: WorldError::Io(_), .. }));
        assert!(err.to_string().contains("cmds.json"));
        std::fs::write(&cmds, r#"[{"op": "voler"}]"#).unwrap();
        assert!(matches!(
            run(&["apply", s(&world), s(&cmds), "--new"]),
            Err(CliError::Commands { err: WorldError::Json(_), .. })
        ));
        std::fs::write(&world, "pas du json").unwrap();
        std::fs::write(&cmds, "[]").unwrap();
        assert!(matches!(run(&["apply", s(&world), s(&cmds)]), Err(CliError::World(WorldError::Json(_)))));
        let _ = std::fs::remove_dir_all(dir); // nettoyage best effort
    }

    #[test]
    fn subcommands_options_and_usage() {
        assert!(run(&["schema"]).is_ok() && run(&["schema", "--brief"]).is_ok());
        assert!(run(&["textures"]).is_ok() && run(&["help", "room"]).is_ok());
        assert!(matches!(run(&["help", "voler"]), Err(CliError::UnknownOp(_))));
        assert!(matches!(run(&["describe", "/nulle/part.json"]), Err(CliError::World(WorldError::Io(_)))));
        assert!(matches!(run(&["schema", "--new"]), Err(CliError::BadOption(_))));
        assert!(matches!(run(&["shot", "a.json", "b.png", "--from", "haut"]), Err(CliError::BadOption(_))));
        for bad in [&["inconnu"][..], &["apply", "un"], &["shot", "a"]] {
            let err = run(bad).unwrap_err();
            assert!(matches!(err, CliError::Usage), "{bad:?}");
            assert!(err.to_string().starts_with("usage"));
        }
    }

    #[test]
    fn brief_and_help_come_from_the_schema() {
        let brief = schema_brief();
        assert!(brief.contains("room(min: [x, z], max: [x, z], level: entier, height?: nombre"), "{brief}");
        assert!(brief.contains("place_row(") && brief.contains("set(target: ElemRef, fields: object)"), "{brief}");
        let room = help("place_prop").unwrap();
        assert!(
            room.contains("- rot (nombre, facultatif)") && room.contains("- kind (PropKind, obligatoire)"),
            "{room}"
        );
    }

    #[test]
    fn changes_name_created_removed_and_modified_elements() {
        let mut h = House::default();
        let before = Counts::of(&h);
        let room = Command::Room {
            min: Vec2::ZERO,
            max: Vec2::splat(4.0),
            level: 0,
            height: None,
            thick: None,
            wall_texture: None,
            floor_texture: None,
        };
        let e = h.apply(room).unwrap();
        assert_eq!(changes(&before, &Counts::of(&h), e), " → créés Wall 0 à 3, créé Slab 0");
        let before = Counts::of(&h);
        let e = h.apply(Command::Remove { target: ElemRef::Wall(1) }).unwrap();
        assert_eq!(changes(&before, &Counts::of(&h), e), " → supprimé, les Wall suivants sont décalés de 1");
        let before = Counts::of(&h);
        let e = h.apply(Command::PlaceOpening { wall: 0, kind: OpeningKind::Window, t: 2.0 }).unwrap();
        assert_eq!(changes(&before, &Counts::of(&h), e), " → Opening [0, 0]");
    }

    #[test]
    fn warnings_are_listed_and_capped() {
        assert_eq!(warnings_text(&[], 3), "Aucun problème détecté.\n");
        let w: Vec<String> = (0..5).map(|i| format!("p{i}")).collect();
        let text = warnings_text(&w, 3);
        assert!(text.starts_with("Avertissements (5)") && text.contains("p2") && !text.contains("p3"), "{text}");
        assert!(text.contains("2 autres"));
    }

    #[test]
    fn describe_lists_references_by_level() {
        let text = describe(&demo_house());
        for needle in [
            "Emprise",
            "Rez-de-chaussée",
            "Wall 0",
            "Opening [0, 0] : Door (Porte)",
            "Slab 0",
            "Roof 0",
            "Prop 0",
            "emprise",
            "Extérieur",
            "Pool 0",
            "Road 0",
            "Relief",
        ] {
            assert!(text.contains(needle), "« {needle} » absent de :\n{text}");
        }
        assert_eq!(describe(&House::default()), "Monde vide.\n");
    }

    #[test]
    fn bounds_cover_every_element() {
        let mut h = House::default();
        h.apply(Command::PlaceProp { kind: PropKind::Lamp, pos: Vec2::new(-3.0, 2.0), level: 0, rot: 0.0 }).unwrap();
        h.apply(Command::AddRoad(Road {
            a: Vec2::new(5.0, -1.0),
            b: Vec2::new(5.0, 7.0),
            width: 3.0,
            color: WHITE_TEX,
            texture: 0,
        }))
        .unwrap();
        assert_eq!(describe::bounds(&h), Some((Vec2::new(-3.0, -1.0), Vec2::new(5.0, 7.0))));
    }
}
