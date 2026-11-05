//! Tests d'intégration du mode ligne de commande : lancent le vrai binaire.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_maison3d")).args(args).output().expect("lancer maison3d")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// Dossier temporaire propre à un test (les tests tournent en parallèle).
fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("maison3d-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir); // absent au premier lancement : sans danger
    std::fs::create_dir_all(&dir).expect("créer le dossier temporaire");
    dir
}

fn write(dir: &Path, file: &str, text: &str) -> String {
    let p = dir.join(file);
    std::fs::write(&p, text).expect("écrire le fichier");
    p.to_str().expect("chemin UTF-8").to_owned()
}

fn read_json(path: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("lire le monde")).expect("monde JSON valide")
}

const ROOM: &str = r#"[{"op": "room", "min": [0, 0], "max": [4, 3], "level": 0}]"#;

/// Crée un monde avec une pièce (4 murs) et renvoie son chemin.
fn world_with_room(dir: &Path) -> String {
    let world = dir.join("monde.json").to_str().expect("chemin UTF-8").to_owned();
    let cmds = write(dir, "room.json", ROOM);
    let o = run(&["apply", &world, &cmds, "--new"]);
    assert!(o.status.success(), "apply room : {}", String::from_utf8_lossy(&o.stderr));
    world
}

#[test]
fn schema_is_valid_json() {
    let o = run(&["schema"]);
    assert!(o.status.success());
    let schema: Value = serde_json::from_str(&stdout(&o)).expect("schéma JSON valide");
    assert!(schema.is_object());
    assert!(stdout(&o).contains("\"room\""), "la commande room figure dans le schéma");
}

#[test]
fn apply_creates_world_and_describe_lists_it() {
    let dir = tmp_dir("apply");
    let world = world_with_room(&dir);
    let json = read_json(&world);
    assert_eq!(json["walls"].as_array().map(Vec::len), Some(4));
    assert_eq!(json["slabs"].as_array().map(Vec::len), Some(1));

    let o = run(&["describe", &world]);
    assert!(o.status.success());
    let text = stdout(&o);
    for i in 0..4 {
        assert!(text.contains(&format!("Wall {i} ")), "mur {i} absent :\n{text}");
    }
    assert!(text.contains("Slab 0"));
}

#[test]
fn refused_command_exits_1_and_leaves_world_untouched() {
    let dir = tmp_dir("refused");
    let world = world_with_room(&dir);
    let before = std::fs::read(&world).expect("lire le monde");
    // la première commande passe, la seconde vise un mur inexistant : tout ou rien
    let cmds = write(
        &dir,
        "bad.json",
        r#"[{"op": "room", "min": [10, 10], "max": [12, 12], "level": 0},
            {"op": "place_opening", "wall": 999, "kind": "Door", "t": 1.0}]"#,
    );
    let o = run(&["apply", &world, &cmds]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stdout(&o).contains("#0 ok"), "le fichier doit être lu, la 2e commande refusée");
    assert_ne!(o.stderr.len(), 0, "message d erreur attendu");
    assert_eq!(std::fs::read(&world).expect("relire le monde"), before);
}

#[test]
fn invalid_json_commands_exit_1() {
    let dir = tmp_dir("badjson");
    let world = world_with_room(&dir);
    let before = std::fs::read(&world).expect("lire le monde");
    let cmds = write(&dir, "bad.json", r#"[{"op": "inconnue"}]"#);
    assert_eq!(run(&["apply", &world, &cmds]).status.code(), Some(1));
    assert_eq!(std::fs::read(&world).expect("relire le monde"), before);
}

#[test]
fn invalid_usage_exits_1() {
    for args in [&["nimporte"][..], &["apply", "un-seul-argument"], &["describe"]] {
        let o = run(args);
        assert_eq!(o.status.code(), Some(1), "{args:?}");
        assert!(!o.stderr.is_empty(), "{args:?} : message d'usage attendu");
    }
}

#[test]
fn describe_missing_world_exits_1() {
    let dir = tmp_dir("missing");
    let missing = dir.join("absent.json");
    assert_eq!(run(&["describe", missing.to_str().expect("chemin UTF-8")]).status.code(), Some(1));
}

/// Les exemples de `docs/exemples/` s'appliquent sans erreur et, ensemble, emploient chaque commande du schéma.
#[test]
fn agent_examples_apply_and_cover_every_command() {
    let dir = tmp_dir("exemples");
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/exemples");
    let mut used = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(&examples).expect("lire docs/exemples") {
        let path = entry.expect("entrée du dossier").path();
        let name = path.file_stem().and_then(|s| s.to_str()).expect("nom UTF-8").to_owned();
        let cmds: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("lire l'exemple")).expect("JSON");
        for c in cmds.as_array().expect("liste de commandes") {
            used.insert(c["op"].as_str().expect("champ op").to_owned());
        }
        let world = dir.join(format!("{name}.json"));
        let o = run(&["apply", world.to_str().expect("chemin UTF-8"), path.to_str().expect("chemin UTF-8"), "--new"]);
        assert!(o.status.success(), "{name} : {}", String::from_utf8_lossy(&o.stderr));
        if name != "retouches" {
            assert!(
                stdout(&o).contains("Aucun problème détecté"),
                "{name} : exemple avec avertissements\n{}",
                stdout(&o)
            );
        }
    }
    let schema: Value = serde_json::from_str(&stdout(&run(&["schema"]))).expect("schéma JSON");
    let ops: Vec<String> = schema["$defs"]["Command"]["oneOf"]
        .as_array()
        .expect("variantes")
        .iter()
        .map(|v| v["properties"]["op"]["const"].as_str().expect("op constant").to_owned())
        .collect();
    let missing: Vec<&String> = ops.iter().filter(|op| !used.contains(*op)).collect();
    assert!(missing.is_empty(), "commandes sans exemple : {missing:?}");
}

/// Chaque commande, champ et type du schéma porte une description : c'est tout ce que l'agent lit.
#[test]
fn schema_documents_every_command_and_field() {
    let schema: Value = serde_json::from_str(&stdout(&run(&["schema"]))).expect("schéma JSON");
    let mut missing = vec![];
    for (name, def) in schema["$defs"].as_object().expect("définitions") {
        if def.get("description").is_none() && def.get("oneOf").is_none() {
            missing.push(name.clone());
        }
        for (i, v) in def["oneOf"].as_array().into_iter().flatten().enumerate() {
            if v.get("description").is_none() {
                missing.push(format!("{name}[{i}]"));
            }
            // `{"Wall": 0}` : la description de la variante suffit ; seules les commandes ont des champs nommés
            let fields = v["properties"].as_object().filter(|f| f.contains_key("op"));
            for (field, p) in fields.into_iter().flatten() {
                if field != "op" && p.get("description").is_none() && p.get("$ref").is_none() {
                    missing.push(format!("{name}[{i}].{field}"));
                }
            }
        }
        for (field, p) in def["properties"].as_object().into_iter().flatten() {
            if p.get("description").is_none() && p.get("$ref").is_none() {
                missing.push(format!("{name}.{field}"));
            }
        }
    }
    assert!(missing.is_empty(), "sans description : {missing:?}");
}

#[test]
fn catalog_and_guide_are_available() {
    let o = run(&["catalog"]);
    assert!(o.status.success());
    let c: Value = serde_json::from_str(&stdout(&o)).expect("catalogue JSON");
    assert_eq!((&c["props"][0]["kind"], &c["props"][0]["category"]), (&Value::from("Table"), &Value::from("Salon")));
    assert_eq!(c["openings"][0]["width"], 0.9);
    let g = run(&["guide"]);
    assert!(g.status.success() && stdout(&g).contains("Boucle de travail"));
}

#[test]
fn check_brief_and_help_for_agents() {
    let dir = tmp_dir("check");
    let world = world_with_room(&dir);
    let cmds = write(&dir, "sofa.json", r#"[{"op": "place_prop", "kind": "Sofa", "pos": [2, 2], "level": 1}]"#);
    let o = run(&["apply", &world, &cmds, "--quiet"]);
    assert!(o.status.success());
    assert!(stdout(&o).contains("#0 ok → créé Prop 0") && stdout(&o).contains("flotte"), "{}", stdout(&o));
    assert!(!stdout(&o).contains("Emprise"), "--quiet n'affiche pas describe");
    assert!(stdout(&run(&["check", &world])).contains("Prop 0 (Sofa) flotte"));
    assert!(stdout(&run(&["schema", "--brief"])).contains("place_row("));
    assert!(stdout(&run(&["help", "set"])).contains("fields"));
    let missing = dir.join("absent.json");
    let o = run(&["apply", missing.to_str().expect("chemin UTF-8"), &cmds]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("--new"));
}
