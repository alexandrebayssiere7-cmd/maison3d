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

#[test]
fn describe_invalid_world_exits_1() {
    let world = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/monde_invalide.json");
    let o = run(&["describe", world]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("Wall(1) invalide"), "{}", String::from_utf8_lossy(&o.stderr));
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
        let reload = run(&["describe", world.to_str().expect("chemin UTF-8")]);
        assert!(reload.status.success(), "{name} relu : {}", String::from_utf8_lossy(&reload.stderr));
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
fn apply_json_prints_one_line_per_command_then_a_summary() {
    let dir = tmp_dir("json");
    let world = world_with_room(&dir);
    let cmds = write(
        &dir,
        "two.json",
        r#"[{"op": "place_prop", "kind": "Lamp", "pos": [2, 2], "level": 0}, {"op": "remove", "target": {"Wall": 0}}]"#,
    );
    let o = run(&["apply", &world, &cmds, "--json", "--dry-run"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let lines: Vec<Value> = stdout(&o).lines().map(|l| serde_json::from_str(l).expect("ligne JSON")).collect();
    assert_eq!(lines.len(), 3, "{}", stdout(&o));
    assert_eq!(lines[0]["created"], serde_json::json!([{"Prop": 0}]));
    assert_eq!(lines[1]["removed"], serde_json::json!({"Wall": 1}));
    assert_eq!(lines[2]["saved"], Value::Null);
    assert_eq!(read_json(&world)["walls"].as_array().map(Vec::len), Some(4), "--dry-run n'a rien écrit");

    let bad = write(&dir, "bad.json", r#"[{"op": "remove", "target": {"Wall": 42}}]"#);
    let o = run(&["apply", &world, &bad, "--json"]);
    assert_eq!(o.status.code(), Some(1));
    let refused: Value = serde_json::from_str(stdout(&o).trim()).expect("refus en JSON");
    assert_eq!((&refused["index"], &refused["ok"]), (&Value::from(0), &Value::from(false)));
}

#[test]
fn search_finds_props_by_word_and_filters() {
    let o = run(&["search", "chaise", "de", "bureau", "--limit", "2"]);
    assert!(o.status.success());
    let out = stdout(&o);
    assert!(out.starts_with("OfficeChair\t"), "{out}");
    assert!(out.lines().last().is_some_and(|l| l.contains("--limit")), "{out}");
    let wall = stdout(&run(&["search", "--room", "kitchen", "--placement", "onwall"]));
    assert!(wall.contains("WallCabinet") && !wall.contains("Fridge"));
    assert!(stdout(&run(&["search", "zzzz"])).starts_with("aucun objet"));
    assert!(!run(&["search", "x", "--room", "cave"]).status.success());
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

#[test]
fn query_finds_named_rooms_and_their_walls() {
    let dir = tmp_dir("query");
    let world = dir.join("monde.json").to_str().expect("chemin UTF-8").to_owned();
    assert!(run(&["apply", &world, "docs/exemples/maison-deux-pieces.json", "--new", "--quiet"]).status.success());
    let rooms = stdout(&run(&["query", &world, "rooms"]));
    assert!(rooms.contains("« séjour »") && rooms.contains("« chambre »"), "{rooms}");
    let o = run(&["query", &world, "room", "chambre"]);
    assert!(o.status.success());
    assert!(stdout(&o).contains("ouest  Wall 4 : face intérieure x = 6.05"), "{}", stdout(&o));
    assert!(stdout(&run(&["query", &world, "near", "8,1"])).starts_with("dans : « chambre »"));
    let missing = run(&["query", &world, "room", "grenier"]);
    assert!(!missing.status.success() && String::from_utf8_lossy(&missing.stderr).contains("pièce inconnue"));
}

#[test]
fn suggest_prints_place_prop_commands_that_apply() {
    let dir = tmp_dir("suggest");
    let world = dir.join("monde.json").to_str().expect("chemin UTF-8").to_owned();
    assert!(run(&["apply", &world, "docs/exemples/maison-deux-pieces.json", "--new", "--quiet"]).status.success());
    let o = run(&["suggest", &world, "Nightstand", "--room", "chambre", "--count", "3"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    assert_eq!(out.lines().count(), 3, "{out}");
    let first = out.lines().next().and_then(|l| l.split_once("  # ")).expect("commande  # raisons").0;
    let cmds = write(&dir, "un.json", &format!("[{first}]"));
    let applied = run(&["apply", &world, &cmds, "--quiet"]);
    assert!(stdout(&applied).contains("Aucun problème"), "{}", stdout(&applied));
    assert!(!run(&["suggest", &world, "Sofa"]).status.success(), "--room manquant");
}

#[test]
fn shot_fails_when_the_image_cannot_be_written() {
    let dir = tmp_dir("shot-nope");
    let world = world_with_room(&dir);
    let png = dir.join("absent").join("vue.png");
    let o = run(&["shot", &world, png.to_str().expect("chemin UTF-8")]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!o.status.success(), "dossier inexistant : la capture doit échouer");
    assert!(err.contains("vue.png"), "le message nomme l'image : {err}");
    let xyz = run(&["shot", &world, dir.join("vue.xyz").to_str().expect("chemin UTF-8")]);
    let err = String::from_utf8_lossy(&xyz.stderr);
    assert!(!xyz.status.success() && err.contains("png"), "format refusé avant le rendu : {err}");
}

/// Deux `apply` sur le même monde : celui qui finit le second n'efface pas le travail du premier.
#[cfg(unix)]
#[test]
fn apply_refuses_to_overwrite_a_world_changed_meanwhile() {
    concurrent_apply_is_refused("concurrent", |_| vec![]);
    // --out naming the world read, spelled another way, is still the world read
    concurrent_apply_is_refused("concurrent-out", |w| {
        let w = Path::new(w);
        let same = w.parent().expect("dossier").join(".").join(w.file_name().expect("nom"));
        vec!["--out".into(), same.to_str().expect("chemin UTF-8").into()]
    });
    // a script reading stdout already saw ok:true lines: the refusal must reach it there too
    let o = concurrent_apply_is_refused("concurrent-json", |_| vec!["--json".into()]);
    let out = stdout(&o);
    let last: Value = serde_json::from_str(out.lines().last().expect("une ligne")).expect("ligne JSON");
    assert_eq!(last["ok"], false, "{out}");
    assert!(last["error"].as_str().is_some_and(|e| e.contains("modifié par ailleurs")), "{out}");
}

/// Le premier `apply` (avec les options `extra`, selon le monde) attend ses commandes pendant qu'un second écrit.
#[cfg(unix)]
fn concurrent_apply_is_refused(name: &str, extra: impl Fn(&str) -> Vec<String>) -> Output {
    use std::io::Write as _;
    use std::process::Stdio;
    let dir = tmp_dir(name);
    let world = world_with_room(&dir);
    let fifo = dir.join("cmds.fifo");
    assert!(Command::new("mkfifo").arg(&fifo).status().expect("lancer mkfifo").success());
    let slow = Command::new(env!("CARGO_BIN_EXE_maison3d"))
        .args(["apply", &world, fifo.to_str().expect("chemin UTF-8")])
        .args(extra(&world))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("lancer maison3d");
    // ouvrir la file en écriture attend que le premier apply l'ouvre en lecture : il a déjà lu le monde
    let mut cmds = std::fs::OpenOptions::new().write(true).open(&fifo).expect("ouvrir la file");
    let other = write(&dir, "other.json", r#"[{"op": "room", "min": [10, 10], "max": [12, 12], "level": 0}]"#);
    assert!(run(&["apply", &world, &other]).status.success(), "le second apply passe");
    let theirs = std::fs::read(&world).expect("lire le monde");
    cmds.write_all(br#"[{"op": "room", "min": [20, 20], "max": [22, 22], "level": 0}]"#).expect("écrire");
    drop(cmds);
    let o = slow.wait_with_output().expect("attendre le premier apply");
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(1), "le premier apply est refusé : {err}");
    assert!(err.contains("modifié par ailleurs"), "{err}");
    assert_eq!(std::fs::read(&world).expect("relire le monde"), theirs, "le travail du second reste intact");
    let left: Vec<_> = std::fs::read_dir(&dir).expect("lister").flatten().map(|e| e.file_name()).collect();
    assert!(!left.iter().any(|n| n.to_string_lossy().ends_with(".tmp")), "{left:?}");
    o
}
