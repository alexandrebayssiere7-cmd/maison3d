//! Menus (démarrage, options, choix du monde) et HUD en jeu.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use bevy::diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowMode};
use bevy_egui::{EguiContexts, egui};
use egui::{Align2, Color32, FontId, Rect, RichText, Sense, Slider, Stroke, StrokeKind, vec2};

use crate::editor::Editor;
use crate::model::{House, demo_house};
use crate::render::MainCamera;
use crate::ui::{ACCENT, hud_frame, level_name};

/// Ancien dossier des mondes, relatif au répertoire courant : encore listé, plus jamais créé.
const LEGACY_DIR: &str = "mondes";

/// Dossier des mondes dans les données utilisateur de `os` (valeur de `std::env::consts::OS`),
/// d'après les variables d'environnement lues par `var` ; `mondes` si aucune n'est définie.
fn worlds_dir_for(os: &str, var: impl Fn(&str) -> Option<OsString>) -> PathBuf {
    let var = |k| var(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = match os {
        "macos" => var("HOME").map(|h| h.join("Library").join("Application Support")),
        "windows" => var("APPDATA"),
        _ => var("XDG_DATA_HOME").or_else(|| var("HOME").map(|h| h.join(".local").join("share"))),
    };
    base.map_or_else(|| PathBuf::from(LEGACY_DIR), |b| b.join("maison3d").join(LEGACY_DIR))
}

/// Dossier où sont créés les mondes personnalisés.
pub fn worlds_dir() -> PathBuf {
    worlds_dir_for(std::env::consts::OS, |k| std::env::var_os(k))
}

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Screen {
    #[default]
    Start,
    Options,
    Levels,
    Game,
}

#[derive(Resource)]
pub struct Settings {
    pub sensitivity: f32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub hud: bool,
    pub fps: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { sensitivity: 1.0, fullscreen: false, vsync: true, hud: true, fps: true }
    }
}

/// Premier « monde N.json » libre de `dir`, pour ne jamais écraser un monde existant.
fn free_name(dir: &Path) -> String {
    (1..=u32::MAX)
        .map(|i| dir.join(format!("monde {i}.json")))
        .find(|f| !f.exists())
        .expect("suite infinie : un nom finit par être libre")
        .to_string_lossy()
        .into_owned()
}

/// Mondes de `dirs` (dossier actuel puis ancien), triés, sans doublon.
fn custom_worlds(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut v: Vec<_> = dirs
        .iter()
        .flat_map(std::fs::read_dir)
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    v.sort();
    v.dedup();
    v
}

/// Nouveau ou Exemple depuis l'éditeur : `house` devient un nouveau monde, sous un nom libre de `dir`
/// comme depuis le menu, pour ne jamais être enregistrée par-dessus le monde ouvert.
pub fn start_world(ed: &mut Editor, house: House, dir: &Path, what: &str) {
    ed.switch_world(house, || free_name(dir), what);
}

/// Repart d'un éditeur neuf (historique vidé) sur ce monde.
fn play(ed: &mut Editor, house: House, file: String, message: String) {
    *ed = Editor::default();
    ed.saved = house.clone();
    ed.house = house;
    ed.saved_file.clone_from(&file);
    ed.file = file;
    ed.message = message;
}

const MARGIN: f32 = 80.0;

/// Derrière les menus, la caméra tourne lentement autour de la maison d'exemple.
pub fn menu_camera(time: Res<Time>, mut cam: Single<&mut Transform, With<MainCamera>>) {
    let a = time.elapsed_secs() * 0.06;
    let focus = Vec3::new(5.0, 2.0, 4.0);
    **cam =
        Transform::from_translation(focus + Vec3::new(a.sin() * 26.0, 11.0, a.cos() * 26.0)).looking_at(focus, Vec3::Y);
}

fn pointer(resp: &egui::Response) {
    if resp.hovered() {
        resp.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

/// Entrée de menu : texte qui glisse et passe à l'accent au survol, barre à gauche.
fn nav_button(ui: &mut egui::Ui, text: &str) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(340.0, 52.0), Sense::click());
    let t = ui.ctx().animate_bool(resp.id, resp.hovered());
    let p = ui.painter();
    p.rect_filled(rect, 6.0, Color32::from_white_alpha((t * 14.0) as u8));
    p.rect_filled(Rect::from_min_size(rect.min, vec2(4.0, rect.height()) * vec2(1.0, t)), 2.0, ACCENT);
    let color = Color32::from_gray(220).lerp_to_gamma(ACCENT, t);
    p.text(
        rect.left_center() + vec2(18.0 + 10.0 * t, 0.0),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(28.0),
        color,
    );
    pointer(&resp);
    resp.clicked()
}

/// Carte de monde : vignette colorée, grande icône, nom et étiquette.
fn card(ui: &mut egui::Ui, icon: &str, name: &str, tag: &str, hue: f32, ghost: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(220.0, 140.0), Sense::click());
    let t = ui.ctx().animate_bool(resp.id, resp.hovered());
    let r = rect.translate(vec2(0.0, -4.0 * t));
    let p = ui.painter();
    if ghost {
        p.rect_filled(r, 12.0, Color32::from_white_alpha((6.0 + 14.0 * t) as u8));
        p.rect_stroke(
            r,
            12.0,
            Stroke::new(1.5_f32, Color32::from_gray(120).lerp_to_gamma(ACCENT, t)),
            StrokeKind::Inside,
        );
    } else {
        p.rect_filled(r.translate(vec2(0.0, 6.0)), 12.0, Color32::from_black_alpha(90));
        p.rect_filled(r, 12.0, Color32::from(egui::ecolor::Hsva::new(hue, 0.55, 0.42, 1.0)));
        let band = Rect::from_min_max(r.left_bottom() - vec2(0.0, 50.0), r.max);
        p.rect_filled(band, egui::CornerRadius { nw: 0, ne: 0, sw: 12, se: 12 }, Color32::from_black_alpha(130));
        p.rect_stroke(r, 12.0, Stroke::new(2.0_f32, ACCENT.gamma_multiply(t)), StrokeKind::Inside);
    }
    let fg = if ghost { Color32::from_gray(190).lerp_to_gamma(ACCENT, t) } else { Color32::WHITE };
    p.text(r.left_top() + vec2(18.0, 14.0), Align2::LEFT_TOP, icon, FontId::proportional(40.0), fg);
    let name: String =
        if name.chars().count() > 22 { name.chars().take(21).chain(['…']).collect() } else { name.into() };
    p.text(r.left_bottom() + vec2(16.0, -30.0), Align2::LEFT_CENTER, name, FontId::proportional(18.0), fg);
    p.text(
        r.left_bottom() + vec2(16.0, -12.0),
        Align2::LEFT_CENTER,
        tag,
        FontId::proportional(12.0),
        Color32::from_gray(170),
    );
    pointer(&resp);
    resp.clicked()
}

/// Interrupteur (d'après la démo egui).
fn toggle(ui: &mut egui::Ui, on: &mut bool) -> bool {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(46.0, 24.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let t = ui.ctx().animate_bool(resp.id, *on);
    let p = ui.painter();
    p.rect_filled(rect, 12.0, Color32::from_gray(70).lerp_to_gamma(ACCENT, t));
    p.circle_filled(
        egui::pos2(egui::lerp(rect.left() + 12.0..=rect.right() - 12.0, t), rect.center().y),
        9.0,
        Color32::WHITE,
    );
    pointer(&resp);
    resp.changed()
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(14.0);
    ui.label(RichText::new(text.to_uppercase()).size(14.0).color(ACCENT).strong());
    ui.add_space(6.0);
}

fn hue(name: &str) -> f32 {
    name.bytes().fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b))) as f32 % 360.0 / 360.0
}

/// Ce que l'utilisateur a demandé dans le menu cette image.
#[derive(Debug)]
enum MenuAction {
    Goto(Screen),
    Quit,
    Fullscreen,
    Vsync,
    /// Monde choisi : maison, fichier, message d'accueil.
    Play(Box<House>, String, String),
}

/// Saisie du nom d'un nouveau monde et dernière erreur affichée.
#[derive(Default)]
pub struct WorldForm {
    new_name: Option<String>,
    error: String,
}

pub fn menu_system(
    mut egui: EguiContexts,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    mut s: ResMut<Settings>,
    mut ed: ResMut<Editor>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut exit: EventWriter<AppExit>,
    mut form: Local<WorldForm>,
) {
    let actions = menu_ui(egui.ctx_mut(), *screen.get(), &mut s, &mut form);
    if apply_menu(actions, &mut next, &mut window, &mut ed, &s) {
        exit.write(AppExit::Success);
    }
}

/// Exécute les demandes du menu ; vrai s'il faut quitter.
fn apply_menu(
    actions: Vec<MenuAction>,
    next: &mut NextState<Screen>,
    window: &mut Window,
    ed: &mut Editor,
    s: &Settings,
) -> bool {
    let mut quit = false;
    for a in actions {
        match a {
            MenuAction::Goto(screen) => next.set(screen),
            MenuAction::Quit => quit = true,
            MenuAction::Fullscreen => {
                window.mode = if s.fullscreen {
                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                } else {
                    WindowMode::Windowed
                };
            }
            MenuAction::Vsync => {
                window.present_mode = if s.vsync { PresentMode::AutoVsync } else { PresentMode::AutoNoVsync };
            }
            MenuAction::Play(house, file, msg) => {
                play(ed, *house, file, msg);
                next.set(Screen::Game);
            }
        }
    }
    quit
}

/// Voile sombre à gauche qui s'efface vers la droite : la maison reste visible.
fn paint_veil(ctx: &egui::Context, screen_rect: Rect) {
    let mut veil = egui::Mesh::default();
    let (dark, clear) = (Color32::from_black_alpha(235), Color32::from_black_alpha(40));
    let mid = screen_rect.left() + screen_rect.width() * 0.75;
    for (x, c) in [(screen_rect.left(), dark), (mid, clear)] {
        veil.colored_vertex(egui::pos2(x, screen_rect.top()), c);
        veil.colored_vertex(egui::pos2(x, screen_rect.bottom()), c);
    }
    veil.add_triangle(0, 1, 2);
    veil.add_triangle(1, 2, 3);
    veil.add_colored_rect(Rect::from_min_max(egui::pos2(mid, screen_rect.top()), screen_rect.max), clear);
    ctx.layer_painter(egui::LayerId::background()).add(veil);
}

/// Dessine le menu de l'écran `screen` et renvoie les demandes de l'utilisateur.
fn menu_ui(ctx: &egui::Context, screen: Screen, s: &mut Settings, form: &mut WorldForm) -> Vec<MenuAction> {
    let screen_rect = ctx.screen_rect();
    paint_veil(ctx, screen_rect);
    let mut actions = vec![];
    if screen != Screen::Start && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        actions.push(MenuAction::Goto(Screen::Start));
    }
    let width = (screen_rect.width() - 2.0 * MARGIN).max(300.0);
    egui::Area::new(egui::Id::new("menu")).fixed_pos(egui::pos2(MARGIN, (screen_rect.height() * 0.12).max(30.0))).show(
        ctx,
        |ui| {
            ui.set_max_width(width);
            ui.visuals_mut().selection.bg_fill = ACCENT;
            ui.visuals_mut().slider_trailing_fill = true;
            ui.label(RichText::new("MAISON 3D").size(64.0).strong().color(Color32::WHITE));
            ui.label(RichText::new("Construire  ·  Meubler  ·  Visiter").size(18.0).color(ACCENT));
            ui.add_space(36.0);
            match screen {
                Screen::Start | Screen::Game => start_menu(ui, &mut actions),
                Screen::Options => options_menu(ui, s, &mut actions),
                Screen::Levels => levels_menu(ui, form, width, screen_rect.bottom(), &mut actions),
            }
        },
    );
    egui::Area::new(egui::Id::new("pied")).anchor(Align2::LEFT_BOTTOM, [MARGIN, -24.0]).interactable(false).show(
        ctx,
        |ui| {
            let hint = if screen == Screen::Start { "" } else { "Échap : retour     " };
            ui.label(
                RichText::new(format!("{hint}v{}", env!("CARGO_PKG_VERSION")))
                    .size(13.0)
                    .color(Color32::from_gray(150)),
            );
        },
    );
    actions
}

fn start_menu(ui: &mut egui::Ui, actions: &mut Vec<MenuAction>) {
    if nav_button(ui, "Jouer") {
        actions.push(MenuAction::Goto(Screen::Levels));
    }
    if nav_button(ui, "Options") {
        actions.push(MenuAction::Goto(Screen::Options));
    }
    if nav_button(ui, "Quitter") {
        actions.push(MenuAction::Quit);
    }
}

fn options_menu(ui: &mut egui::Ui, s: &mut Settings, actions: &mut Vec<MenuAction>) {
    ui.label(RichText::new("Options").size(30.0).color(Color32::WHITE));
    egui::Frame::new().fill(Color32::from_black_alpha(120)).corner_radius(12.0).inner_margin(24.0).show(ui, |ui| {
        ui.style_mut().text_styles.insert(egui::TextStyle::Body, FontId::proportional(17.0));
        egui::Grid::new("options").num_columns(2).spacing([60.0, 18.0]).show(ui, |ui| {
            ui.label("Sensibilité de la souris");
            ui.add(Slider::new(&mut s.sensitivity, 0.2..=3.0).show_value(false));
            ui.end_row();
            ui.label("Plein écran");
            if toggle(ui, &mut s.fullscreen) {
                actions.push(MenuAction::Fullscreen);
            }
            ui.end_row();
            ui.label("Synchronisation verticale");
            if toggle(ui, &mut s.vsync) {
                actions.push(MenuAction::Vsync);
            }
            ui.end_row();
            ui.label("Afficher le HUD");
            toggle(ui, &mut s.hud);
            ui.end_row();
            ui.label("Afficher les images/s");
            toggle(ui, &mut s.fps);
            ui.end_row();
        });
    });
    ui.add_space(20.0);
    if nav_button(ui, "← Retour") {
        actions.push(MenuAction::Goto(Screen::Start));
    }
}

/// Choix du monde : test, standard (à venir), mondes enregistrés, nouveau monde.
fn levels_menu(ui: &mut egui::Ui, form: &mut WorldForm, width: f32, bottom: f32, actions: &mut Vec<MenuAction>) {
    let height = (bottom - ui.cursor().top() - 110.0).max(150.0);
    let mut chosen = egui::ScrollArea::vertical().max_height(height).show(ui, |ui| world_cards(ui, form, width)).inner;
    if form.new_name.is_some() {
        chosen = new_world_form(ui, form).or(chosen);
    }
    if !form.error.is_empty() {
        ui.add_space(6.0);
        ui.colored_label(Color32::from_rgb(255, 110, 110), &form.error);
    }
    ui.add_space(12.0);
    if nav_button(ui, "← Retour") {
        actions.push(MenuAction::Goto(Screen::Start));
    }
    if let Some((house, file, msg)) = chosen {
        form.error.clear();
        form.new_name = None;
        actions.push(MenuAction::Play(Box::new(house), file, msg));
    }
}

/// Cartes des mondes ; renvoie celui qui a été cliqué.
fn world_cards(ui: &mut egui::Ui, form: &mut WorldForm, width: f32) -> Option<(House, String, String)> {
    let mut chosen = None;
    ui.set_width(width);
    let grid = |ui: &mut egui::Ui, add: &mut dyn FnMut(&mut egui::Ui)| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(18.0, 18.0);
            add(ui);
        });
    };
    heading(ui, "Test");
    grid(ui, &mut |ui| {
        if card(ui, "🏠", "Monde de test", "Maison d'exemple", 0.58, false) {
            chosen = Some((demo_house(), free_name(&worlds_dir()), "Monde de test".to_string()));
        }
    });
    heading(ui, "Mondes standard");
    grid(ui, &mut |ui| {
        card(ui, "⏳", "Bientôt disponibles", "En préparation", 0.0, true);
    });
    heading(ui, "Mes mondes");
    grid(ui, &mut |ui| {
        if card(ui, "+", "Nouveau monde", "Partir de zéro", 0.0, true) {
            form.new_name = Some(String::new());
        }
        for p in custom_worlds(&[worlds_dir(), LEGACY_DIR.into()]) {
            let name = p.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            if card(ui, "🏡", &name, "Personnalisé", hue(&name), false) {
                let file = p.to_string_lossy().into_owned();
                match House::load(&p) {
                    Ok(h) => chosen = Some((h, file.clone(), format!("Ouvert : {file}"))),
                    Err(e) => form.error = format!("Impossible d'ouvrir {name} : {e}"),
                }
            }
        }
    });
    chosen
}

/// Champ du nom d'un nouveau monde, boutons Créer et Annuler.
fn new_world_form(ui: &mut egui::Ui, form: &mut WorldForm) -> Option<(House, String, String)> {
    let name = form.new_name.as_mut()?;
    ui.add_space(12.0);
    let (mut chosen, mut cancel) = (None, false);
    ui.horizontal(|ui| {
        let edit = ui.add(
            egui::TextEdit::singleline(name)
                .hint_text("Nom du nouveau monde")
                .font(FontId::proportional(18.0))
                .desired_width(280.0)
                .margin(vec2(10.0, 8.0)),
        );
        edit.request_focus();
        let enter = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let create = ui.add(
            egui::Button::new(RichText::new("Créer").size(17.0).color(Color32::BLACK))
                .fill(ACCENT)
                .min_size(vec2(90.0, 36.0)),
        );
        pointer(&create);
        if create.clicked() || enter {
            match new_world(&worlds_dir(), name) {
                Ok(c) => chosen = Some(c),
                Err(e) => form.error = e.to_string(),
            }
        }
        cancel = ui.add(egui::Button::new(RichText::new("Annuler").size(17.0)).min_size(vec2(90.0, 36.0))).clicked();
    });
    if cancel {
        form.new_name = None;
    }
    chosen
}

/// Pourquoi un nouveau monde ne peut pas être créé.
#[derive(Debug)]
enum NewWorldError {
    /// Nom vide ou contenant un séparateur de chemin.
    InvalidName,
    /// Un monde porte déjà ce nom.
    Exists(String),
}

impl std::fmt::Display for NewWorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidName => write!(f, "Nom invalide"),
            Self::Exists(name) => write!(f, "« {name} » existe déjà"),
        }
    }
}

/// Monde vide nommé `name` dans `dir`, ou pourquoi c'est impossible.
fn new_world(dir: &Path, name: &str) -> Result<(House, String, String), NewWorldError> {
    let name = name.trim();
    let file = dir.join(format!("{name}.json"));
    if name.is_empty() || name.contains(['/', '\\']) {
        Err(NewWorldError::InvalidName)
    } else if file.exists() {
        Err(NewWorldError::Exists(name.into()))
    } else {
        Ok((House::default(), file.to_string_lossy().into_owned(), format!("Nouveau monde « {name} »")))
    }
}

/// Fermer la fenêtre en jeu enregistre d'abord le monde ; si ça échoue, on reste ouvert une fois
/// pour que l'erreur soit vue, et une seconde demande ferme quand même.
pub fn save_on_close(
    mut close: EventReader<bevy::window::WindowCloseRequested>,
    mut ed: ResMut<Editor>,
    screen: Res<State<Screen>>,
    mut exit: EventWriter<AppExit>,
    mut warned: Local<bool>,
) {
    if close.read().count() == 0 {
        return;
    }
    if may_close(&mut ed, *screen.get() == Screen::Game, &mut warned) {
        exit.write(AppExit::Success);
    }
}

/// Enregistre le monde modifié avant de fermer ; un échec retient la fermeture une seule fois.
fn may_close(ed: &mut Editor, in_game: bool, warned: &mut bool) -> bool {
    if in_game && !ed.save_if_modified() && !*warned {
        *warned = true;
        return false;
    }
    true
}

pub fn hud_system(
    mut egui: EguiContexts,
    mut ed: ResMut<Editor>,
    s: Res<Settings>,
    diag: Res<DiagnosticsStore>,
    mut next: ResMut<NextState<Screen>>,
) {
    let fps = diag.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(Diagnostic::smoothed).unwrap_or(0.0);
    let menu = egui::Area::new(egui::Id::new("hud"))
        .anchor(Align2::RIGHT_TOP, [-14.0, 14.0])
        .show(egui.ctx_mut(), |ui| hud_frame().show(ui, |ui| hud(ui, &mut ed, &s, fps)).inner)
        .inner;
    if menu {
        next.set(Screen::Start);
    }
}

/// Nom du monde ouvert, pas celui tapé dans le champ tant qu'il n'est pas ouvert ou enregistré.
fn world_title(ed: &Editor) -> String {
    Path::new(&ed.saved_file).file_stem().unwrap_or_default().to_string_lossy().to_uppercase()
}

/// Nom du monde, mode, étage, heure et images/s ; vrai si l'on retourne au menu.
fn hud(ui: &mut egui::Ui, ed: &mut Editor, s: &Settings, fps: f64) -> bool {
    ui.set_min_width(170.0);
    if s.hud {
        ui.label(RichText::new(world_title(ed)).strong().size(15.0).color(ACCENT));
        ui.label(
            RichText::new(format!("{}  ·  {}", if ed.walk { "Visite" } else { "Édition" }, level_name(ed.level)))
                .color(Color32::from_gray(220)),
        );
        ui.label(
            RichText::new(format!("🕒 {:02}h{:02}", ed.hour as u32, (ed.hour.fract() * 60.0) as u32))
                .color(Color32::from_gray(220)),
        );
    }
    if s.fps {
        ui.label(RichText::new(format!("{fps:.0} img/s")).monospace().color(Color32::GRAY));
    }
    // en visite la souris est capturée : on sort d'abord par Échap
    if ed.walk {
        return false;
    }
    ui.add_space(4.0);
    let menu = ui.add(
        egui::Button::new(RichText::new("☰  Menu").color(Color32::BLACK).strong())
            .fill(ACCENT)
            .corner_radius(6.0)
            .min_size(vec2(ui.available_width(), 28.0)),
    );
    // si l'enregistrement échoue, on reste en jeu : l'erreur y est affichée
    menu.on_hover_text("Enregistre le monde s'il a été modifié").clicked() && ed.save_if_modified()
}

#[cfg(test)]
mod tests {
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
        ed.apply([crate::command::Command::Remove { target: crate::model::ElemRef::Prop(0) }]).unwrap();
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
}
