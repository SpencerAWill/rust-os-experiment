//! `check-settings`: keeps the agent guardrails in `.claude/settings.json` in place.
//!
//! The rules (spec M0.3 §4.3), every finding listed:
//!
//! 1. `permissions.deny` holds every rule of [`REQUIRED_DENY`] (more are allowed).
//! 2. `sandbox.enabled` and `sandbox.failIfUnavailable` are `true`, and
//!    `sandbox.allowUnsandboxedCommands` is `false`.
//! 3. The lists that harden hold their required entries: `sandbox.filesystem.denyRead`
//!    ([`REQUIRED_DENY_READ`]) and `sandbox.credentials.envVars` with mode `deny`
//!    ([`REQUIRED_CREDENTIAL_DENY`]); more are allowed.
//! 4. The lists that relax hold only entries of their allowlist, compared exactly (no prefix,
//!    no wildcard unless listed): `sandbox.excludedCommands` ([`EXCLUDED_ALLOWLIST`]),
//!    `sandbox.filesystem.allowWrite` ([`WRITE_PATH_ALLOWLIST`]), `sandbox.network.allowedDomains`
//!    ([`DOMAIN_ALLOWLIST`]) and `sandbox.network.allowMachLookup` ([`MACH_LOOKUP_ALLOWLIST`],
//!    empty).
//! 5. The settings that run programs or relax the sandbox are refused: the keys of
//!    [`TOP_LEVEL_KEYS`] and the other closed key sets are all the file may hold (any other key,
//!    such as `proxyAuthHelper` or `enabledPlugins`, is a finding: a new key needs an entry with
//!    its reason in this file); `hooks` equals [`expected_hooks`] exactly (the push guard stays
//!    and no other hook is added); the switches of [`FORBIDDEN_TRUE`] are not `true`
//!    (`allowLocalBinding`, `allowAllUnixSockets`, `filesystem.disabled`,
//!    `filesystem.allowGitConfig`, `allowAppleEvents`, `enableWeakerNestedSandbox`,
//!    `enableWeakerNetworkIsolation`, `allowNestedUserNamespaces`, `autoAllowBashIfSandboxed`,
//!    `allowPty`, `enableAllProjectMcpServers`, `skipDangerousModePermissionPrompt`,
//!    `skipAutoPermissionPrompt`, top-level `disableAllHooks`); the keys of
//!    [`FORBIDDEN_ANY_VALUE`] are absent (`network.httpProxyPort`, `socksProxyPort`,
//!    `network.mitmProxy`, `sandbox.seccomp`, `sandbox.mandatoryDenySearchDepth`,
//!    `permissions.defaultMode` as a personal preference, and the top-level `env`, `statusLine`,
//!    `apiKeyHelper`, `otelHeadersHelper`, `awsAuthRefresh`, which run programs outside the
//!    sandbox); the lists of [`MUST_BE_EMPTY`] have no entry (`network.allowUnixSockets`,
//!    `filesystem.allowRead`, `filesystem.allowWriteWithinDeny`, `ignoreViolations`,
//!    `permissions.additionalDirectories`). A key a specific rule covers gets that rule's
//!    message only.
//! 6. `permissions.allow` has no bare tool name (`Bash`, `Read`, `Edit`, `Write`), no `Bash(*)`
//!    and no entry starting `Edit(` or `WebFetch(domain:` ([`FORBIDDEN_ALLOW_PREFIXES`]).
//! 7. The file is valid JSON and each field above has the right type.
//!
//! It reads only `.claude/settings.json` (the shared file, not `settings.local.json`), writes
//! nothing and runs no program. The repository root is the parent of this crate's directory, so
//! the result does not depend on the caller's current directory.

use std::fs;
use std::path::Path;

use serde_json::Value;

/// The file checked, relative to the repository root.
const SETTINGS_PATH: &str = ".claude/settings.json";

/// The `permissions.deny` rules of spec §4.2, matched as exact strings.
const REQUIRED_DENY: &[&str] = &[
    "Bash(sudo *)",
    "Bash(diskutil *)",
    "Bash(dd *)",
    "Bash(hdiutil *)",
    "Bash(launchctl *)",
    "Bash(tmutil *)",
    "Bash(csrutil *)",
    "Bash(nvram *)",
    "Bash(osascript *)",
    "Bash(security *)",
    "Bash(shutdown *)",
    "Bash(reboot *)",
    "Bash(halt *)",
    "Bash(brew install *)",
    "Bash(brew reinstall *)",
    "Bash(brew upgrade *)",
    "Bash(brew uninstall *)",
    "Bash(brew remove *)",
    "Bash(brew rm *)",
    "Bash(brew autoremove *)",
    "Bash(brew bundle *)",
    "Read(~/.ssh/**)",
    "Read(~/.aws/**)",
    "Read(~/.gnupg/**)",
    "Read(~/.config/gh/**)",
    "Read(~/Library/Keychains/**)",
    "Read(~/.netrc)",
    "Read(~/.git-credentials)",
    "Read(~/.cargo/credentials.toml)",
    "Read(~/.cargo/credentials)",
    "Edit(/.git/hooks/**)",
    "Edit(/.git/config)",
    "Edit(/.claude/settings.local.json)",
];

/// The paths `sandbox.filesystem.denyRead` must hold (spec §4.1.5).
const REQUIRED_DENY_READ: &[&str] = &[
    "~/.ssh",
    "~/.aws",
    "~/.gnupg",
    "~/.config/gh",
    "~/Library/Keychains",
    "~/.netrc",
    "~/.git-credentials",
    "~/.cargo/credentials.toml",
    "~/.cargo/credentials",
];

/// The environment variables a sandboxed command must not see, each with mode `deny` (§4.1.6).
const REQUIRED_CREDENTIAL_DENY: &[&str] = &[
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "GH_ENTERPRISE_TOKEN",
    "CARGO_REGISTRY_TOKEN",
];

/// The commands that may run outside the sandbox (`sandbox.excludedCommands`), compared exactly.
///
/// Each entry needs a reason, written in a comment above it. All of them fail sandboxed because
/// the `Read` deny rules of spec §4.2.2 also deny `~/.config/gh` and the keychain to sandboxed
/// commands (`log.md`, 2026-11-28); an excluded command runs alone in its Bash call. Not `gh *`,
/// which would also exclude `gh extension` and `gh alias --shell`.
const EXCLUDED_ALLOWLIST: &[&str] = &[
    // The loop's REST and GraphQL calls; needs the token in `~/.config/gh`.
    "gh api *",
    // Reads and writes issues (chunk issues, follow-ups); needs the token.
    "gh issue *",
    // Opens and reads pull requests; needs the token.
    "gh pr *",
    // Reads and updates the "OS backlog" Project (user-owned Project 8); needs the token.
    "gh project *",
    // Creates and edits labels; needs the token.
    "gh label *",
    // Searches issues and pull requests; needs the token.
    "gh search *",
    // Shows whether `gh` is logged in; reads the token's config.
    "gh auth status",
    // Pushes the chunk branch to `origin` (the loop's form); the credential helper or key lives
    // in a path the sandbox denies (`osxkeychain` and the keychain on macOS). Pinned to `origin`
    // because `git push *` also matched `--receive-pack=<cmd> <path>`, `--exec=<cmd>` and
    // local-path remotes, which run any command outside the sandbox; `guard-push` refuses those
    // flags (spec §4.7).
    "git push origin *",
    // The same with `-u`, for the first push of a branch.
    "git push -u origin *",
];

/// The paths outside the repository a sandboxed command may write (`sandbox.filesystem.allowWrite`).
const WRITE_PATH_ALLOWLIST: &[&str] = &[
    // The lock `cargo deny` takes when it fetches the RustSec advisory database
    // (`log.md`, 2026-11-28); `~/.cargo` as a whole and `~/.rustup` were not needed.
    "~/.cargo/advisory-dbs",
];

/// The hosts a sandboxed command may reach without approval (`sandbox.network.allowedDomains`).
const DOMAIN_ALLOWLIST: &[&str] = &[
    // `cargo deny` fetches `https://github.com/RustSec/advisory-db`, and `git fetch` reads the
    // public repository.
    "github.com",
];

/// The Mach services a sandboxed command may look up (`sandbox.network.allowMachLookup`).
///
/// Empty: `com.apple.sysmond` would fix only the `doctor` sweep's `pgrep` listing and would let
/// every sandboxed command list the host's processes and their arguments (spec §4.1.3).
const MACH_LOOKUP_ALLOWLIST: &[&str] = &[];

/// Booleans that relax the sandbox or the permission rules; `true` is a finding.
const FORBIDDEN_TRUE: &[&[&str]] = &[
    &["sandbox", "network", "allowLocalBinding"],
    &["sandbox", "network", "allowAllUnixSockets"],
    &["sandbox", "filesystem", "disabled"],
    &["sandbox", "filesystem", "allowGitConfig"],
    &["sandbox", "allowAppleEvents"],
    &["sandbox", "enableWeakerNestedSandbox"],
    &["sandbox", "enableWeakerNetworkIsolation"],
    &["sandbox", "allowNestedUserNamespaces"],
    &["sandbox", "autoAllowBashIfSandboxed"],
    &["disableAllHooks"],
    &["sandbox", "allowPty"],
    // MCP servers run outside the sandbox.
    &["enableAllProjectMcpServers"],
    // Approval preferences, personal (spec §2).
    &["skipDangerousModePermissionPrompt"],
    &["skipAutoPermissionPrompt"],
];

/// Keys that must be absent: any value relaxes the sandbox (a proxy port sends traffic around
/// the domain allowlist) or is a personal preference.
const FORBIDDEN_ANY_VALUE: &[&[&str]] = &[
    &["sandbox", "network", "httpProxyPort"],
    &["sandbox", "network", "socksProxyPort"],
    // Lowering the depth relaxes; the default is used.
    &["sandbox", "mandatoryDenySearchDepth"],
    // Routes domains to an upstream proxy.
    &["sandbox", "network", "mitmProxy"],
    // Custom seccomp binaries.
    &["sandbox", "seccomp"],
    // The default mode is a personal approval preference (spec §2).
    &["permissions", "defaultMode"],
    // `env` can set `GIT_CONFIG_*` (such as `core.hooksPath`) for the excluded `git push`; the
    // others run a program outside the sandbox.
    &["env"],
    &["statusLine"],
    &["apiKeyHelper"],
    &["otelHeadersHelper"],
    &["awsAuthRefresh"],
];

/// The only `hooks` value allowed: the push guard (spec §4.7), written here and not read from
/// the file. Removing or changing it drops the guard on the excluded `git push`, and an added
/// hook would run outside the sandbox.
fn expected_hooks() -> Value {
    serde_json::json!({
        "PreToolUse": [{
            "matcher": "Bash",
            "hooks": [{
                "type": "command",
                "command": "sh \"$CLAUDE_PROJECT_DIR/.claude/hooks/guard-push.sh\"",
                "timeout": 120
            }]
        }]
    })
}

// The closed key set, one constant per object level of the shared settings, each holding exactly
// the keys `.claude/settings.json` uses. The schema has many keys that run programs or relax the
// sandbox (`awsCredentialExport`, `gcpAuthRefresh`, `proxyAuthHelper`, `fileSuggestion`,
// `subagentStatusLine`, `processWrapper`, `enabledPlugins`, `extraKnownMarketplaces`, ...), and
// refusing them one by one missed some (review round 3, log.md). Any other key is a finding: a
// new key needs an entry here with its reason. `hooks` is pinned whole by `expected_hooks`.

/// Keys allowed at the top level (`hooks`: the push guard, pinned separately).
const TOP_LEVEL_KEYS: &[&str] = &["permissions", "sandbox", "hooks"];

/// Keys allowed in `permissions`.
const PERMISSIONS_KEYS: &[&str] = &["allow", "deny"];

/// Keys allowed in `sandbox`.
const SANDBOX_KEYS: &[&str] = &[
    "enabled",
    "allowUnsandboxedCommands",
    "failIfUnavailable",
    "excludedCommands",
    "filesystem",
    "network",
    "credentials",
];

/// Keys allowed in `sandbox.filesystem`.
const FILESYSTEM_KEYS: &[&str] = &["allowWrite", "denyRead"];

/// Keys allowed in `sandbox.network`.
const NETWORK_KEYS: &[&str] = &["allowedDomains"];

/// Keys allowed in `sandbox.credentials`.
const CREDENTIALS_KEYS: &[&str] = &["envVars"];

/// Whether a specific rule (forbidden, must-be-empty or allowlist) covers the key at `path`.
fn specifically_handled(path: &[&str]) -> bool {
    FORBIDDEN_TRUE
        .iter()
        .chain(FORBIDDEN_ANY_VALUE)
        .chain(MUST_BE_EMPTY)
        .any(|p| *p == path)
        || path == ["sandbox", "network", "allowMachLookup"]
}

/// A finding for each key of the object at `path` that is not in `allowed`.
fn closed_keys(root: &Value, path: &[&str], allowed: &[&str], findings: &mut Vec<String>) {
    let name = if path.is_empty() {
        "the top level".to_string()
    } else {
        path.join(".")
    };
    let map = match get(root, path).or((path.is_empty()).then_some(root)) {
        Some(Value::Object(map)) => map,
        Some(_) => {
            findings.push(format!("{name} must be an object"));
            return;
        }
        None => return,
    };
    for key in map.keys().filter(|k| !allowed.contains(&k.as_str())) {
        // A key a specific rule below already refuses is reported there, with a clearer message.
        let mut full = path.to_vec();
        full.push(key);
        if specifically_handled(&full) {
            continue;
        }
        findings.push(format!(
            "{name} has the key `{key}`, which is not allowed; a new key needs an entry with its \
             reason in xtask/src/check_settings.rs"
        ));
    }
}

/// Lists that must stay empty (an object counts by its keys).
const MUST_BE_EMPTY: &[&[&str]] = &[
    &["sandbox", "network", "allowUnixSockets"],
    &["sandbox", "filesystem", "allowRead"],
    &["sandbox", "filesystem", "allowWriteWithinDeny"],
    &["sandbox", "ignoreViolations"],
    &["permissions", "additionalDirectories"],
];

/// Prefixes of `permissions.allow` entries that grant sandbox writes or hosts.
const FORBIDDEN_ALLOW_PREFIXES: &[&str] = &["Edit(", "WebFetch(domain:"];

/// Tool names that, bare in `permissions.allow`, allow every call of the tool.
const BARE_TOOLS: &[&str] = &["Bash", "Read", "Edit", "Write"];

/// The value at `path` below `root`, if every key exists.
fn get<'a>(root: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(root, |v, key| v.get(key))
}

/// The strings of the array at `path` (an absent key is an empty list). A wrong type is a
/// finding, and so is each element that is not a string.
fn string_list<'a>(root: &'a Value, path: &[&str], findings: &mut Vec<String>) -> Vec<&'a str> {
    let name = path.join(".");
    match get(root, path) {
        None => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| {
                let s = item.as_str();
                if s.is_none() {
                    findings.push(format!("{name} has an entry that is not a string: {item}"));
                }
                s
            })
            .collect(),
        Some(other) => {
            findings.push(format!("{name} must be an array of strings, found {other}"));
            Vec::new()
        }
    }
}

/// A finding unless the boolean at `path` is exactly `expected`.
fn require_bool(root: &Value, path: &[&str], expected: bool, findings: &mut Vec<String>) {
    let name = path.join(".");
    match get(root, path) {
        Some(Value::Bool(b)) if *b == expected => {}
        Some(Value::Bool(_)) => findings.push(format!("{name} must be {expected}")),
        Some(other) => findings.push(format!("{name} must be {expected}, found {other}")),
        None => findings.push(format!("{name} is missing and must be {expected}")),
    }
}

/// A finding unless the boolean at `path` is `false` or absent (a switch that relaxes).
fn forbid_true(root: &Value, path: &[&str], findings: &mut Vec<String>) {
    let name = path.join(".");
    match get(root, path) {
        None | Some(Value::Bool(false)) => {}
        Some(other) => findings.push(format!("{name} must be false or absent, found {other}")),
    }
}

/// A finding for each entry of the list at `path`, or each key of the object there.
fn require_empty(root: &Value, path: &[&str], findings: &mut Vec<String>) {
    match get(root, path) {
        Some(Value::Object(map)) => {
            for key in map.keys() {
                findings.push(format!(
                    "{} has the key `{key}`, it must be empty",
                    path.join(".")
                ));
            }
        }
        _ => allow_only(root, path, &[], findings),
    }
}

/// A finding for each missing entry of `required` in the list at `path`.
fn require_entries(root: &Value, path: &[&str], required: &[&str], findings: &mut Vec<String>) {
    let have = string_list(root, path, findings);
    for entry in required {
        if !have.contains(entry) {
            findings.push(format!("{} is missing `{entry}`", path.join(".")));
        }
    }
}

/// A finding for each entry of the list at `path` that is not in `allowed` (exact comparison).
fn allow_only(root: &Value, path: &[&str], allowed: &[&str], findings: &mut Vec<String>) {
    for entry in string_list(root, path, findings) {
        if !allowed.contains(&entry) {
            findings.push(format!(
                "{} has `{entry}`, which is not in the allowlist in xtask/src/check_settings.rs",
                path.join(".")
            ));
        }
    }
}

/// Rule 3, credentials: each required variable has an entry with that name and mode `deny`.
fn check_credentials(root: &Value, findings: &mut Vec<String>) {
    let path = ["sandbox", "credentials", "envVars"];
    let entries: &[Value] = match get(root, &path) {
        None => &[],
        Some(Value::Array(items)) => items,
        Some(other) => {
            findings.push(format!(
                "{} must be an array, found {other}",
                path.join(".")
            ));
            return;
        }
    };
    for name in REQUIRED_CREDENTIAL_DENY {
        let denied = entries
            .iter()
            .any(|e| e["name"] == *name && e["mode"] == "deny");
        if !denied {
            findings.push(format!(
                "{} has no entry with name `{name}` and mode `deny`",
                path.join(".")
            ));
        }
    }
}

/// Every finding in `settings`, one message each.
fn check(settings: &Value) -> Vec<String> {
    if !settings.is_object() {
        return vec!["the settings file must hold a JSON object".to_string()];
    }
    let mut findings = Vec::new();
    let f = &mut findings;

    let levels: [(&[&str], &[&str]); 6] = [
        (&[], TOP_LEVEL_KEYS),
        (&["permissions"], PERMISSIONS_KEYS),
        (&["sandbox"], SANDBOX_KEYS),
        (&["sandbox", "filesystem"], FILESYSTEM_KEYS),
        (&["sandbox", "network"], NETWORK_KEYS),
        (&["sandbox", "credentials"], CREDENTIALS_KEYS),
    ];
    for (path, allowed) in levels {
        closed_keys(settings, path, allowed, f);
    }

    require_entries(settings, &["permissions", "deny"], REQUIRED_DENY, f);
    for entry in string_list(settings, &["permissions", "allow"], f) {
        if BARE_TOOLS.contains(&entry) || entry == "Bash(*)" {
            f.push(format!(
                "permissions.allow has `{entry}`, which allows every call of the tool"
            ));
        } else if FORBIDDEN_ALLOW_PREFIXES
            .iter()
            .any(|p| entry.starts_with(p))
        {
            f.push(format!(
                "permissions.allow has `{entry}`, which grants sandbox writes or hosts"
            ));
        }
    }

    require_bool(settings, &["sandbox", "enabled"], true, f);
    require_bool(settings, &["sandbox", "failIfUnavailable"], true, f);
    require_bool(settings, &["sandbox", "allowUnsandboxedCommands"], false, f);

    require_entries(
        settings,
        &["sandbox", "filesystem", "denyRead"],
        REQUIRED_DENY_READ,
        f,
    );
    check_credentials(settings, f);

    let allowlists: [(&[&str], &[&str]); 4] = [
        (&["sandbox", "excludedCommands"], EXCLUDED_ALLOWLIST),
        (
            &["sandbox", "filesystem", "allowWrite"],
            WRITE_PATH_ALLOWLIST,
        ),
        (&["sandbox", "network", "allowedDomains"], DOMAIN_ALLOWLIST),
        (
            &["sandbox", "network", "allowMachLookup"],
            MACH_LOOKUP_ALLOWLIST,
        ),
    ];
    for (path, allowed) in allowlists {
        allow_only(settings, path, allowed, f);
    }

    for path in FORBIDDEN_TRUE {
        forbid_true(settings, path, f);
    }
    for path in FORBIDDEN_ANY_VALUE {
        if let Some(value) = get(settings, path) {
            f.push(format!("{} must be absent, found {value}", path.join(".")));
        }
    }
    for path in MUST_BE_EMPTY {
        require_empty(settings, path, f);
    }

    match settings.get("hooks") {
        None => f.push("hooks is missing; the push guard hook must stay".to_string()),
        Some(hooks) if *hooks != expected_hooks() => f.push(format!(
            "hooks differs from the push guard hook in xtask/src/check_settings.rs, found {hooks}"
        )),
        Some(_) => {}
    }

    findings
}

/// Every finding in the text of a settings file (rule 7: invalid JSON is one).
fn check_text(text: &str) -> Vec<String> {
    match serde_json::from_str::<Value>(text) {
        Ok(settings) => check(&settings),
        Err(e) => vec![format!("{SETTINGS_PATH} is not valid JSON: {e}")],
    }
}

pub(crate) fn run() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("cannot find the repository root")?;
    let text = fs::read_to_string(root.join(SETTINGS_PATH))
        .map_err(|e| format!("{SETTINGS_PATH}: {e}"))?;
    let findings = check_text(&text);
    if findings.is_empty() {
        println!("check-settings: ok");
        Ok(())
    } else {
        for finding in &findings {
            println!("{SETTINGS_PATH}: {finding}");
        }
        Err(format!(
            "check-settings: {} finding(s) in {SETTINGS_PATH}",
            findings.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const COMMITTED: &str = include_str!("../../.claude/settings.json");

    fn committed() -> Value {
        serde_json::from_str(COMMITTED).expect("the committed settings are valid JSON")
    }

    /// The findings after `change` is applied to the committed settings.
    fn findings_after(change: impl FnOnce(&mut Value)) -> Vec<String> {
        let mut settings = committed();
        change(&mut settings);
        check(&settings)
    }

    /// Asserts exactly one finding, and that it mentions `needle`.
    fn assert_one(findings: &[String], needle: &str) {
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].contains(needle), "{findings:?}");
    }

    fn remove_entry(list: &mut Value, entry: &str) {
        list.as_array_mut().unwrap().retain(|e| e != entry);
    }

    #[test]
    fn the_committed_file_passes() {
        assert_eq!(check_text(COMMITTED), Vec::<String>::new());
    }

    #[test]
    fn a_crlf_copy_of_the_committed_file_passes() {
        let crlf = COMMITTED.replace("\r\n", "\n").replace('\n', "\r\n");
        assert!(crlf.contains("\r\n"));
        assert_eq!(check_text(&crlf), Vec::<String>::new());
    }

    #[test]
    fn every_required_deny_rule_is_in_the_committed_file() {
        let settings = committed();
        let deny = settings["permissions"]["deny"].as_array().unwrap();
        for rule in REQUIRED_DENY {
            assert!(deny.iter().any(|d| d == rule), "{rule}");
        }
    }

    #[test]
    fn a_missing_deny_rule_is_named() {
        let findings = findings_after(|s| {
            remove_entry(&mut s["permissions"]["deny"], "Bash(security *)");
        });
        assert_one(&findings, "Bash(security *)");
    }

    #[test]
    fn a_deny_rule_in_another_form_does_not_count() {
        let findings = findings_after(|s| {
            let deny = s["permissions"]["deny"].as_array_mut().unwrap();
            let sudo = deny.iter_mut().find(|d| *d == "Bash(sudo *)").unwrap();
            *sudo = json!("Bash(sudo:*)");
        });
        assert_one(&findings, "Bash(sudo *)");
    }

    #[test]
    fn sandbox_enabled_false_or_absent_is_a_finding() {
        let off = findings_after(|s| s["sandbox"]["enabled"] = json!(false));
        assert_one(&off, "sandbox.enabled");
        let absent = findings_after(|s| {
            s["sandbox"].as_object_mut().unwrap().remove("enabled");
        });
        assert_one(&absent, "sandbox.enabled");
    }

    #[test]
    fn fail_if_unavailable_false_or_absent_is_a_finding() {
        let off = findings_after(|s| s["sandbox"]["failIfUnavailable"] = json!(false));
        assert_one(&off, "sandbox.failIfUnavailable");
        let absent = findings_after(|s| {
            s["sandbox"]
                .as_object_mut()
                .unwrap()
                .remove("failIfUnavailable");
        });
        assert_one(&absent, "sandbox.failIfUnavailable");
    }

    #[test]
    fn unsandboxed_commands_allowed_is_a_finding() {
        let findings = findings_after(|s| s["sandbox"]["allowUnsandboxedCommands"] = json!(true));
        assert_one(&findings, "sandbox.allowUnsandboxedCommands");
    }

    #[test]
    fn a_missing_deny_read_path_is_named() {
        let findings = findings_after(|s| {
            remove_entry(&mut s["sandbox"]["filesystem"]["denyRead"], "~/.config/gh");
        });
        assert_one(&findings, "~/.config/gh");
    }

    #[test]
    fn a_missing_credential_deny_is_named() {
        let findings = findings_after(|s| {
            s["sandbox"]["credentials"]["envVars"]
                .as_array_mut()
                .unwrap()
                .retain(|e| e["name"] != "GH_TOKEN");
        });
        assert_one(&findings, "GH_TOKEN");
    }

    #[test]
    fn a_credential_with_the_right_name_and_mode_mask_is_a_finding() {
        let findings = findings_after(|s| {
            let vars = s["sandbox"]["credentials"]["envVars"]
                .as_array_mut()
                .unwrap();
            let gh = vars.iter_mut().find(|e| e["name"] == "GH_TOKEN").unwrap();
            gh["mode"] = json!("mask");
        });
        assert_one(&findings, "GH_TOKEN");
    }

    #[test]
    fn an_unlisted_exclusion_is_a_finding_including_wildcards() {
        for entry in [
            "gh *",
            "*",
            "~",
            "git push",
            "git push *",
            "git push --receive-pack=x *",
            "git push *; true",
        ] {
            let findings = findings_after(|s| {
                s["sandbox"]["excludedCommands"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(entry));
            });
            assert_one(&findings, entry);
        }
    }

    #[test]
    fn an_unlisted_write_path_is_a_finding_by_exact_match() {
        // `~/.cargo` is a prefix of the listed path and `~/.cargo/..` starts with it.
        for entry in [
            "~",
            "~/.cargo",
            "~/.cargo/..",
            "~/.cargo/advisory-dbs/",
            "*",
        ] {
            let findings = findings_after(|s| {
                s["sandbox"]["filesystem"]["allowWrite"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(entry));
            });
            assert_one(&findings, entry);
        }
    }

    #[test]
    fn an_unlisted_domain_is_a_finding_including_a_wildcard_over_a_listed_one() {
        for entry in ["*", "*.github.com", "example.com"] {
            let findings = findings_after(|s| {
                s["sandbox"]["network"]["allowedDomains"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(entry));
            });
            assert_one(&findings, entry);
        }
    }

    #[test]
    fn any_mach_service_is_a_finding() {
        let findings = findings_after(|s| {
            s["sandbox"]["network"]["allowMachLookup"] = json!(["com.apple.sysmond"]);
        });
        assert_one(&findings, "com.apple.sysmond");
    }

    #[test]
    fn local_binding_true_is_a_finding() {
        let findings =
            findings_after(|s| s["sandbox"]["network"]["allowLocalBinding"] = json!(true));
        assert_one(&findings, "allowLocalBinding");
    }

    #[test]
    fn a_unix_socket_entry_is_a_finding() {
        let findings = findings_after(|s| {
            s["sandbox"]["network"]["allowUnixSockets"] = json!(["/var/run/docker.sock"]);
        });
        assert_one(&findings, "/var/run/docker.sock");
    }

    #[test]
    fn all_unix_sockets_true_is_a_finding() {
        let findings =
            findings_after(|s| s["sandbox"]["network"]["allowAllUnixSockets"] = json!(true));
        assert_one(&findings, "allowAllUnixSockets");
    }

    #[test]
    fn an_allow_read_entry_is_a_finding() {
        let findings = findings_after(|s| {
            s["sandbox"]["filesystem"]["allowRead"] = json!(["~/.config/gh"]);
        });
        assert_one(&findings, "allowRead");
    }

    #[test]
    fn a_bare_tool_allow_is_a_finding() {
        for entry in ["Bash", "Read", "Edit", "Write", "Bash(*)"] {
            let findings = findings_after(|s| {
                s["permissions"]["allow"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(entry));
            });
            assert_one(&findings, &format!("`{entry}`"));
        }
    }

    #[test]
    fn disable_all_hooks_true_is_a_finding() {
        let findings = findings_after(|s| s["disableAllHooks"] = json!(true));
        assert_one(&findings, "disableAllHooks");
    }

    #[test]
    fn allow_git_config_true_is_a_finding() {
        let findings =
            findings_after(|s| s["sandbox"]["filesystem"]["allowGitConfig"] = json!(true));
        assert_one(&findings, "allowGitConfig");
    }

    #[test]
    fn filesystem_disabled_true_is_a_finding() {
        let findings = findings_after(|s| s["sandbox"]["filesystem"]["disabled"] = json!(true));
        assert_one(&findings, "filesystem.disabled");
    }

    #[test]
    fn allow_apple_events_true_is_a_finding() {
        let findings = findings_after(|s| s["sandbox"]["allowAppleEvents"] = json!(true));
        assert_one(&findings, "allowAppleEvents");
    }

    #[test]
    fn weaker_nested_sandbox_true_is_a_finding() {
        let findings = findings_after(|s| s["sandbox"]["enableWeakerNestedSandbox"] = json!(true));
        assert_one(&findings, "enableWeakerNestedSandbox");
    }

    #[test]
    fn weaker_network_isolation_true_is_a_finding() {
        let findings =
            findings_after(|s| s["sandbox"]["enableWeakerNetworkIsolation"] = json!(true));
        assert_one(&findings, "enableWeakerNetworkIsolation");
    }

    #[test]
    fn nested_user_namespaces_true_is_a_finding() {
        let findings = findings_after(|s| s["sandbox"]["allowNestedUserNamespaces"] = json!(true));
        assert_one(&findings, "allowNestedUserNamespaces");
    }

    #[test]
    fn auto_allow_bash_if_sandboxed_true_is_a_finding() {
        let findings = findings_after(|s| s["sandbox"]["autoAllowBashIfSandboxed"] = json!(true));
        assert_one(&findings, "autoAllowBashIfSandboxed");
    }

    #[test]
    fn any_proxy_port_is_a_finding() {
        for key in ["httpProxyPort", "socksProxyPort"] {
            for value in [json!(8080), json!(0), json!("8080")] {
                let findings = findings_after(|s| s["sandbox"]["network"][key] = value.clone());
                assert_one(&findings, key);
            }
        }
    }

    #[test]
    fn an_ignore_violations_entry_is_a_finding() {
        let findings = findings_after(|s| {
            s["sandbox"]["ignoreViolations"] = json!({ "*": ["/usr/bin"] });
        });
        assert_one(&findings, "ignoreViolations");
        let empty = findings_after(|s| s["sandbox"]["ignoreViolations"] = json!({}));
        assert!(empty.is_empty(), "{empty:?}");
    }

    #[test]
    fn an_allow_write_within_deny_entry_is_a_finding() {
        let findings = findings_after(|s| {
            s["sandbox"]["filesystem"]["allowWriteWithinDeny"] = json!(["~/.ssh"]);
        });
        assert_one(&findings, "~/.ssh");
    }

    #[test]
    fn an_additional_directory_is_a_finding() {
        let findings = findings_after(|s| {
            s["permissions"]["additionalDirectories"] = json!(["/tmp"]);
        });
        assert_one(&findings, "additionalDirectories");
    }

    #[test]
    fn any_default_mode_is_a_finding() {
        for mode in ["bypassPermissions", "plan", "acceptEdits", "default"] {
            let findings = findings_after(|s| s["permissions"]["defaultMode"] = json!(mode));
            assert_one(&findings, "defaultMode");
        }
    }

    #[test]
    fn allow_pty_true_is_a_finding() {
        let findings = findings_after(|s| s["sandbox"]["allowPty"] = json!(true));
        assert_one(&findings, "allowPty");
    }

    #[test]
    fn enable_all_project_mcp_servers_true_is_a_finding() {
        let findings = findings_after(|s| s["enableAllProjectMcpServers"] = json!(true));
        assert_one(&findings, "enableAllProjectMcpServers");
    }

    #[test]
    fn skip_dangerous_mode_permission_prompt_true_is_a_finding() {
        let findings = findings_after(|s| s["skipDangerousModePermissionPrompt"] = json!(true));
        assert_one(&findings, "skipDangerousModePermissionPrompt");
    }

    #[test]
    fn skip_auto_permission_prompt_true_is_a_finding() {
        let findings = findings_after(|s| s["skipAutoPermissionPrompt"] = json!(true));
        assert_one(&findings, "skipAutoPermissionPrompt");
    }

    #[test]
    fn a_mandatory_deny_search_depth_is_a_finding() {
        let findings = findings_after(|s| s["sandbox"]["mandatoryDenySearchDepth"] = json!(1));
        assert_one(&findings, "mandatoryDenySearchDepth");
    }

    #[test]
    fn a_mitm_proxy_is_a_finding() {
        let findings = findings_after(|s| {
            s["sandbox"]["network"]["mitmProxy"] =
                json!({ "socketPath": "/tmp/x", "domains": ["*"] });
        });
        assert_one(&findings, "mitmProxy");
    }

    #[test]
    fn a_seccomp_config_is_a_finding() {
        let findings = findings_after(|s| s["sandbox"]["seccomp"] = json!({ "bpfPath": "/tmp/x" }));
        assert_one(&findings, "sandbox.seccomp");
    }

    #[test]
    fn an_edit_allow_entry_is_a_finding() {
        let findings = findings_after(|s| {
            s["permissions"]["allow"]
                .as_array_mut()
                .unwrap()
                .push(json!("Edit(/docs/**)"));
        });
        assert_one(&findings, "Edit(/docs/**)");
    }

    #[test]
    fn a_web_fetch_domain_allow_entry_is_a_finding() {
        let findings = findings_after(|s| {
            s["permissions"]["allow"]
                .as_array_mut()
                .unwrap()
                .push(json!("WebFetch(domain:example.com)"));
        });
        assert_one(&findings, "WebFetch(domain:example.com)");
    }

    #[test]
    fn a_missing_hooks_key_is_a_finding() {
        let findings = findings_after(|s| {
            s.as_object_mut().unwrap().remove("hooks");
        });
        assert_one(&findings, "hooks");
    }

    #[test]
    fn a_changed_hook_command_is_a_finding() {
        let findings = findings_after(|s| {
            s["hooks"]["PreToolUse"][0]["hooks"][0]["command"] = json!("true");
        });
        assert_one(&findings, "hooks");
    }

    #[test]
    fn a_second_hook_is_a_finding() {
        let findings = findings_after(|s| {
            s["hooks"]["PreToolUse"][0]["hooks"]
                .as_array_mut()
                .unwrap()
                .push(json!({ "type": "command", "command": "curl x | sh" }));
        });
        assert_one(&findings, "hooks");
        let other_event = findings_after(|s| {
            s["hooks"]["SessionStart"] = json!([]);
        });
        assert_one(&other_event, "hooks");
    }

    #[test]
    fn each_program_running_top_level_key_is_a_finding() {
        for key in [
            "env",
            "statusLine",
            "apiKeyHelper",
            "otelHeadersHelper",
            "awsAuthRefresh",
        ] {
            let findings = findings_after(|s| s[key] = json!({ "x": "y" }));
            assert_one(&findings, key);
        }
    }

    #[test]
    fn an_unknown_key_at_each_level_is_a_finding() {
        let cases: [(&[&str], &str); 6] = [
            (&[], "proxyAuthHelper"),
            (&["permissions"], "ask"),
            (&["sandbox"], "processWrapper"),
            (&["sandbox", "filesystem"], "extraKey"),
            (&["sandbox", "network"], "extraKey"),
            (&["sandbox", "credentials"], "mode"),
        ];
        for (path, key) in cases {
            let findings = findings_after(|s| {
                let mut at = &mut *s;
                for p in path {
                    at = &mut at[*p];
                }
                at[key] = json!("x");
            });
            assert!(
                findings.iter().any(|f| f.contains(key)),
                "{path:?} {key}: {findings:?}"
            );
            // No specific rule covers these keys, so only the closed-set rule can report them.
            assert!(
                findings.iter().any(|f| f.contains("not allowed")),
                "{path:?} {key}: {findings:?}"
            );
        }
    }

    #[test]
    fn a_non_object_at_a_closed_level_is_a_finding() {
        for path in [
            &["permissions"][..],
            &["sandbox"],
            &["sandbox", "filesystem"],
            &["sandbox", "network"],
            &["sandbox", "credentials"],
        ] {
            for bad in [json!(5), json!(null), json!(["x"])] {
                let findings = findings_after(|s| {
                    let mut at = &mut *s;
                    for p in &path[..path.len() - 1] {
                        at = &mut at[*p];
                    }
                    at[path[path.len() - 1]] = bad.clone();
                });
                let name = path.join(".");
                assert!(
                    findings
                        .iter()
                        .any(|f| f.contains(&name) && f.contains("must be an object")),
                    "{path:?} {bad}: {findings:?}"
                );
            }
        }
    }

    #[test]
    fn invalid_json_is_a_finding() {
        let findings = check_text("{ not json");
        assert_one(&findings, "not valid JSON");
    }

    #[test]
    fn a_wrong_type_is_a_finding() {
        let string_enabled = findings_after(|s| s["sandbox"]["enabled"] = json!("true"));
        assert_one(&string_enabled, "sandbox.enabled");
        let object_deny = findings_after(|s| s["permissions"]["deny"] = json!({}));
        assert!(
            object_deny
                .iter()
                .any(|f| f.contains("permissions.deny must be an array")),
            "{object_deny:?}"
        );
        let number_entry = findings_after(|s| {
            s["sandbox"]["excludedCommands"]
                .as_array_mut()
                .unwrap()
                .push(json!(1));
        });
        assert_one(&number_entry, "not a string");
        assert_eq!(check(&json!([])).len(), 1);
    }

    #[test]
    fn every_finding_is_listed() {
        let findings = findings_after(|s| {
            s["sandbox"]["enabled"] = json!(false);
            s["sandbox"]["network"]["allowLocalBinding"] = json!(true);
            remove_entry(&mut s["permissions"]["deny"], "Bash(dd *)");
        });
        assert_eq!(findings.len(), 3, "{findings:?}");
    }
}
