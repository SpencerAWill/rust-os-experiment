//! `cargo xtask refs [--fetch]`: the reference library's manifest, status and download.
//!
//! The rules are in spec section 4.4 of chunk M0.1. Downloaded documents live in
//! `reference/files/` and their text in `reference/text/`; both are gitignored.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

/// The file format of a document, also its extension in `files/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum Format {
    #[serde(rename = "pdf")]
    Pdf,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "md")]
    Md,
    #[serde(rename = "tar.gz")]
    TarGz,
    #[serde(rename = "zip")]
    Zip,
}

/// One `[[doc]]` table of `manifest.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Doc {
    pub id: String,
    pub title: String,
    pub version: String,
    pub url: String,
    pub sha256: Option<String>,
    pub format: Format,
    pub licence: String,
    #[serde(default)]
    pub manual: bool,
    pub how: Option<String>,
}

/// How one entry stands on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RefStatus {
    Ok,
    Missing,
    Changed,
    NoText,
    /// A `manual` entry without `sha256` whose file is present: the file's digest.
    Unpinned(String),
}

/// The manifest file: a list of `[[doc]]` tables.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(default)]
    doc: Vec<Doc>,
}

/// Largest download `curl` accepts, in bytes.
const MAX_FILESIZE: &str = "1073741824";

impl Format {
    /// The extension of the file in `files/`.
    fn ext(self) -> &'static str {
        match self {
            Format::Pdf => "pdf",
            Format::Html => "html",
            Format::Md => "md",
            Format::TarGz => "tar.gz",
            Format::Zip => "zip",
        }
    }

    /// The extension of the text in `text/`, for the formats that have text.
    fn text_ext(self) -> Option<&'static str> {
        match self {
            Format::Pdf => Some("txt"),
            Format::Html => Some("html"),
            Format::Md => Some("md"),
            Format::TarGz | Format::Zip => None,
        }
    }
}

impl Doc {
    fn file(&self, root: &Path) -> PathBuf {
        root.join("reference/files")
            .join(format!("{}.{}", self.id, self.format.ext()))
    }

    fn text(&self, root: &Path) -> Option<PathBuf> {
        let ext = self.format.text_ext()?;
        Some(
            root.join("reference/text")
                .join(format!("{}.{ext}", self.id)),
        )
    }
}

fn is_kebab(id: &str) -> bool {
    !id.is_empty()
        && id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Parses and validates `manifest.toml` text.
pub(crate) fn parse_manifest(text: &str) -> Result<Vec<Doc>, String> {
    let manifest: Manifest = toml::from_str(text).map_err(|e| format!("manifest.toml: {e}"))?;
    let mut seen = BTreeSet::new();
    for d in &manifest.doc {
        let id = &d.id;
        if !is_kebab(id) {
            return Err(format!("manifest.toml: id `{id}` is not kebab-case"));
        }
        if !seen.insert(id.as_str()) {
            return Err(format!("manifest.toml: duplicate id `{id}`"));
        }
        if !d.url.starts_with("https://") {
            return Err(format!("manifest.toml: `{id}`: url must be https"));
        }
        if let Some(sha) = &d.sha256
            && !is_sha256(sha)
        {
            return Err(format!(
                "manifest.toml: `{id}`: sha256 must be 64 lowercase hex digits"
            ));
        }
        if d.sha256.is_none() && !d.manual {
            return Err(format!(
                "manifest.toml: `{id}`: sha256 is required unless `manual = true`"
            ));
        }
        if d.how.is_some() && !d.manual {
            return Err(format!(
                "manifest.toml: `{id}`: `how` needs `manual = true`"
            ));
        }
        if d.manual && d.how.is_none() {
            return Err(format!(
                "manifest.toml: `{id}`: `manual = true` needs `how`"
            ));
        }
    }
    Ok(manifest.doc)
}

fn digest_of(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(Sha256::digest(&bytes)
        .iter()
        .fold(String::new(), |mut hex, b| {
            write!(hex, "{b:02x}").expect("writing to a String cannot fail");
            hex
        }))
}

/// The status of `doc` under the repository root `root`.
pub(crate) fn status(root: &Path, doc: &Doc) -> RefStatus {
    let file = doc.file(root);
    if !file.is_file() {
        return RefStatus::Missing;
    }
    let Ok(actual) = digest_of(&file) else {
        return RefStatus::Missing;
    };
    match &doc.sha256 {
        Some(want) if *want != actual => return RefStatus::Changed,
        Some(_) => {}
        None => return RefStatus::Unpinned(actual),
    }
    match doc.text(root) {
        Some(text) if !text.is_file() => RefStatus::NoText,
        _ => RefStatus::Ok,
    }
}

/// Verifies `tmp` against `sha256`, then renames it to `dest`; on a mismatch deletes `tmp`.
pub(crate) fn place(tmp: &Path, dest: &Path, sha256: &str) -> Result<(), String> {
    let actual = digest_of(tmp);
    match actual {
        Ok(actual) if actual == sha256 => std::fs::rename(tmp, dest)
            .map_err(|e| format!("cannot move {} to {}: {e}", tmp.display(), dest.display())),
        Ok(actual) => {
            drop(std::fs::remove_file(tmp));
            Err(format!(
                "checksum mismatch for {}: expected {sha256}, got {actual}; the publisher changed the document",
                dest.display()
            ))
        }
        Err(e) => {
            drop(std::fs::remove_file(tmp));
            Err(e)
        }
    }
}

/// The `curl` arguments that download `url` to `out`.
pub(crate) fn curl_args(url: &str, out: &Path) -> Vec<String> {
    [
        "--fail",
        "--location",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--max-filesize",
        MAX_FILESIZE,
        "--output",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .chain([out.display().to_string(), url.to_string()])
    .collect()
}

/// Handles one entry for [`fetch_all`].
fn fetch_one(
    root: &Path,
    doc: &Doc,
    download: &dyn Fn(&Doc, &Path) -> Result<(), String>,
    convert: &dyn Fn(&Path, &Path) -> Result<(), String>,
) -> Result<(), String> {
    let file = doc.file(root);
    let mut regenerate = false;
    match status(root, doc) {
        RefStatus::Ok | RefStatus::Unpinned(_) => {}
        RefStatus::NoText => regenerate = true,
        RefStatus::Missing if doc.manual => {
            let how = doc.how.as_deref().unwrap_or("");
            println!("manual {}: {how} (expected {})", doc.id, file.display());
            return Ok(());
        }
        RefStatus::Changed if doc.manual => {
            return Err(format!(
                "checksum mismatch for the manual download {}",
                file.display()
            ));
        }
        RefStatus::Missing | RefStatus::Changed => {
            let sha = doc.sha256.as_deref().unwrap_or("");
            let tmp = file.with_extension(format!("{}.part", doc.format.ext()));
            if let Some(dir) = file.parent() {
                std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            if let Err(e) = download(doc, &tmp) {
                drop(std::fs::remove_file(&tmp));
                return Err(e);
            }
            place(&tmp, &file, sha)?;
            regenerate = true;
        }
    }
    if regenerate && let Some(text) = doc.text(root) {
        if let Some(dir) = text.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        if let Err(e) = convert(&file, &text) {
            drop(std::fs::remove_file(&text));
            return Err(e);
        }
    }
    Ok(())
}

/// Downloads or converts what `docs` need; attempts every entry and is `Err` if any failed.
pub(crate) fn fetch_all(
    root: &Path,
    docs: &[Doc],
    download: &dyn Fn(&Doc, &Path) -> Result<(), String>,
    convert: &dyn Fn(&Path, &Path) -> Result<(), String>,
) -> Result<(), String> {
    let mut failed = 0usize;
    for doc in docs {
        match fetch_one(root, doc, download, convert) {
            Ok(()) => println!("ok {}", doc.id),
            Err(e) => {
                failed += 1;
                eprintln!("FAIL {}: {e}", doc.id);
            }
        }
    }
    if failed == 0 {
        Ok(())
    } else {
        Err(format!(
            "refs --fetch: {failed} entr{} failed",
            if failed == 1 { "y" } else { "ies" }
        ))
    }
}

/// The real download: `curl` with [`curl_args`], never through a shell.
fn curl_download(doc: &Doc, out: &Path) -> Result<(), String> {
    let status = std::process::Command::new("curl")
        .args(curl_args(&doc.url, out))
        .status()
        .map_err(|e| format!("cannot start `curl`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`curl` failed ({status}) for {}", doc.url))
    }
}

/// The real conversion: `pdftotext -layout` for a PDF, a copy for `html` and `md`, writing to
/// a temporary name and renaming on success.
fn real_convert(src: &Path, dest: &Path) -> Result<(), String> {
    let tmp = dest.with_extension("part");
    let result = if src.extension().is_some_and(|e| e == "pdf") {
        std::process::Command::new("pdftotext")
            .arg("-layout")
            .arg(src)
            .arg(&tmp)
            .status()
            .map_err(|e| format!("cannot start `pdftotext`: {e}"))
            .and_then(|s| {
                s.success()
                    .then_some(())
                    .ok_or_else(|| format!("`pdftotext` failed ({s}) for {}", src.display()))
            })
    } else {
        std::fs::copy(src, &tmp)
            .map(drop)
            .map_err(|e| format!("cannot copy {}: {e}", src.display()))
    };
    result
        .and_then(|()| std::fs::rename(&tmp, dest).map_err(|e| format!("{}: {e}", dest.display())))
        .inspect_err(|_| drop(std::fs::remove_file(&tmp)))
}

fn load(root: &Path) -> Result<Vec<Doc>, String> {
    let path = root.join("reference/manifest.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    parse_manifest(&text)
}

/// Every entry's id and status, for doctor check 7.
pub(crate) fn statuses(root: &Path) -> Result<Vec<(String, RefStatus)>, String> {
    Ok(load(root)?
        .into_iter()
        .map(|d| {
            let s = status(root, &d);
            (d.id, s)
        })
        .collect())
}

/// Runs `cargo xtask refs [--fetch]`.
pub(crate) fn run(fetch: bool) -> Result<(), String> {
    let root = crate::doctor::repository_root(&crate::doctor::SystemRunner)?;
    let docs = load(&root)?;
    if fetch {
        return fetch_all(&root, &docs, &curl_download, &real_convert);
    }
    for doc in &docs {
        match status(&root, doc) {
            RefStatus::Ok => println!("ok {}", doc.id),
            RefStatus::Missing => println!("missing {}", doc.id),
            RefStatus::Changed => println!("changed {}", doc.id),
            RefStatus::NoText => println!("no-text {}", doc.id),
            RefStatus::Unpinned(sha) => println!("unpinned {}: sha256 = \"{sha}\"", doc.id),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;

    const SHA_A: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"; // "abc"
    const SHA_Z: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    fn entry(id: &str) -> String {
        format!(
            "[[doc]]\nid = \"{id}\"\ntitle = \"T\"\nversion = \"1\"\nurl = \"https://example.org/x.pdf\"\nsha256 = \"{SHA_A}\"\nformat = \"pdf\"\nlicence = \"free\"\n"
        )
    }

    fn rejects(text: &str, needle: &str) {
        let err = parse_manifest(text).expect_err("manifest should be rejected");
        assert!(err.contains(needle), "`{err}` should mention `{needle}`");
    }

    #[test]
    fn a_valid_manifest_parses_also_as_crlf() {
        let text = format!(
            "# header\n{}\n{}[[doc]]\nid = \"m\"\ntitle = \"M\"\nversion = \"2\"\nurl = \"https://example.org/\"\nformat = \"zip\"\nlicence = \"click\"\nmanual = true\nhow = \"download it\"\n",
            entry("a-1"),
            entry("b")
        );
        let docs = parse_manifest(&text).unwrap();
        assert_eq!(docs.len(), 3);
        assert_eq!(docs[0].id, "a-1");
        assert_eq!(docs[2].format, Format::Zip);
        assert!(docs[2].manual && docs[2].sha256.is_none());
        assert_eq!(parse_manifest(&text.replace('\n', "\r\n")).unwrap(), docs);
        assert_eq!(parse_manifest("# only a header\n").unwrap(), vec![]);
    }

    #[test]
    fn unknown_field_is_rejected() {
        rejects(&format!("{}extra = 1\n", entry("a")), "extra");
    }

    #[test]
    fn missing_required_field_is_rejected() {
        rejects(&entry("a").replace("title = \"T\"\n", ""), "title");
    }

    #[test]
    fn duplicate_id_is_rejected() {
        rejects(&format!("{}{}", entry("a"), entry("a")), "duplicate");
    }

    #[test]
    fn non_kebab_id_is_rejected() {
        for bad in ["A", "a_b", "-a", "a-", "a--b", "", "a b"] {
            rejects(&entry(bad), "kebab");
        }
    }

    #[test]
    fn http_url_is_rejected() {
        rejects(&entry("a").replace("https://", "http://"), "https");
    }

    #[test]
    fn unknown_format_is_rejected() {
        rejects(&entry("a").replace("\"pdf\"", "\"docx\""), "docx");
    }

    #[test]
    fn bad_sha256_is_rejected() {
        for bad in [
            "abc",
            &SHA_A.to_uppercase(),
            &format!("{SHA_A}0"),
            &SHA_A.replace('b', "g"),
        ] {
            rejects(&entry("a").replace(SHA_A, bad), "sha256");
        }
    }

    #[test]
    fn how_without_manual_is_rejected() {
        rejects(&format!("{}how = \"x\"\n", entry("a")), "manual");
    }

    #[test]
    fn manual_without_how_is_rejected() {
        rejects(&format!("{}manual = true\n", entry("a")), "how");
    }

    #[test]
    fn missing_sha256_on_a_non_manual_entry_is_rejected() {
        rejects(
            &entry("a").replace(&format!("sha256 = \"{SHA_A}\"\n"), ""),
            "sha256",
        );
    }

    /// A temporary tree with `reference/files` and `reference/text`.
    fn tree(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("xtask-refs-{name}-{}", std::process::id()));
        drop(fs::remove_dir_all(&root));
        fs::create_dir_all(root.join("reference/files")).unwrap();
        fs::create_dir_all(root.join("reference/text")).unwrap();
        root
    }

    fn doc(id: &str, format: Format, sha: Option<&str>, manual: bool) -> Doc {
        Doc {
            id: id.to_string(),
            title: "T".into(),
            version: "1".into(),
            url: "https://example.org/".into(),
            sha256: sha.map(str::to_string),
            format,
            licence: "free".into(),
            manual,
            how: manual.then(|| "by hand".to_string()),
        }
    }

    #[test]
    fn status_gives_each_of_the_five_statuses() {
        let root = tree("status");
        let files = root.join("reference/files");
        let text = root.join("reference/text");
        // ok: present, matching, text present
        fs::write(files.join("ok.pdf"), "abc").unwrap();
        fs::write(text.join("ok.txt"), "t").unwrap();
        // changed
        fs::write(files.join("changed.pdf"), "xyz").unwrap();
        // no-text
        fs::write(files.join("notext.md"), "abc").unwrap();
        // unpinned
        fs::write(files.join("unpinned.zip"), "abc").unwrap();
        // an archive needs no text
        fs::write(files.join("archive.zip"), "abc").unwrap();
        let s = |d: &Doc| status(&root, d);
        assert_eq!(
            s(&doc("ok", Format::Pdf, Some(SHA_A), false)),
            RefStatus::Ok
        );
        assert_eq!(
            s(&doc("gone", Format::Pdf, Some(SHA_A), false)),
            RefStatus::Missing
        );
        assert_eq!(
            s(&doc("changed", Format::Pdf, Some(SHA_A), false)),
            RefStatus::Changed
        );
        assert_eq!(
            s(&doc("notext", Format::Md, Some(SHA_A), false)),
            RefStatus::NoText
        );
        assert_eq!(
            s(&doc("unpinned", Format::Zip, None, true)),
            RefStatus::Unpinned(SHA_A.to_string())
        );
        assert_eq!(
            s(&doc("archive", Format::Zip, Some(SHA_A), false)),
            RefStatus::Ok
        );
        drop(fs::remove_dir_all(&root));
    }

    #[test]
    fn place_with_the_right_digest_moves_the_file() {
        let root = tree("place-ok");
        let tmp = root.join("reference/files/a.pdf.part");
        let dest = root.join("reference/files/a.pdf");
        fs::write(&tmp, "abc").unwrap();
        place(&tmp, &dest, SHA_A).unwrap();
        assert_eq!(fs::read_to_string(&dest).unwrap(), "abc");
        assert!(!tmp.exists());
        drop(fs::remove_dir_all(&root));
    }

    #[test]
    fn place_with_a_wrong_digest_leaves_nothing() {
        let root = tree("place-bad");
        let tmp = root.join("reference/files/a.pdf.part");
        let dest = root.join("reference/files/a.pdf");
        fs::write(&tmp, "abc").unwrap();
        let err = place(&tmp, &dest, SHA_Z).unwrap_err();
        assert!(err.contains(SHA_Z) && err.contains(SHA_A), "{err}");
        assert!(!dest.exists() && !tmp.exists());
        drop(fs::remove_dir_all(&root));
    }

    #[test]
    fn curl_args_are_a_vector_with_the_https_limits() {
        let args = curl_args("https://example.org/a b.pdf", Path::new("/t/out"));
        let has = |s: &str| args.iter().any(|a| a == s);
        for flag in [
            "--fail",
            "--proto",
            "=https",
            "--proto-redir",
            "--max-filesize",
            "https://example.org/a b.pdf",
        ] {
            assert!(has(flag), "missing {flag} in {args:?}");
        }
        let at = |s: &str| args.iter().position(|a| a == s).unwrap();
        assert_eq!(args[at("--proto") + 1], "=https");
        assert_eq!(args[at("--proto-redir") + 1], "=https");
        assert_eq!(args[at("--max-filesize") + 1], "1073741824");
        assert_eq!(args[at("--output") + 1], "/t/out");
    }

    /// A fake download that writes "abc" (the digest of `SHA_A`) unless the id is in `fail`.
    fn fake_download<'a>(
        calls: &'a RefCell<Vec<String>>,
        fail: &'a [&str],
    ) -> impl Fn(&Doc, &Path) -> Result<(), String> + 'a {
        move |d, out| {
            calls.borrow_mut().push(d.id.clone());
            if fail.contains(&d.id.as_str()) {
                Err("network down".into())
            } else {
                fs::write(out, "abc").map_err(|e| e.to_string())
            }
        }
    }

    fn fake_convert(_src: &Path, dest: &Path) -> Result<(), String> {
        fs::write(dest, "text").map_err(|e| e.to_string())
    }

    #[test]
    fn a_failed_download_does_not_stop_the_next_entry() {
        let root = tree("fetch-fail");
        let docs = [
            doc("first", Format::Pdf, Some(SHA_A), false),
            doc("second", Format::Pdf, Some(SHA_A), false),
        ];
        let calls = RefCell::new(Vec::new());
        let result = fetch_all(
            &root,
            &docs,
            &fake_download(&calls, &["first"]),
            &fake_convert,
        );
        assert!(result.is_err());
        assert_eq!(*calls.borrow(), ["first", "second"]);
        assert!(root.join("reference/files/second.pdf").is_file());
        assert!(root.join("reference/text/second.txt").is_file());
        assert!(!root.join("reference/files/first.pdf").exists());
        drop(fs::remove_dir_all(&root));
    }

    #[test]
    fn a_failed_conversion_leaves_no_text_and_is_err() {
        let root = tree("fetch-convert");
        let docs = [doc("a", Format::Pdf, Some(SHA_A), false)];
        let calls = RefCell::new(Vec::new());
        let bad = |_: &Path, dest: &Path| -> Result<(), String> {
            fs::write(dest, "half").unwrap();
            Err("pdftotext failed".into())
        };
        assert!(fetch_all(&root, &docs, &fake_download(&calls, &[]), &bad).is_err());
        assert_eq!(
            fs::read_dir(root.join("reference/text")).unwrap().count(),
            0
        );
        drop(fs::remove_dir_all(&root));
    }

    #[test]
    fn a_no_text_entry_is_converted_without_downloading() {
        let root = tree("fetch-notext");
        fs::write(root.join("reference/files/a.pdf"), "abc").unwrap();
        let docs = [doc("a", Format::Pdf, Some(SHA_A), false)];
        let calls = RefCell::new(Vec::new());
        fetch_all(&root, &docs, &fake_download(&calls, &[]), &fake_convert).unwrap();
        assert!(calls.borrow().is_empty());
        assert!(root.join("reference/text/a.txt").is_file());
        drop(fs::remove_dir_all(&root));
    }

    #[test]
    fn a_missing_manual_entry_calls_neither() {
        let root = tree("fetch-manual");
        let docs = [doc("m", Format::Pdf, None, true)];
        let calls = RefCell::new(Vec::new());
        let convert_calls = RefCell::new(0);
        let convert = |_: &Path, _: &Path| -> Result<(), String> {
            *convert_calls.borrow_mut() += 1;
            Ok(())
        };
        fetch_all(&root, &docs, &fake_download(&calls, &[]), &convert).unwrap();
        assert!(calls.borrow().is_empty());
        assert_eq!(*convert_calls.borrow(), 0);
        drop(fs::remove_dir_all(&root));
    }
}
