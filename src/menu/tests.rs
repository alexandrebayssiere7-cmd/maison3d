//! Tests du menu, du HUD et des enregistrements à la fermeture ou au changement de monde.

use super::*;

fn frame(mut f: impl FnMut(&egui::Context)) {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(1280.0, 800.0))),
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| f(ctx));
}

#[test]
fn every_screen_draws_without_requests() {
    let (mut s, mut form) = (Settings::default(), WorldForm { new_name: Some("x".into()), error: "oups".into() });
    for screen in [Screen::Start, Screen::Options, Screen::Levels, Screen::Game] {
        frame(|ctx| assert!(menu_ui(ctx, screen, &mut s, &mut form).is_empty()));
    }
    assert_eq!(form.error, "oups");
    let mut ed = Editor::default();
    frame(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            assert!(!hud(ui, &mut ed, &s, 60.0));
            ed.walk = true;
            assert!(!hud(ui, &mut ed, &s, 60.0));
        });
    });
    assert!((0.0..1.0).contains(&hue("Mon monde")));
}

#[test]
fn menu_requests_change_screen_window_and_world() {
    let (mut next, mut window, mut ed, mut s) =
        (NextState::<Screen>::default(), Window::default(), Editor::default(), Settings::default());
    s.fullscreen = true;
    s.vsync = false;
    let actions = vec![MenuAction::Fullscreen, MenuAction::Vsync, MenuAction::Goto(Screen::Options)];
    assert!(!apply_menu(actions, &mut next, &mut window, &mut ed, &s));
    assert!(matches!(window.mode, WindowMode::BorderlessFullscreen(_)));
    assert_eq!(window.present_mode, PresentMode::AutoNoVsync);
    assert!(matches!(next, NextState::Pending(Screen::Options)));
    let play = MenuAction::Play(Box::new(demo_house()), "mondes/a.json".into(), "Bienvenue".into());
    assert!(apply_menu(vec![play, MenuAction::Quit], &mut next, &mut window, &mut ed, &s));
    assert_eq!((ed.file.as_str(), ed.message.as_str()), ("mondes/a.json", "Bienvenue"));
    assert!(matches!(next, NextState::Pending(Screen::Game)));
    assert!(!ed.can_undo(), "nouvel éditeur, historique vide");
}

#[test]
fn new_world_names_are_checked() {
    let dir = std::env::temp_dir().join("maison3d_menu_new_world");
    // le dossier peut ne pas exister : seul compte qu'il soit vide ensuite
    let _ = std::fs::remove_dir_all(&dir);
    assert!(matches!(new_world(&dir, "  "), Err(NewWorldError::InvalidName)));
    assert!(matches!(new_world(&dir, "a/b"), Err(NewWorldError::InvalidName)));
    assert!(matches!(new_world(&dir, "a\\b"), Err(NewWorldError::InvalidName)));
    let (house, file, msg) = new_world(&dir, " Chalet ").unwrap();
    let chalet = dir.join("Chalet.json");
    assert_eq!(
        (house, file.as_str(), msg.as_str()),
        (House::default(), chalet.to_str().unwrap(), "Nouveau monde « Chalet »")
    );
    assert_eq!(free_name(&dir), dir.join("monde 1.json").to_str().unwrap());
    House::default().save(&chalet).unwrap();
    House::default().save(dir.join("monde 1.json")).unwrap();
    assert!(matches!(new_world(&dir, "Chalet"), Err(NewWorldError::Exists(n)) if n == "Chalet"));
    assert_eq!(free_name(&dir), dir.join("monde 2.json").to_str().unwrap());
}

#[test]
fn worlds_are_listed_from_new_and_legacy_dirs() {
    let root = std::env::temp_dir().join("maison3d_menu_custom_worlds");
    // le dossier peut ne pas exister : seul compte qu'il soit vide ensuite
    let _ = std::fs::remove_dir_all(&root);
    let (new, old) = (root.join("nouveau"), root.join("ancien"));
    for f in [new.join("b.json"), old.join("a.json")] {
        House::default().save(f).unwrap();
    }
    std::fs::write(new.join("notes.txt"), "").unwrap();
    let listed = custom_worlds(&[new.clone(), old.clone(), new.clone(), root.join("absent")]);
    assert_eq!(listed, [old.join("a.json"), new.join("b.json")]);
}

#[test]
fn worlds_dir_follows_each_os_convention() {
    let env = |pairs: &'static [(&str, &str)]| {
        move |k: &str| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| OsString::from(v))
    };
    let home = env(&[("HOME", "/h"), ("APPDATA", "/a"), ("XDG_DATA_HOME", "/x")]);
    let suffix = Path::new("maison3d").join("mondes");
    assert_eq!(worlds_dir_for("macos", home), Path::new("/h/Library/Application Support").join(&suffix));
    assert_eq!(worlds_dir_for("windows", home), Path::new("/a").join(&suffix));
    assert_eq!(worlds_dir_for("linux", home), Path::new("/x").join(&suffix));
    let no_xdg = env(&[("HOME", "/h"), ("XDG_DATA_HOME", "")]);
    assert_eq!(worlds_dir_for("linux", no_xdg), Path::new("/h/.local/share").join(&suffix));
    let nothing = env(&[]);
    for os in ["macos", "windows", "linux", "freebsd"] {
        assert_eq!(worlds_dir_for(os, nothing), Path::new("mondes"), "{os}");
    }
    assert!(worlds_dir().ends_with(&suffix) || worlds_dir() == Path::new("mondes"));
}

#[test]
fn new_world_errors_have_labels() {
    assert_eq!(NewWorldError::InvalidName.to_string(), "Nom invalide");
    assert_eq!(NewWorldError::Exists("Chalet".into()).to_string(), "« Chalet » existe déjà");
}

#[test]
fn closing_saves_and_warns_once_on_failure() {
    let mut ed = Editor::default();
    let mut warned = false;
    assert!(may_close(&mut ed, true, &mut warned), "rien à enregistrer");
    ed.house.walls.push(crate::model::room_walls(Vec2::ZERO, Vec2::ONE, 0, 3.0, 0.2, [1.0; 3]).remove(0));
    ed.saved_file = std::env::temp_dir().to_string_lossy().into_owned();
    assert!(!may_close(&mut ed, true, &mut warned), "échec : on reste pour voir l erreur");
    assert!(may_close(&mut ed, true, &mut warned), "seconde demande : on ferme");
    assert!(may_close(&mut ed, false, &mut false), "hors jeu : rien à enregistrer");
}

/// Fichier temporaire propre à ce processus, supprimé en fin de test.
struct TempFile(PathBuf);

impl TempFile {
    fn new(name: &str) -> Self {
        Self(std::env::temp_dir().join(format!("maison3d_{}_{name}", std::process::id())))
    }
}

impl AsRef<Path> for TempFile {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // le test a pu ne jamais créer le fichier (ou en faire un dossier)
        let _ = std::fs::remove_file(&self.0).or_else(|_| std::fs::remove_dir_all(&self.0));
    }
}

/// Un monde enregistré dans `name` puis ouvert comme depuis le menu.
fn opened(name: &str) -> (Editor, TempFile) {
    let path = TempFile::new(name);
    demo_house().save(&path).unwrap();
    let mut ed = Editor::default();
    play(&mut ed, House::load(&path).unwrap(), path.0.to_string_lossy().into_owned(), String::new());
    (ed, path)
}

#[test]
fn closing_saves_changes_made_without_undo_point() {
    let (mut ed, path) = opened("maison3d_close_no_undo.json");
    // comme la voiture garée : la maison change sans point d annulation
    ed.house.props[0].pos += Vec2::new(3.0, 0.0);
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(House::load(&path).unwrap(), ed.house);
}

#[test]
fn closing_after_save_then_undo_restores_the_file() {
    let (mut ed, path) = opened("maison3d_close_undo.json");
    let wall = crate::model::room_walls(Vec2::splat(30.0), Vec2::splat(34.0), 0, 3.0, 0.2, [1.0; 3]).remove(0);
    ed.apply([crate::command::Command::AddWall(wall)]).unwrap();
    assert!(ed.save());
    ed.undo();
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(House::load(&path).unwrap(), demo_house(), "le mur annulé ne reste pas dans le fichier");
}

#[test]
fn closing_does_not_overwrite_a_world_changed_by_an_agent() {
    let (mut ed, path) = opened("close_agent.json");
    // un agent réécrit le monde pendant la partie
    House::default().save(&path).unwrap();
    let theirs = std::fs::read(&path).unwrap();
    ed.house.props[0].pos += Vec2::new(3.0, 0.0);
    let mut warned = false;
    assert!(!may_close(&mut ed, true, &mut warned), "refus : on reste pour voir l erreur");
    assert!(ed.message.starts_with("Erreur d'enregistrement"), "{}", ed.message);
    // the user must know that closing again drops the changes, and how to keep them
    assert!(ed.message.contains("abandonne les modifications"), "{}", ed.message);
    assert!(ed.message.contains("sous un autre nom"), "{}", ed.message);
    assert!(!ed.message.contains("rouvrir"), "reopening would drop the changes too: {}", ed.message);
    assert!(may_close(&mut ed, true, &mut warned), "seconde demande : on ferme");
    assert_eq!(std::fs::read(&path).unwrap(), theirs, "le travail de l agent reste intact");
}

#[test]
fn switching_world_changes_nothing_when_an_agent_changed_the_open_one() {
    let (mut ed, chalet) = opened("chalet_agent.json");
    let dir = TempFile::new("nouveaux_mondes_agent");
    std::fs::create_dir_all(&dir).unwrap();
    let wall = crate::model::room_walls(Vec2::splat(30.0), Vec2::splat(34.0), 0, 3.0, 0.2, [1.0; 3]).remove(0);
    ed.apply([crate::command::Command::AddWall(wall)]).unwrap();
    House::default().save(&chalet).unwrap();
    let theirs = std::fs::read(&chalet).unwrap();
    let (house, file, saved_file) = (ed.house.clone(), ed.file.clone(), ed.saved_file.clone());
    start_world(&mut ed, demo_house(), &dir.0, "Nouveau monde");
    assert!(ed.message.starts_with("Erreur d'enregistrement"), "{}", ed.message);
    assert_eq!((ed.house == house, ed.file == file, ed.saved_file == saved_file), (true, true, true));
    assert!(ed.can_undo(), "l historique est gardé");
    assert_eq!(std::fs::read(&chalet).unwrap(), theirs, "le travail de l agent reste intact");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0, "aucun nouveau monde n est créé");
}

#[test]
fn a_free_name_taken_meanwhile_is_not_overwritten() {
    let (mut ed, _chalet) = opened("chalet_free.json");
    let dir = TempFile::new("nouveaux_mondes_free");
    std::fs::create_dir_all(&dir).unwrap();
    start_world(&mut ed, House::default(), &dir.0, "Nouveau monde");
    ed.house.props.push(demo_house().props[0].clone());
    // un agent crée le même fichier avant le premier enregistrement
    let first = dir.0.join("monde 1.json");
    demo_house().save(&first).unwrap();
    assert!(!ed.save(), "le fichier créé entre-temps n est pas écrasé");
    assert!(ed.message.contains("sous un autre nom"), "the save button also says how to keep the work: {}", ed.message);
    assert_eq!(House::load(&first).unwrap(), demo_house());
    // issue : enregistrer sous un autre nom
    let other = dir.0.join("autre.json");
    ed.file = other.to_string_lossy().into_owned();
    assert!(ed.save(), "{}", ed.message);
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(House::load(&other).unwrap(), ed.house);
}

#[test]
fn closing_an_untouched_world_does_not_rewrite_it() {
    let (mut ed, path) = opened("maison3d_close_untouched.json");
    std::fs::write(&path, "témoin").unwrap();
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "témoin");
}

#[test]
fn undoing_an_open_then_closing_leaves_the_opened_file_intact() {
    let (mut ed, a) = opened("open_undo_a.json");
    ed.house.props[0].pos += Vec2::new(3.0, 0.0);
    let edited = ed.house.clone();
    let b = TempFile::new("open_undo_b.json");
    House::default().save(&b).unwrap();
    ed.file = b.0.to_string_lossy().into_owned();
    ed.load();
    assert_eq!(ed.house, House::default());
    assert_eq!(ed.message, format!("Ouvert : {} ({} enregistré)", ed.file, a.0.display()), "un seul message");
    ed.undo();
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(House::load(&b).unwrap(), House::default(), "b.json n est pas écrasé");
    assert_eq!(House::load(&a).unwrap(), edited, "les changements de a.json sont gardés");
}

#[test]
fn reopening_the_same_file_is_undoable_and_writes_nothing() {
    let (mut ed, path) = opened("reopen_same.json");
    ed.house.props[0].pos += Vec2::new(3.0, 0.0);
    let edited = ed.house.clone();
    ed.load();
    let disk = House::load(&path).unwrap();
    assert_eq!((&disk, &disk), (&ed.saved, &ed.house), "le disque, saved et l écran concordent");
    assert_eq!(disk, demo_house(), "les changements abandonnés ne sont pas écrits");
    ed.undo();
    assert_eq!(ed.house, edited, "Ctrl+Z rend le travail abandonné");
    // le même fichier écrit autrement reste le même fichier
    let (dir, name) = (path.0.parent().unwrap(), path.0.file_name().unwrap());
    ed.file = dir.join(".").join(name).to_string_lossy().into_owned();
    ed.load();
    assert_eq!(House::load(&path).unwrap(), demo_house(), "toujours rien d écrit");
    ed.undo();
    assert_eq!(ed.house, edited);
}

#[test]
fn reopening_an_untouched_world_keeps_the_redo_stack() {
    let (mut ed, _path) = opened("reopen_redo.json");
    ed.apply([crate::command::Command::Remove { target: crate::model::ElemRef::Prop(0).into() }]).unwrap();
    ed.undo();
    assert!(ed.can_redo());
    ed.load();
    assert!(ed.can_redo(), "rouvrir sans changement ne coupe pas Ctrl+Y");
}

#[test]
fn undoing_a_reopen_then_closing_saves_the_changes() {
    let (mut ed, path) = opened("reopen_undo_close.json");
    ed.house.props[0].pos += Vec2::new(3.0, 0.0);
    let edited = ed.house.clone();
    ed.load();
    ed.undo();
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(House::load(&path).unwrap(), edited, "Ctrl+Z après réouverture reprend le travail");
}

#[test]
fn closing_saves_to_the_opened_file_not_the_typed_name() {
    let (mut ed, a) = opened("close_typed_a.json");
    ed.house.props[0].pos += Vec2::new(3.0, 0.0);
    let b = TempFile::new("close_typed_b.json");
    ed.file = b.0.to_string_lossy().into_owned();
    assert!(may_close(&mut ed, true, &mut false));
    assert!(!b.0.exists(), "le nom tapé sans ouvrir n est pas une cible");
    assert_eq!(House::load(&a).unwrap(), ed.house);
}

#[test]
fn hud_names_the_opened_world_not_the_typed_one() {
    let mut ed = Editor::default();
    play(&mut ed, House::default(), "mondes/chalet.json".into(), String::new());
    ed.file = "mondes/autre.json".into();
    assert_eq!(world_title(&ed), "CHALET");
}

#[test]
fn new_world_then_opening_another_leaves_the_open_world_intact() {
    // scénario de maison3d-87 : sur chalet.json, Nouveau, on tape b.json, Ctrl+O, puis on ferme
    let (mut ed, chalet) = opened("chalet.json");
    let dir = TempFile::new("nouveaux_mondes");
    std::fs::create_dir_all(&dir).unwrap();
    start_world(&mut ed, House::default(), &dir.0, "Nouveau monde");
    let b = TempFile::new("chalet_b.json");
    House::default().save(&b).unwrap();
    ed.file = b.0.to_string_lossy().into_owned();
    ed.load();
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(House::load(&chalet).unwrap(), demo_house(), "chalet.json intact");
    assert_eq!(House::load(&b).unwrap(), House::default());
}

#[test]
fn new_world_then_closing_leaves_the_open_world_intact() {
    let (mut ed, chalet) = opened("chalet_close.json");
    let dir = TempFile::new("nouveaux_mondes_close");
    std::fs::create_dir_all(&dir).unwrap();
    start_world(&mut ed, House::default(), &dir.0, "Nouveau monde");
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(House::load(&chalet).unwrap(), demo_house(), "chalet.json intact");
}

#[test]
fn new_and_example_save_the_open_world_then_take_a_free_name() {
    let (mut ed, chalet) = opened("chalet_edit.json");
    let dir = TempFile::new("nouveaux_mondes_edit");
    std::fs::create_dir_all(&dir).unwrap();
    ed.house.props[0].pos += Vec2::new(3.0, 0.0);
    let edited = ed.house.clone();
    start_world(&mut ed, House::default(), &dir.0, "Nouveau monde");
    assert_eq!(House::load(&chalet).unwrap(), edited, "le monde modifié est d abord enregistré");
    let first = dir.0.join("monde 1.json");
    assert_eq!(ed.saved_file, first.to_string_lossy());
    assert_eq!(ed.file, ed.saved_file);
    assert!(!ed.modified() && !ed.can_undo());
    assert!(may_close(&mut ed, true, &mut false));
    assert!(!first.exists(), "un monde vide intact n est pas écrit");
    start_world(&mut ed, demo_house(), &dir.0, "Maison d exemple");
    assert!(!ed.modified(), "l exemple n est écrit que s il change");
    ed.house.props.clear();
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(House::load(&first).unwrap(), ed.house, "l exemple modifié va dans son propre fichier");
    assert_eq!(House::load(&chalet).unwrap(), edited);
}

#[test]
fn new_world_is_refused_when_the_open_one_cannot_be_saved() {
    let (mut ed, _chalet) = opened("chalet_fail.json");
    ed.house.props.clear();
    let kept = ed.house.clone();
    ed.saved_file = std::env::temp_dir().to_string_lossy().into_owned();
    start_world(&mut ed, demo_house(), &std::env::temp_dir(), "Nouveau monde");
    assert_eq!(ed.house, kept, "le travail non enregistré reste ouvert");
    assert!(ed.message.starts_with("Erreur d'enregistrement"), "{}", ed.message);
}

#[test]
fn two_new_worlds_in_a_row_get_two_files() {
    let (mut ed, _chalet) = opened("chalet_twice.json");
    let dir = TempFile::new("nouveaux_mondes_twice");
    std::fs::create_dir_all(&dir).unwrap();
    start_world(&mut ed, House::default(), &dir.0, "Nouveau monde");
    ed.house.props.push(demo_house().props[0].clone());
    let first = ed.house.clone();
    start_world(&mut ed, House::default(), &dir.0, "Nouveau monde");
    assert_eq!(ed.saved_file, dir.0.join("monde 2.json").to_string_lossy(), "le premier a pris monde 1");
    ed.house.props.push(demo_house().props[1].clone());
    assert!(may_close(&mut ed, true, &mut false));
    assert_eq!(House::load(dir.0.join("monde 1.json")).unwrap(), first, "le premier monde n est pas écrasé");
    assert_eq!(House::load(dir.0.join("monde 2.json")).unwrap(), ed.house);
}
