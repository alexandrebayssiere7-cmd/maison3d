//! Écriture d'un monde sur le disque.

use super::{House, WorldError};

impl House {
    /// Écrit le monde, quel que soit le contenu actuel du fichier.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), WorldError> {
        self.write(path.as_ref(), None)
    }

    /// Comme `save`, mais refuse d'écraser le fichier s'il ne contient plus `read`, la maison lue au départ.
    pub fn save_if_unchanged(&self, path: impl AsRef<std::path::Path>, read: &House) -> Result<(), WorldError> {
        self.write(path.as_ref(), Some(read))
    }

    /// Écrit à côté, dans un .tmp propre au processus, puis renomme : un arrêt en pleine écriture
    /// ne corrompt pas le monde, et deux processus n'écrivent pas dans le même .tmp.
    fn write(&self, path: &std::path::Path, read: Option<&House>) -> Result<(), WorldError> {
        let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
        std::fs::create_dir_all(dir)?;
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(format!(".{}.tmp", std::process::id()));
        let json = serde_json::to_string_pretty(self)?;
        let res = std::fs::write(&tmp, json).map_err(WorldError::from).and_then(|()| {
            // Saves into this directory take turns from the check to the rename, so no other save can
            // slip in between. The lock is advisory and lives on the directory: the world file itself
            // is replaced by each rename. Released when `lock` is dropped.
            // shortcut: opening a directory fails on Windows, which no build targets; lock a sibling file there
            let lock = std::fs::File::open(dir)?;
            lock.lock()?;
            // vérifié au dernier moment, juste avant de remplacer le fichier
            read.map_or(Ok(()), |r| check_unchanged(path, r))?;
            Ok(std::fs::rename(&tmp, path)?)
        });
        if res.is_err() {
            // le .tmp est à nous ; s'il ne s'efface pas, l'erreur rendue reste la bonne à afficher
            let _ = std::fs::remove_file(&tmp);
        }
        res
    }
}

/// The file still holds `read`, or is gone (nothing to lose): `Ok`. Another world: `Changed`.
/// A file that no longer loads keeps its own error, so it is not mistaken for another writer.
fn check_unchanged(path: &std::path::Path, read: &House) -> Result<(), WorldError> {
    match House::load(path) {
        Ok(h) if h == *read => Ok(()),
        Ok(_) => Err(WorldError::Changed),
        Err(WorldError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Do both paths name the same file, however they are spelled? Existing files are compared after
/// resolving links; otherwise as absolute paths without `.` components.
pub fn same_file(a: impl AsRef<std::path::Path>, b: impl AsRef<std::path::Path>) -> bool {
    let (a, b) = (a.as_ref(), b.as_ref());
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => matches!((std::path::absolute(a), std::path::absolute(b)), (Ok(x), Ok(y)) if x == y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::demo_house;

    #[test]
    fn save_writes_atomically_and_reports_errors() {
        let dir = std::env::temp_dir().join("maison3d_house_save");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("sous/maison.json");
        let h = demo_house();
        h.save(&path).unwrap();
        assert_eq!(House::load(&path).unwrap(), h);
        assert_eq!(std::fs::read_dir(dir.join("sous")).unwrap().count(), 1, "aucun .tmp ne reste");
        // le dossier parent est un fichier
        assert!(matches!(h.save(path.join("x.json")), Err(WorldError::Io(_))));
        assert!(matches!(h.save(&dir), Err(WorldError::Io(_))), "un dossier n'est pas un fichier");
        let _ = std::fs::remove_dir_all(dir);
    }
    #[test]
    fn save_does_not_share_its_tmp_file_with_other_processes() {
        let dir = std::env::temp_dir().join(format!("maison3d_house_tmp_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("maison.json");
        // le .tmp d'un autre enregistrement en cours, sous le nom commun d'avant
        std::fs::create_dir_all(dir.join("maison.json.tmp")).unwrap();
        demo_house().save(&path).unwrap();
        assert_eq!(House::load(&path).unwrap(), demo_house());
        assert!(dir.join("maison.json.tmp").is_dir(), "le .tmp de l'autre n'est pas touché");
        let _ = std::fs::remove_dir_all(dir);
    }
    #[test]
    fn save_if_unchanged_refuses_a_file_changed_since_read() {
        let dir = std::env::temp_dir().join(format!("maison3d_house_changed_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("maison.json");
        let read = demo_house();
        read.save(&path).unwrap();
        let mut mine = read.clone();
        mine.props.clear();
        // un autre processus enregistre entre notre lecture et notre écriture
        House::default().save(&path).unwrap();
        let theirs = std::fs::read(&path).unwrap();
        assert!(matches!(mine.save_if_unchanged(&path, &read), Err(WorldError::Changed)));
        assert_eq!(std::fs::read(&path).unwrap(), theirs, "leur version reste intacte");
        // a file that no longer loads reports why, rather than claiming another writer changed it
        std::fs::write(&path, "pas un monde").unwrap();
        assert!(matches!(mine.save_if_unchanged(&path, &read), Err(WorldError::Json(_))));
        assert_eq!(std::fs::read(&path).unwrap(), b"pas un monde", "an unreadable file is not overwritten either");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1, "aucun .tmp ne reste");
        // inchangé, ou effacé entre-temps : on écrit
        read.save(&path).unwrap();
        mine.save_if_unchanged(&path, &read).unwrap();
        assert_eq!(House::load(&path).unwrap(), mine);
        std::fs::remove_file(&path).unwrap();
        mine.save_if_unchanged(&path, &read).unwrap();
        assert_eq!(House::load(&path).unwrap(), mine);
        assert!(matches!(mine.save_if_unchanged(&dir, &read), Err(WorldError::Io(_))), "un dossier est illisible");
        let _ = std::fs::remove_dir_all(dir);
    }
    /// Any failure after the .tmp is created removes it: with the pid in its name, leftovers would pile up.
    #[cfg(unix)]
    #[test]
    fn a_failed_write_leaves_no_tmp_file() {
        let dir = std::env::temp_dir().join(format!("maison3d_house_write_fails_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("maison.json");
        // writing through a dangling link fails even as root, like a full disk would
        let tmp = dir.join(format!("maison.json.{}.tmp", std::process::id()));
        std::os::unix::fs::symlink(dir.join("absent/x"), &tmp).unwrap();
        assert!(matches!(demo_house().save(&path), Err(WorldError::Io(_))));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0, "no .tmp is left behind");
        let _ = std::fs::remove_dir_all(dir);
    }
    /// Another save holds the directory between its check and its rename: ours waits, then sees its work.
    #[test]
    fn a_save_waits_for_the_one_in_progress() {
        let dir = std::env::temp_dir().join(format!("maison3d_house_lock_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("maison.json");
        let read = demo_house();
        read.save(&path).unwrap();
        let held = std::fs::File::open(&dir).unwrap();
        held.lock().unwrap();
        let (mut mine, p, r) = (read.clone(), path.clone(), read.clone());
        mine.props.clear();
        let ours = std::thread::spawn(move || mine.save_if_unchanged(&p, &r));
        std::thread::sleep(std::time::Duration::from_millis(200));
        // the other save writes its world while it still holds the lock
        std::fs::write(&path, serde_json::to_string_pretty(&House::default()).unwrap()).unwrap();
        drop(held);
        assert!(matches!(ours.join().unwrap(), Err(WorldError::Changed)));
        assert_eq!(House::load(&path).unwrap(), House::default(), "their work is kept");
        let _ = std::fs::remove_dir_all(dir);
    }
    #[test]
    fn same_file_compares_paths_not_spellings() {
        let dir = std::env::temp_dir().join(format!("maison3d_same_file_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (a, b) = (dir.join("a.json"), dir.join("b.json"));
        std::fs::write(&a, "").unwrap();
        std::fs::write(&b, "").unwrap();
        assert!(same_file(&a, dir.join(".").join("a.json")), "même fichier écrit autrement");
        assert!(!same_file(&a, &b));
        assert!(!same_file(&a, "absent.json"), "un seul des deux existe");
        assert!(same_file("absent.json", "absent.json"), "absents : même texte");
        assert!(same_file("absent.json", "./absent.json"), "absent files compare as normalized paths");
        assert!(!same_file("absent.json", "other.json"));
        // nettoyage seulement : un reste ne fausse aucun test
        let _ = std::fs::remove_dir_all(dir);
    }
}
