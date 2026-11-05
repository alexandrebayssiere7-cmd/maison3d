# Git

- **Une branche par feature**, créée depuis `main` à jour : `feature/<sujet>`, `fix/<sujet>`, `refactor/<sujet>`, `chore/<sujet>` (kebab-case). Jamais de commit direct sur `main`.
- **Une branche = un sujet** : une idée annexe découverte en route va dans sa propre branche.
- **Commits atomiques** : un commit = un changement logique qui compile et dont les tests passent (`cargo test`). On ne mélange pas un refactor et une feature, ni un reformatage et une correction.
- **Messages** : titre en français à l'impératif présent (« Ajoute… », « Corrige… »), ≤ 72 caractères, sans point final. Si besoin, une ligne vide puis un corps qui explique le *pourquoi*, pas le *comment*.
- **Aucune mention de Claude** dans les messages : pas de `Co-Authored-By: Claude`, pas de `Generated with Claude Code`, ni aucune autre référence à Claude ou Anthropic.
- **Historique propre avant le merge** : pas de commits « wip », « fix typo », « oups ». Les corriger avec `git commit --fixup <sha>` puis `GIT_SEQUENCE_EDITOR=true git rebase -i --autosquash main`.
- **Rebase, pas de merge de `main` dans la branche** : `git fetch && git rebase origin/main` pour rester à jour.
- **On ne réécrit que ses propres branches** pas encore partagées ; une fois poussée, `git push --force-with-lease` seulement (jamais `--force`). `main` n'est jamais réécrite.
- **Merge dans `main`** via une PR, CI verte, puis suppression de la branche.
- Rien de généré ni de secret dans le dépôt : `target/`, sorties d'outils, `.env` passent par `.gitignore`.

# Qualité du code (score CRAP)

Le score CRAP (`CC² × (1 − couverture)³ + CC`) mesure le risque d'une fonction : complexe et peu testée = dangereuse à modifier. Viser moins de 30 partout.

- **Découper les fonctions** : pas de système Bevy de plus de ~10 branches. Un système lit les entrées (souris, clavier, caméra) et dessine ; la logique (que se passe-t-il au clic, au relâché, quel élément créer) va dans des méthodes de `Editor`, `House` ou des fonctions pures, testables sans Bevy.
- **Tester au maximum** : toute logique nouvelle ou modifiée arrive avec ses tests (`#[cfg(test)] mod tests` en bas du fichier). Couvrir les cas d'erreur (`Err` de `House::apply`, indices hors limites), pas seulement le cas nominal.
- Toute modification de la maison passe par `House::apply` / `Editor::apply` : un seul endroit à tester pour la validation.
- Un `match` qui renvoie un libellé par variante gonfle la complexité sans risque réel : un test qui parcourt `ALL` suffit.

Mesurer :

```bash
cargo llvm-cov --summary-only        # couverture par fichier
crapper src/*.rs                     # score CRAP (github.com/unclebob/crapper), écrit .metrics/crap.edn
```

`crapper` doit recevoir les fichiers explicitement : il ignore tout chemin contenant un dossier `test`, et le projet est sous `~/Desktop/test/`.

# Code Rust

- **Erreurs typées** : pas de `Result<_, String>`. Les erreurs de domaine sont des `enum` (ex. `EditError`) avec `Display` pour le libellé ; les tests vérifient la variante avec `matches!`, pas le texte.
- **Pas de `unwrap()` / `expect()` hors tests**, sauf `expect("raison")` sur un invariant réellement garanti.
- **Pas d'erreur avalée en silence** : un `let _ = …` sur un `Result` porte un commentaire qui dit pourquoi c'est sans danger.
- Le formatage suit `rustfmt.toml` ; `cargo clippy --all-targets -- -D warnings` doit passer (lints réglés dans `Cargo.toml`, section `[lints]`).

# Architecture

- **Pas de logique dupliquée** : une opération (ex. l'écriture atomique d'un monde, `House::save`) vit à un seul endroit, testé une fois.
- **Fichiers ≤ ~800 lignes** : au-delà, découper par domaine (`model/terrain.rs`, `model/props.rs`…).
- **Le cœur ne dépend pas de Bevy** : `model`, `command`, `terrain` n'importent de Bevy que les types mathématiques (`Vec2`, `Vec3`…). Tout le reste reste testable sans moteur.
- **Format de sauvegarde versionné** : le monde JSON porte un champ `version`. Tout changement incompatible de `House` arrive avec une migration et un test qui charge un ancien fichier de `tests/fixtures/`.
- **Historique d'annulation** : `VecDeque`, pas de `Vec::remove(0)` sur des clones de maison.

# Tests

- **Aller-retour** : sérialiser puis relire une `House` et chaque variante de `Command` redonne la même valeur.
- **Schéma figé** : le schéma JSON des commandes (`maison3d schema`) est commité dans `tests/fixtures/` ; un test échoue s'il change. Les agents en dépendent, tout changement doit être voulu.
- **Propriétés (`proptest`)** sur `House::apply` : une suite de commandes au hasard ne panique jamais, et un `Err` laisse la maison intacte.
- **CLI** : `apply`, `describe`, `schema` ont des tests d'intégration dans `tests/` qui lancent le binaire.
- **Nouvelle `Command`** = test nominal + test d’erreur + doc sur la variante et chaque champ (vérifié par `schema_documents_every_command_and_field`) + schéma régénéré + emploi dans un exemple de `docs/exemples/` (vérifié par `agent_examples_apply_and_cover_every_command`) ; mettre à jour `docs/guide-agent.md` si elle change la façon de construire.

# Outillage

- CI GitHub Actions (`.github/workflows/ci.yml`, sur `main` et les PR, cargo en `--locked`, actions épinglées sur SHA) : job `fmt` rapide ; job `check` avec `clippy -D warnings`, `test`, `llvm-cov`, et échec si une fonction dépasse un score CRAP de 30 ; `cargo deny check advisories` (exceptions motivées dans `.github/deny.toml`) ; build macOS. Dependabot hebdomadaire (cargo, actions).
- Toolchain figée dans `rust-toolchain.toml` (CI, release et local) ; la monter = un commit dédié.
- Release (`.github/workflows/release.yml`) : pousser un tag `v*` publie les binaires Linux et macOS dans une GitHub Release.
- Hook pre-commit dans `.githooks/` (`cargo fmt --check` + `clippy`) ; l'activer avec `git config core.hooksPath .githooks`.
- `.gitignore` tenu à jour : sorties générées (`graphify-out/`, `*.tmp`) ignorées.

# Idées et plans

- `idea/` : banque d’idées rangée par thème, structure décrite dans `idea/CLAUDE.md`.
- `plan/` : plans d’implémentation des idées retenues.
- Les deux dossiers sont ignorés par git.
