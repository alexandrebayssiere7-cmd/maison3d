//! Command-line option parsing, shared by every subcommand.

use super::CliError;

/// Options sans valeur.
const SWITCHES: [&str; 5] = ["--new", "--quiet", "--brief", "--dry-run", "--json"];

/// Options that take a value; any option in neither list is unknown.
const VALUED: [&str; 12] = [
    "--out",
    "--room",
    "--placement",
    "--limit",
    "--level",
    "--radius",
    "--facing",
    "--count",
    "--from",
    "--zone",
    "--size",
    "--only",
];

/// Options avec leur valeur (`None` pour un interrupteur).
type Flags<'a> = Vec<(&'a str, Option<&'a str>)>;

/// Sépare les arguments positionnels des options `--nom [valeur]`. Refuse une option répétée
/// et une option à valeur suivie d'une autre option (`--out --json`).
/// Rejects an unknown option by its name, wherever it stands.
pub(super) fn split<'a>(args: &[&'a str]) -> Result<(Vec<&'a str>, Flags<'a>), CliError> {
    let (mut pos, mut flags): (_, Flags) = (vec![], vec![]);
    let mut it = args.iter().copied().peekable();
    while let Some(a) = it.next() {
        if !a.starts_with("--") {
            pos.push(a);
            continue;
        }
        let value = if SWITCHES.contains(&a) {
            None
        } else if VALUED.contains(&a) {
            let v = it.next_if(|v| !v.starts_with("--"));
            Some(v.ok_or_else(|| CliError::BadOption(format!("{a} sans valeur")))?)
        } else {
            return Err(CliError::BadOption(a.to_owned()));
        };
        if flags.iter().any(|(n, _)| *n == a) {
            return Err(CliError::BadOption(format!("{a} répétée")));
        }
        flags.push((a, value));
    }
    Ok((pos, flags))
}

/// Valeur d'une option parmi `all`, sans tenir compte de la casse (`--room kitchen`).
pub(super) fn choice<T: Copy + std::fmt::Debug>(
    flags: &[(&str, Option<&str>)],
    name: &str,
    all: &[T],
) -> Result<Option<T>, CliError> {
    let Some((_, v)) = flags.iter().find(|(n, _)| *n == name) else { return Ok(None) };
    let bad = || {
        let names: Vec<String> = all.iter().map(|x| format!("{x:?}")).collect();
        CliError::BadOption(format!("{name} {} (valeurs : {})", v.unwrap_or(""), names.join(", ")))
    };
    let v = v.ok_or_else(bad)?;
    all.iter().find(|x| format!("{x:?}").eq_ignore_ascii_case(v)).map(|x| Some(*x)).ok_or_else(bad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_value_cannot_be_another_option() {
        for args in [&["apply", "m.json", "c.json", "--out", "--json"][..], &["--level", "--radius", "1"], &["--out"]] {
            let err = split(args).unwrap_err();
            assert!(matches!(&err, CliError::BadOption(o) if o.ends_with("sans valeur")), "{args:?} : {err}");
        }
        // a negative value is not an option
        let (pos, flags) = split(&["near", "0,0", "--level", "-1"]).unwrap();
        assert_eq!((pos, flags), (vec!["near", "0,0"], vec![("--level", Some("-1"))]));
    }

    #[test]
    fn unknown_option_is_named_wherever_it_stands() {
        for args in
            [&["apply", "m.json", "c.json", "--dryrun", "--json"][..], &["schema", "--dryrun"], &["--dryrun", "x"]]
        {
            let err = split(args).unwrap_err();
            assert!(matches!(&err, CliError::BadOption(o) if o == "--dryrun"), "{args:?} : {err}");
        }
    }

    #[test]
    fn repeated_option_is_refused() {
        for bad in [&["--count", "1", "--count", "3"][..], &["--level", "0", "--level", "0"], &["--brief", "--brief"]] {
            let err = split(bad).unwrap_err();
            assert!(matches!(&err, CliError::BadOption(o) if o.ends_with("répétée")), "{bad:?} : {err}");
        }
    }

    #[test]
    fn every_known_option_is_accepted() {
        for name in SWITCHES {
            assert_eq!(split(&[name]).unwrap().1, vec![(name, None)]);
        }
        for name in VALUED {
            assert_eq!(split(&[name, "1"]).unwrap().1, vec![(name, Some("1"))]);
        }
    }
}
