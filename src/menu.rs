//! Menus (démarrage, options, choix du monde) et HUD en jeu.

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

/// Dossier des mondes personnalisés.
const DIR: &str = "mondes";

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

/// Premier « mondes/monde N.json » libre, pour ne jamais écraser un monde existant.
fn free_name() -> String {
    (1..=u32::MAX)
        .map(|i| format!("{DIR}/monde {i}.json"))
        .find(|f| !Path::new(f).exists())
        .expect("suite infinie : un nom finit par être libre")
}

fn custom_worlds() -> Vec<PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(DIR)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    v.sort();
    v
}

/// Repart d'un éditeur neuf (historique vidé) sur ce monde.
fn play(ed: &mut Editor, house: House, file: String, message: String) {
    *ed = Editor::default();
    ed.house = house;
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
            chosen = Some((demo_house(), free_name(), "Monde de test".to_string()));
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
        for p in custom_worlds() {
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
            match new_world(name) {
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

/// Monde vide nommé `name` dans le dossier des mondes, ou pourquoi c'est impossible.
fn new_world(name: &str) -> Result<(House, String, String), NewWorldError> {
    let name = name.trim();
    let file = format!("{DIR}/{name}.json");
    if name.is_empty() || name.contains(['/', '\\']) {
        Err(NewWorldError::InvalidName)
    } else if Path::new(&file).exists() {
        Err(NewWorldError::Exists(name.into()))
    } else {
        Ok((House::default(), file, format!("Nouveau monde « {name} »")))
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
    if in_game && ed.can_undo() && !ed.save() && !*warned {
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

/// Nom du monde, mode, étage, heure et images/s ; vrai si l'on retourne au menu.
fn hud(ui: &mut egui::Ui, ed: &mut Editor, s: &Settings, fps: f64) -> bool {
    ui.set_min_width(170.0);
    if s.hud {
        let world = Path::new(&ed.file).file_stem().unwrap_or_default().to_string_lossy().to_uppercase();
        ui.label(RichText::new(world).strong().size(15.0).color(ACCENT));
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
    menu.on_hover_text("Enregistre le monde s'il a été modifié").clicked() && (!ed.can_undo() || ed.save())
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
        assert!(matches!(new_world("  "), Err(NewWorldError::InvalidName)));
        assert!(matches!(new_world("a/b"), Err(NewWorldError::InvalidName)));
        assert!(matches!(new_world("a\\b"), Err(NewWorldError::InvalidName)));
        let (house, file, msg) = new_world(" Chalet ").unwrap();
        assert_eq!(
            (house, file.as_str(), msg.as_str()),
            (House::default(), "mondes/Chalet.json", "Nouveau monde « Chalet »")
        );
        assert!(free_name().starts_with("mondes/monde "));
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
        ed.checkpoint();
        ed.file = std::env::temp_dir().to_string_lossy().into_owned();
        assert!(!may_close(&mut ed, true, &mut warned), "échec : on reste pour voir l erreur");
        assert!(may_close(&mut ed, true, &mut warned), "seconde demande : on ferme");
        assert!(may_close(&mut ed, false, &mut false), "hors jeu : rien à enregistrer");
    }
}
