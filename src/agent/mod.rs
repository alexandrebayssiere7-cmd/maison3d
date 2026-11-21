//! Mode ligne de commande pour piloter le monde sans souris (agents, scripts) :
//!
//! ```text
//! maison3d apply <monde.json> <commandes.json> [--new] [--quiet] [--dry-run] [--out <autre.json>] [--json]
//!                                       applique une liste de commandes (tout ou rien)
//! maison3d describe <monde.json>        résumé texte du monde
//! maison3d check <monde.json>           avertissements de cohérence
//! maison3d suggest <monde.json> <Sorte> --room <nom|x,z> [--level N] [--facing n|Sorte] [--count N]
//!                                       places proposées pour un objet, en commandes place_prop
//! maison3d query <monde.json> rooms | room <nom|x,z> [--level N] | near x,z [--level N] [--radius R]
//!                                       pièces, contenu d'une pièce, éléments autour d'un point
//! maison3d schema [--brief]             schéma JSON des commandes (ou résumé d'une ligne par commande)
//! maison3d help <op>                    détail d'une commande et de ses champs
//! maison3d textures                     indices des matières
//! maison3d catalog                      objets, ouvertures et toits avec leurs dimensions
//! maison3d search [mots] [--room R] [--placement P] [--limit N]
//!                                       objets qui répondent aux mots, les plus pertinents d'abord
//! maison3d guide                        mode d'emploi pour un agent (docs/guide-agent.md)
//! maison3d shot <monde.json> <image.png> [--level N] [--from so|se|no|ne] [--zone x0,z0,x1,z1]
//!                                        [--size LxH] [--only 3/4|plan]
//! ```

use std::fmt::Write;

use bevy::math::Vec3;

use crate::command::{Command, EditError};
use crate::model::{ElemRef, House, OpeningKind, Placement, PropKind, RoofKind, Room, Slot, WorldError, search};
use options::{choice, split};

mod describe;
mod options;
mod query;
mod shot;
mod suggest;

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
    Commands {
        path: String,
        err: WorldError,
    },
    Refused {
        index: usize,
        cmd: String,
        err: EditError,
    },
    /// Nom cherché et noms connus.
    UnknownRoom(String, String),
    NoRoom(String),
    /// `suggest` dans un espace ouvert : point désigné.
    OpenRoom(String),
    /// Image de `shot` absente après le rendu, avec la cause quand elle est connue.
    NotWritten(String, Option<std::io::Error>),
}

const USAGE: &str = "usage : maison3d apply <monde.json> <commandes.json> [--new] [--quiet] [--dry-run] \
[--out <autre.json>] [--json] | describe <monde.json> | \
check <monde.json> | query <monde.json> rooms|room <nom|x,z> [--level N]|near x,z [--level N] [--radius R] | \
suggest <monde.json> <Sorte> --room <nom|x,z> [--level N] [--facing n|Sorte] [--count N] | schema [--brief] | help <op> | textures | catalog | guide | \
search [mots] [--room R] [--placement P] [--limit N] | shot <monde.json> <image.png> [--level N] [--from so|se|no|ne] [--zone x0,z0,x1,z1] [--size LxH] [--only 3/4|plan]";

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
            Self::UnknownRoom(r, known) if known.is_empty() => {
                write!(
                    f,
                    "pièce inconnue : {r} (aucune pièce nommée : name_room, ou désigner la pièce par un point x,z)"
                )
            }
            Self::UnknownRoom(r, known) => write!(f, "pièce inconnue : {r} (pièces nommées : {known})"),
            Self::NoRoom(why) => write!(f, "aucune pièce {why}"),
            Self::OpenRoom(at) => write!(
                f,
                "{at} n'est pas dans une pièce fermée (dehors, ou un mur manque) : suggest ne propose \
                 qu'à l'intérieur ; fermer la pièce, ou poser l'objet avec place_prop"
            ),
            Self::NotWritten(p, Some(e)) => write!(f, "image non écrite : {p} ({e})"),
            Self::NotWritten(p, None) => write!(f, "image non écrite : {p}"),
        }
    }
}

impl From<WorldError> for CliError {
    fn from(e: WorldError) -> Self {
        Self::World(e)
    }
}

fn run(args: &[&str]) -> Result<(), CliError> {
    let (pos, flags) = split(args)?;
    let has = |f: &str| flags.iter().any(|(n, _)| *n == f);
    let only = |allowed: &[&str]| match flags.iter().find(|(n, _)| !allowed.contains(n)) {
        Some((n, _)) => Err(CliError::BadOption((*n).to_owned())),
        None => Ok(()),
    };
    match pos.as_slice() {
        ["apply", world, cmds] => {
            only(&["--new", "--quiet", "--dry-run", "--out", "--json"])?;
            let out = flags.iter().find(|(n, _)| *n == "--out").and_then(|(_, v)| *v);
            let o = ApplyOpts {
                new: has("--new"),
                quiet: has("--quiet"),
                dry_run: has("--dry-run"),
                json: has("--json"),
                out,
            };
            apply(world, cmds, &o)
        }
        ["shot", world, png] => {
            let o = shot::ShotOpts::parse(&flags)?;
            shot::shot(House::load(world)?, png, &o)
        }
        ["schema"] => {
            only(&["--brief"])?;
            println!("{}", if has("--brief") { schema_brief() } else { schema_text() });
            Ok(())
        }
        ["search", words @ ..] => {
            only(&["--room", "--placement", "--limit"])?;
            print!("{}", search_text(&words.join(" "), &flags)?);
            Ok(())
        }
        ["query", world, "room", name @ ..] if !name.is_empty() => {
            only(&["--level"])?;
            print!("{}", query::room(&House::load(world)?, &name.join(" "), &flags)?);
            Ok(())
        }
        ["suggest", world, kind] => {
            print!("{}", suggest::suggest(&House::load(world)?, kind, &flags)?);
            Ok(())
        }
        ["query", world, "near", at] => {
            print!("{}", query::near(&House::load(world)?, at, &flags)?);
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
        ["query", world, "rooms"] => print!("{}", query::rooms(&House::load(world)?)),
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

/// Options de `apply`.
#[derive(Default)]
struct ApplyOpts<'a> {
    /// Créer le monde s'il est absent.
    new: bool,
    /// Ne pas afficher `describe` à la fin.
    quiet: bool,
    /// Tout valider et afficher, sans rien écrire.
    dry_run: bool,
    /// Une ligne JSON par commande puis une pour le bilan, au lieu du texte.
    json: bool,
    /// Écrire le résultat ici plutôt que dans le monde lu.
    out: Option<&'a str>,
}

/// Applique toutes les commandes ou aucune. Un monde absent n'est créé qu'avec `--new`.
fn apply(world: &str, cmds: &str, o: &ApplyOpts) -> Result<(), CliError> {
    let mut h = load_or_new(world, o.new)?;
    for (i, c) in read_commands(cmds)?.into_iter().enumerate() {
        let what = serde_json::to_string(&c).unwrap_or_else(|_| format!("{c:?}"));
        let before = Counts::of(&h);
        match h.apply(c) {
            Ok(e) if o.json => println!("{}", step_json(i, &before, &Counts::of(&h), e)),
            Ok(e) => println!("#{i} ok{}", changes(&before, &Counts::of(&h), e)),
            Err(err) => {
                if o.json {
                    println!("{}", serde_json::json!({ "index": i, "ok": false, "error": err.to_string() }));
                }
                return Err(CliError::Refused { index: i, cmd: what, err });
            }
        }
    }
    let saved = if o.dry_run { None } else { Some(o.out.unwrap_or(world)) };
    if let Some(path) = saved {
        h.save(path)?;
    }
    print!("{}", summary(&h, saved, o));
    Ok(())
}

fn load_or_new(world: &str, new: bool) -> Result<House, CliError> {
    match std::path::Path::new(world).exists() {
        true => Ok(House::load(world)?),
        false if new => Ok(House::default()),
        false => Err(CliError::NoWorld(world.into())),
    }
}

fn read_commands(cmds: &str) -> Result<Vec<Command>, CliError> {
    let bad = |err: WorldError| CliError::Commands { path: cmds.into(), err };
    let text = std::fs::read_to_string(cmds).map_err(|e| bad(e.into()))?;
    serde_json::from_str(&text).map_err(|e| bad(e.into()))
}

/// Bilan de fin d'`apply` : avertissements, fichier écrit (ou simulation), puis `describe`.
fn summary(h: &House, saved: Option<&str>, o: &ApplyOpts) -> String {
    if o.json {
        return format!("{}\n", serde_json::json!({ "saved": saved, "warnings": h.warnings() }));
    }
    let mut s = warnings_text(&h.warnings(), 30);
    match saved {
        None => s.push_str("Simulation (--dry-run) : rien n'a été écrit.\n"),
        Some(path) if o.out.is_some() => {
            let _ = writeln!(s, "Écrit dans {path}.");
        }
        Some(_) => {}
    }
    if !o.quiet {
        let _ = write!(s, "\n{}", describe(h));
    }
    s
}

/// Taille de chaque liste du monde, pour dire ce qu'une commande a créé ou supprimé.
struct Counts([(&'static str, fn(usize) -> ElemRef, usize); 6]);

impl Counts {
    fn of(h: &House) -> Self {
        Self([
            ("Wall", ElemRef::Wall, h.walls.len()),
            ("Slab", ElemRef::Slab, h.slabs.len()),
            ("Roof", ElemRef::Roof, h.roofs.len()),
            ("Prop", ElemRef::Prop, h.props.len()),
            ("Pool", ElemRef::Pool, h.pools.len()),
            ("Road", ElemRef::Road, h.roads.len()),
        ])
    }
}

/// `{"index":3,"ok":true,"elem":{"Prop":3},"created":[{"Wall":4},…],"removed":{"Wall":1}}` :
/// les références se recopient telles quelles dans le `target` d'une commande suivante.
fn step_json(index: usize, before: &Counts, after: &Counts, elem: Option<ElemRef>) -> serde_json::Value {
    let (mut created, mut removed) = (vec![], serde_json::Map::new());
    for ((name, mk, b), (_, _, a)) in before.0.iter().zip(&after.0) {
        created.extend((*b..*a).map(mk));
        if a < b {
            removed.insert((*name).to_owned(), (b - a).into());
        }
    }
    serde_json::json!({ "index": index, "ok": true, "elem": elem, "created": created, "removed": removed })
}

/// ` → créé : Wall 4 à 7, Slab 1` ; ` → supprimé : …` ; ` → Prop 3` pour une modification.
fn changes(before: &Counts, after: &Counts, result: Option<ElemRef>) -> String {
    let mut parts = vec![];
    for ((name, _, b), (_, _, a)) in before.0.iter().zip(&after.0) {
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

fn part_names(slots: &[Slot]) -> Vec<&'static str> {
    slots.iter().map(|s| s.name).collect()
}

/// Une ligne par objet trouvé : ce qu'il faut pour le choisir et le poser.
fn search_text(query: &str, flags: &[(&str, Option<&str>)]) -> Result<String, CliError> {
    let room = match flags.iter().find(|(n, _)| *n == "--room") {
        Some((_, v)) => Some(v.and_then(Room::parse).ok_or_else(|| {
            let names: Vec<String> = Room::ALL.iter().map(|r| format!("{r:?}")).collect();
            CliError::BadOption(format!(
                "--room {} (valeurs : {}, ou leur nom français)",
                v.unwrap_or(""),
                names.join(", ")
            ))
        })?),
        None => None,
    };
    let placement = choice(flags, "--placement", &Placement::ALL)?;
    let limit = match flags.iter().find(|(n, _)| *n == "--limit") {
        Some((_, v)) => v
            .and_then(|v| v.parse().ok())
            .filter(|&n| n > 0)
            .ok_or_else(|| CliError::BadOption("--limit N (N ≥ 1)".into()))?,
        None => 10,
    };
    let hits = search(query, room, placement);
    if hits.is_empty() {
        return Ok("aucun objet : essayer un synonyme, un mot anglais, ou parcourir `maison3d catalog`\n".into());
    }
    let mut out = String::new();
    for &k in hits.iter().take(limit) {
        // `search` ne renvoie que des sortes annotées
        let (s, Some(a)) = (k.size(), k.annotation()) else { continue };
        let with: Vec<String> = a.goes_with.iter().map(|k| format!("{k:?}")).collect();
        let _ = writeln!(
            out,
            "{k:?}\t{}\t{:.2} x {:.2} x {:.2} m\t{:?}\tdevant {:.1} m\t{:?}\tavec {}",
            k.name(),
            s.x,
            s.y,
            s.z,
            a.placement,
            a.front,
            a.rooms,
            with.join(", "),
        );
    }
    if hits.len() > limit {
        let _ = writeln!(out, "… {} autres (--limit {})", hits.len() - limit, hits.len());
    }
    Ok(out)
}

/// Dimensions par défaut de tout ce que les commandes `place_*` savent poser.
fn catalog() -> serde_json::Value {
    // en f64 pour que le JSON affiche 1.6 et non 1.600000023841858
    let r2 = |v: f32| (f64::from(v) * 100.0).round() / 100.0;
    let props: Vec<_> = PropKind::ALL
        .iter()
        .map(|&k| {
            let s: Vec3 = k.size();
            let a = k.annotation();
            serde_json::json!({
                "kind": k, "name": k.name(), "category": k.category().name(), "size": [r2(s.x), r2(s.y), r2(s.z)],
                "outdoor": k.outdoor(), "dim": k.dim(), "parts": part_names(k.slots()),
                "placement": a.map(|a| a.placement), "rooms": a.map(|a| a.rooms), "front": a.map(|a| r2(a.front)),
                "goes_with": a.map(|a| a.goes_with), "needs": a.map(|a| a.needs), "synonyms": a.map(|a| a.synonyms),
            })
        })
        .collect();
    let openings: Vec<_> = OpeningKind::ALL
        .iter()
        .map(|&k| {
            let (width, height, sill) = k.default_dims();
            serde_json::json!({
                "kind": k, "name": k.name(), "width": r2(width), "height": r2(height), "sill": r2(sill),
                "parts": part_names(k.slots()),
            })
        })
        .collect();
    let roofs: Vec<_> = RoofKind::ALL.iter().map(|&k| serde_json::json!({ "kind": k, "name": k.name() })).collect();
    serde_json::json!({
        "note": "size = [largeur x, profondeur z, hauteur] en mètres à rot 0, avant vers +z ; outdoor = posé sur le relief ; dim = ce que règle le champ height (hauteur, longueur, rayon), null s'il ne sert pas ; parts = parties colorables, dans l'ordre des indices `part` de paint (0 = color/texture de l'élément) ; placement = Floor (au sol, dégagé), AgainstWall (dos au mur, avant vers la pièce), OnWall (fixé au mur en hauteur), OnTop (posé sur un meuble), Ceiling (au plafond), Row (module à aligner avec place_row) ; rooms = pièces où l'objet a sa place ; front = espace libre à laisser devant (m) ; goes_with = objets posés habituellement avec ; needs = Road (sur une route ou une allée), StairOpening (trémie dans le sol d'arrivée) ; synonyms = mots pour le chercher",
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
        assert_eq!(c["openings"][0]["parts"][2], "Poignée");
        let bed = c["props"].as_array().unwrap().iter().find(|p| p["kind"] == "Bed").unwrap();
        assert_eq!((bed["placement"].as_str(), bed["rooms"][0].as_str()), (Some("AgainstWall"), Some("Bedroom")));
        assert!(bed["synonyms"].as_array().unwrap().iter().any(|s| s == "bed"));
        let sofa = &c["props"][2];
        assert_eq!(
            (&sofa["kind"], &sofa["category"], &sofa["parts"][1]),
            (&"Sofa".into(), &"Salon".into(), &"Pieds".into())
        );
        assert_eq!(
            c["props"].as_array().and_then(|p| p.iter().find(|p| p["kind"] == "Stairs")).map(|s| &s["dim"]),
            Some(&"hauteur".into())
        );
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
    fn dry_run_and_out_leave_the_world_untouched() {
        let dir = tmp("dry_run");
        let (world, cmds, copy) = (dir.join("monde.json"), dir.join("cmds.json"), dir.join("copie.json"));
        std::fs::write(&cmds, r#"[{"op": "room", "min": [0, 0], "max": [4, 3], "level": 0}]"#).unwrap();
        run(&["apply", s(&world), s(&cmds), "--new", "--dry-run"]).unwrap();
        assert!(!world.exists(), "--dry-run n'écrit rien");
        run(&["apply", s(&world), s(&cmds), "--new", "--quiet"]).unwrap();
        let saved = House::load(&world).unwrap();
        run(&["apply", s(&world), s(&cmds), "--out", s(&copy), "--json"]).unwrap();
        assert_eq!(House::load(&world).unwrap(), saved, "--out laisse le monde lu intact");
        assert_eq!(House::load(&copy).unwrap().walls.len(), 8);
        assert!(matches!(run(&["apply", s(&world), s(&cmds), "--out"]), Err(CliError::BadOption(_))));
        let _ = std::fs::remove_dir_all(dir); // nettoyage best effort
    }

    #[test]
    fn summary_says_where_the_world_went() {
        let h = House::default();
        let dry = ApplyOpts { dry_run: true, quiet: true, ..ApplyOpts::default() };
        assert!(summary(&h, None, &dry).contains("--dry-run"));
        let out = ApplyOpts { out: Some("b.json"), quiet: true, ..ApplyOpts::default() };
        assert!(summary(&h, Some("b.json"), &out).contains("Écrit dans b.json"));
        let json = ApplyOpts { json: true, ..ApplyOpts::default() };
        let v: serde_json::Value = serde_json::from_str(&summary(&h, None, &json)).unwrap();
        assert_eq!(v, serde_json::json!({ "saved": null, "warnings": [] }));
    }

    #[test]
    fn step_json_lists_reusable_references() {
        let mut h = House::default();
        let before = Counts::of(&h);
        let e = h
            .apply(Command::PlaceProp { kind: PropKind::Lamp, pos: Vec2::ZERO, level: 0, rot: 0.0, scale: None })
            .unwrap();
        let v = step_json(2, &before, &Counts::of(&h), e);
        assert_eq!(
            v,
            serde_json::json!({ "index": 2, "ok": true, "elem": {"Prop": 0}, "created": [{"Prop": 0}], "removed": {} })
        );
        let before = Counts::of(&h);
        let e = h.apply(Command::Remove { target: ElemRef::Prop(0) }).unwrap();
        assert_eq!(step_json(0, &before, &Counts::of(&h), e)["removed"], serde_json::json!({ "Prop": 1 }));
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
        assert!(run(&["search", "frigo"]).is_ok() && run(&["search", "--room", "Salle de bain"]).is_ok());
        assert_eq!(Room::parse("séjour"), Some(Room::Living));
        assert_eq!(Room::parse("kitchen"), Some(Room::Kitchen));
        assert!(matches!(run(&["search", "x", "--limit", "0"]), Err(CliError::BadOption(_))));
        for bad in [
            &["search", "--room", "cave"][..],
            &["search", "--placement"],
            &["search", "--limit", "x"],
            &["search", "--new"],
        ] {
            assert!(matches!(run(bad), Err(CliError::BadOption(_))), "{bad:?}");
        }
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
            name: None,
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
    fn cli_errors_explain_what_to_do() {
        let world = CliError::World(WorldError::Io(std::io::Error::other("disque plein")));
        let cmds =
            CliError::Commands { path: "c.json".into(), err: WorldError::Io(std::io::Error::other("illisible")) };
        for (err, want) in [
            (CliError::Usage, "usage"),
            (CliError::BadOption("--vite".into()), "option invalide : --vite"),
            (CliError::NoWorld("m.json".into()), "--new"),
            (CliError::UnknownOp("voler".into()), "schema --brief"),
            (world, "disque plein"),
            (cmds, "c.json : illisible"),
            (CliError::OpenRoom("-1,-1".into()), "-1,-1 n'est pas dans une pièce fermée"),
            (CliError::NotWritten("v.png".into(), None), "image non écrite : v.png"),
            (CliError::NotWritten("v.png".into(), Some(std::io::Error::other("absent"))), "v.png (absent)"),
        ] {
            assert!(err.to_string().contains(want), "{err:?} → {err}");
        }
    }

    /// Monde à deux étages : « salon » au level 0, « haut » au level 1 avec un canapé (Prop 0).
    fn two_levels(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let dir = tmp(name);
        let (world, cmds) = (dir.join("monde.json"), dir.join("cmds.json"));
        std::fs::write(
            &cmds,
            r#"[{"op": "room", "min": [0, 0], "max": [4, 3], "level": 0, "name": "salon"},
                {"op": "room", "min": [0, 0], "max": [4, 3], "level": 1, "name": "haut"},
                {"op": "place_prop", "kind": "Sofa", "pos": [2, 1.5], "level": 1}]"#,
        )
        .unwrap();
        run(&["apply", s(&world), s(&cmds), "--new", "--quiet"]).unwrap();
        (dir, world)
    }

    #[test]
    fn usage_offers_radius_to_near_only() {
        let room = USAGE.split("room <nom|x,z>").nth(1).and_then(|r| r.split("near").next()).unwrap_or_default();
        assert!(room.contains("--level") && !room.contains("--radius"), "{room}");
        assert!(USAGE.contains("near x,z [--level N] [--radius R]"));
    }

    #[test]
    fn option_useless_for_the_subcommand_is_refused() {
        let (dir, world) = two_levels("useless_option");
        assert!(run(&["query", s(&world), "room", "salon", "--level", "0"]).is_ok());
        let err = run(&["query", s(&world), "room", "salon", "--radius", "5"]).unwrap_err();
        assert!(matches!(err, CliError::BadOption(_)), "{err}");
        let _ = std::fs::remove_dir_all(dir); // nettoyage best effort
    }

    #[test]
    fn facing_must_be_on_the_room_level() {
        let (dir, world) = two_levels("facing_level");
        let w = s(&world);
        let err = run(&["suggest", w, "Armchair", "--room", "salon", "--facing", "0"]).unwrap_err();
        assert!(matches!(err, CliError::BadOption(_)), "{err}");
        assert!(run(&["suggest", w, "Armchair", "--room", "haut", "--level", "1", "--facing", "0"]).is_ok());
        let _ = std::fs::remove_dir_all(dir); // nettoyage best effort
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
        h.apply(Command::PlaceProp {
            kind: PropKind::Lamp,
            pos: Vec2::new(-3.0, 2.0),
            level: 0,
            rot: 0.0,
            scale: None,
        })
        .unwrap();
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
