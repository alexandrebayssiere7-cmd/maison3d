//! Recherche dans le catalogue par mots-clés : un agent demande « chaise de bureau » et reçoit
//! les quelques objets qui conviennent, sans lire les 234 entrées de `catalog`.

use super::{Placement, Room};
use crate::model::PropKind;

/// Objets qui répondent à `query`, du plus pertinent au moins pertinent. Une requête vide garde
/// tout ce qui passe les filtres, dans l'ordre du catalogue.
pub fn search(query: &str, room: Option<Room>, placement: Option<Placement>) -> Vec<PropKind> {
    let words = tokens(query);
    let mut hits: Vec<(u32, PropKind)> = PropKind::ALL
        .into_iter()
        .filter_map(|k| {
            let a = k.annotation()?;
            let keep = room.is_none_or(|r| a.rooms.contains(&r)) && placement.is_none_or(|p| a.placement == p);
            let s = score(k, &words);
            (keep && (words.is_empty() || s > 0)).then_some((s, k))
        })
        .collect();
    // tri stable : à score égal, l'ordre du catalogue
    hits.sort_by_key(|h| std::cmp::Reverse(h.0));
    hits.into_iter().map(|(_, k)| k).collect()
}

/// Pertinence de `k` pour les mots de la requête : 3 par mot trouvé tel quel, 1 par début de mot
/// commun, et un bonus si la requête entière est un de ses synonymes.
fn score(k: PropKind, words: &[String]) -> u32 {
    let Some(a) = k.annotation() else { return 0 };
    // le nom JSON aussi (`TvStand` → « tvstand »)
    let kind = format!("{k:?}");
    let phrases: Vec<Vec<String>> =
        a.synonyms.iter().copied().chain([k.name(), k.category().name(), kind.as_str()]).map(tokens).collect();
    let known: Vec<&String> = phrases.iter().flatten().collect();
    let per_word: u32 = words
        .iter()
        .map(|w| {
            if known.contains(&w) {
                3
            } else {
                // dans les deux sens : « frig » → « frigo », « lampe » → « lamp »
                u32::from(known.iter().any(|k| prefix(w, k) || prefix(k, w)))
            }
        })
        .sum();
    let whole = phrases.iter().any(|p| !p.is_empty() && *p == words);
    per_word + if whole { 5 } else { 0 }
}

/// `short` commence `long` et n'est pas trop court pour être parlant (« bus » ne trouve pas « bureau »).
fn prefix(short: &str, long: &str) -> bool {
    short.len() >= 4 && long.starts_with(short)
}

/// Mots significatifs, sans accents ni pluriel : « Chaises de bureau » → [chaise, bureau].
fn tokens(text: &str) -> Vec<String> {
    const STOP: [&str; 11] = ["de", "du", "des", "la", "le", "les", "un", "une", "a", "the", "of"];
    text.split(|c: char| !c.is_alphanumeric())
        .map(fold)
        .filter(|w| !w.is_empty() && !STOP.contains(&w.as_str()))
        .map(|w| match w.strip_suffix(['s', 'x']) {
            Some(stem) if stem.len() >= 3 => stem.to_owned(),
            _ => w,
        })
        .collect()
}

/// En minuscules et sans accents : « Séjour » → « sejour ».
pub fn fold(text: &str) -> String {
    text.chars().flat_map(char::to_lowercase).map(unaccent).collect()
}

fn unaccent(c: char) -> char {
    match c {
        'à' | 'â' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'î' | 'ï' => 'i',
        'ô' | 'ö' => 'o',
        'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_drop_accents_plurals_and_small_words() {
        assert_eq!(tokens("Chaises de Bureau"), ["chaise", "bureau"]);
        assert_eq!(tokens("réfrigérateur"), ["refrigerateur"]);
        assert_eq!(tokens("  "), Vec::<String>::new());
        // un mot court garde son s
        assert_eq!(tokens("bus"), ["bus"]);
    }

    #[test]
    fn finds_by_french_or_english_synonym_and_by_prefix() {
        assert_eq!(search("frigo", None, None).first(), Some(&PropKind::Fridge));
        assert_eq!(search("Réfrigérateurs", None, None).first(), Some(&PropKind::Fridge));
        assert_eq!(search("refrigerator", None, None).first(), Some(&PropKind::Fridge));
        assert_eq!(search("TvStand", None, None).first(), Some(&PropKind::TvStand));
        assert!(search("frig", None, None).contains(&PropKind::Fridge));
        // le mot connu est le début du mot cherché : « lamp » dans « lampes »
        assert!(search("lampes plafond", None, None).contains(&PropKind::CeilingLamp));
        assert_eq!(search("zzzz", None, None), []);
    }

    #[test]
    fn whole_synonym_ranks_first() {
        // « îlot » seul trouve l'îlot avant tout ce qui contient « cuisine »
        assert_eq!(search("îlot de cuisine", None, None).first(), Some(&PropKind::KitchenIsland));
    }

    #[test]
    fn filters_by_room_and_placement() {
        let kitchen = search("", Some(Room::Kitchen), None);
        assert!(kitchen.contains(&PropKind::Fridge) && !kitchen.contains(&PropKind::Bed));
        let on_wall = search("", Some(Room::Kitchen), Some(Placement::OnWall));
        assert!(on_wall.contains(&PropKind::WallCabinet) && !on_wall.contains(&PropKind::Fridge));
        assert_eq!(search("frigo", Some(Room::Garden), None), []);
        assert_eq!(search("", None, None).len(), PropKind::ALL.len());
    }
}
