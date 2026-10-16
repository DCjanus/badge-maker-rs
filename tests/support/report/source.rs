//! Provenance for the revision under test.
//!
//! The derivation is a pure function over [`SourceInput`] so that CI, pull
//! request, and local behavior can be tested with small fixtures instead of
//! mutating the process environment.

use serde::Serialize;

/// Repository state that environment variables cannot describe.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RepoState {
    /// The working tree has uncommitted or untracked changes.
    pub dirty: bool,
    /// `HEAD` is a merge commit, which is what GitHub checks out for pull
    /// request runs.
    pub merge_commit: bool,
}

/// Raw provenance inputs for one report run.
#[derive(Debug, Clone, Default)]
pub struct SourceInput {
    pub server_url: Option<String>,
    pub repository: Option<String>,
    pub run_id: Option<String>,
    pub run_attempt: Option<String>,
    pub git_ref: Option<String>,
    pub event_payload: Option<String>,
    pub cargo_repository: Option<String>,
    pub commit_sha: String,
    pub repo_state: RepoState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Ci,
    Local,
}

/// Source row data for the report info panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceContext {
    pub kind: SourceKind,
    /// `owner/name` when the environment or the crate metadata identifies it.
    pub repository: Option<String>,
    pub ci_label: String,
    pub ci_url: Option<String>,
    pub commit_sha: String,
    pub commit_short: String,
    pub commit_url: Option<String>,
    pub pull_request_number: Option<u64>,
    pub pull_request_url: Option<String>,
    pub git_ref: Option<String>,
    /// `HEAD` is a merge commit, which is what GitHub checks out for pull
    /// request runs.
    pub merge_commit: bool,
    /// A local run rendered uncommitted working-tree changes.
    pub local_changes: bool,
    /// One short sentence about which revision the artifacts describe.
    pub revision_note: String,
}

impl SourceContext {
    pub fn is_ci(&self) -> bool {
        self.kind == SourceKind::Ci
    }
}

/// Derives the source row without touching the process environment.
pub fn derive(input: &SourceInput) -> SourceContext {
    let run_id = trimmed(input.run_id.as_deref());
    let run_attempt = trimmed(input.run_attempt.as_deref());
    let git_ref = trimmed(input.git_ref.as_deref()).map(str::to_owned);
    let kind = if run_id.is_some() {
        SourceKind::Ci
    } else {
        SourceKind::Local
    };

    let repository = trimmed(input.repository.as_deref())
        .map(str::to_owned)
        .or_else(|| trimmed(input.cargo_repository.as_deref()).and_then(repository_slug));
    let remote = match (trimmed(input.server_url.as_deref()), &repository) {
        (Some(server), Some(repository)) => {
            Some(format!("{}/{repository}", server.trim_end_matches('/')))
        }
        _ => None,
    };

    let ci_url = match (run_id, run_attempt, remote.as_deref()) {
        (Some(run_id), Some(attempt), Some(remote)) => {
            Some(format!("{remote}/actions/runs/{run_id}/attempts/{attempt}"))
        }
        (Some(run_id), None, Some(remote)) => Some(format!("{remote}/actions/runs/{run_id}")),
        _ => None,
    };
    let ci_label = match (run_id, run_attempt) {
        (Some(run_id), Some(attempt)) => format!("{run_id} / attempt {attempt}"),
        (Some(run_id), None) => run_id.to_owned(),
        (None, _) => "Local run".to_owned(),
    };

    let commit_sha = input.commit_sha.trim();
    let commit_short = commit_sha.chars().take(7).collect::<String>();
    let commit_base = match kind {
        SourceKind::Ci => remote.clone(),
        SourceKind::Local => trimmed(input.cargo_repository.as_deref()).map(|url| {
            url.trim_end_matches('/')
                .trim_end_matches(".git")
                .to_owned()
        }),
    };
    let commit_url = commit_base
        .as_deref()
        .filter(|_| is_commit_sha(commit_sha))
        .and_then(|base| github_commit_url(base, commit_sha));

    let pull_request_number = pull_request_number(input.event_payload.as_deref())
        .or_else(|| git_ref.as_deref().and_then(pull_request_number_from_ref));
    let pull_request_url = match (pull_request_number, remote.as_deref()) {
        (Some(number), Some(remote)) => Some(format!("{remote}/pull/{number}")),
        _ => None,
    };

    let local_changes = kind == SourceKind::Local && input.repo_state.dirty;
    let revision_note = revision_note(
        kind,
        input.repo_state.merge_commit,
        pull_request_number,
        local_changes,
    );

    SourceContext {
        kind,
        repository,
        ci_label,
        ci_url,
        commit_sha: commit_sha.to_owned(),
        commit_short,
        commit_url,
        pull_request_number,
        pull_request_url,
        git_ref,
        merge_commit: input.repo_state.merge_commit,
        local_changes,
        revision_note,
    }
}

fn revision_note(
    kind: SourceKind,
    merge_commit: bool,
    pull_request_number: Option<u64>,
    local_changes: bool,
) -> String {
    match kind {
        SourceKind::Ci if merge_commit => match pull_request_number {
            Some(number) => format!("PR merge · refs/pull/{number}/merge"),
            None => "merge commit".to_owned(),
        },
        SourceKind::Ci => "CI checkout".to_owned(),
        SourceKind::Local if local_changes => {
            "Local run · commit link is HEAD, uncommitted changes rendered".to_owned()
        }
        SourceKind::Local => "Local run · clean tree, commit link matches HEAD".to_owned(),
    }
}

fn trimmed(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// Returns `owner/name` for a Cargo repository URL.
fn repository_slug(url: &str) -> Option<String> {
    let url = url.trim().trim_end_matches('/').trim_end_matches(".git");
    let path = match url.split_once("://") {
        Some((_, rest)) => rest.split_once('/').map(|(_, path)| path),
        None => url.split_once(':').map(|(_, path)| path),
    }?;
    let slug = path.trim_matches('/');
    (!slug.is_empty()).then(|| slug.to_owned())
}

fn is_commit_sha(value: &str) -> bool {
    value.len() >= 7 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// GitHub commit URL. Non-GitHub remotes stay unlinked rather than pointing at
/// a guessed URL.
fn github_commit_url(base: &str, sha: &str) -> Option<String> {
    let base = base.trim_end_matches('/');
    let host = base.split_once("://")?.1.split('/').next()?;
    host.contains("github")
        .then(|| format!("{base}/commit/{sha}"))
}

/// Reads only the pull request number from the GitHub event payload; the
/// payload itself is never copied into the report.
fn pull_request_number(event_payload: Option<&str>) -> Option<u64> {
    let payload: serde_json::Value = serde_json::from_str(event_payload?).ok()?;
    payload
        .get("pull_request")
        .and_then(|pull_request| pull_request.get("number"))
        .or_else(|| payload.get("number"))
        .and_then(serde_json::Value::as_u64)
}

fn pull_request_number_from_ref(git_ref: &str) -> Option<u64> {
    let rest = git_ref.strip_prefix("refs/pull/")?;
    rest.split_once('/')?.0.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{RepoState, SourceInput, SourceKind, derive};

    const PULL_REQUEST_PAYLOAD: &str =
        r#"{"action":"opened","pull_request":{"number":36,"title":"internal note"}}"#;

    fn ci_input() -> SourceInput {
        SourceInput {
            server_url: Some("https://github.com".into()),
            repository: Some("deps-rs/badge-maker-rs".into()),
            run_id: Some("38875326061".into()),
            run_attempt: Some("1".into()),
            git_ref: Some("refs/pull/36/merge".into()),
            event_payload: Some(PULL_REQUEST_PAYLOAD.into()),
            cargo_repository: Some("https://github.com/deps-rs/badge-maker-rs".into()),
            commit_sha: "74a8fdbd567337359ca1d7ca6a516ccecefa97d4".into(),
            repo_state: RepoState {
                dirty: false,
                merge_commit: true,
            },
        }
    }

    #[test]
    fn ci_pull_request_run_links_exact_attempt_commit_and_pull_request() {
        let context = derive(&ci_input());
        assert_eq!(context.kind, SourceKind::Ci);
        assert_eq!(
            context.ci_url.as_deref(),
            Some("https://github.com/deps-rs/badge-maker-rs/actions/runs/38875326061/attempts/1")
        );
        assert_eq!(
            context.commit_url.as_deref(),
            Some(
                "https://github.com/deps-rs/badge-maker-rs/commit/74a8fdbd567337359ca1d7ca6a516ccecefa97d4"
            )
        );
        assert_eq!(
            context.pull_request_url.as_deref(),
            Some("https://github.com/deps-rs/badge-maker-rs/pull/36")
        );
        assert_eq!(context.ci_label, "38875326061 / attempt 1");
        assert_eq!(context.revision_note, "PR merge · refs/pull/36/merge");
    }

    #[test]
    fn event_payload_contributes_only_the_pull_request_number() {
        let payload: serde_json::Value = serde_json::from_str(PULL_REQUEST_PAYLOAD).unwrap();
        assert_eq!(payload["pull_request"]["title"], "internal note");
        let serialized = serde_json::to_string(&derive(&ci_input())).unwrap();
        assert!(serialized.contains("pull_request_number"));
        assert!(!serialized.contains("internal note"));
    }

    #[test]
    fn pull_request_falls_back_to_the_merge_ref() {
        let mut input = ci_input();
        input.event_payload = Some(r#"{"action":"synchronize"}"#.into());
        let context = derive(&input);
        assert_eq!(context.pull_request_number, Some(36));
        assert_eq!(
            context.pull_request_url.as_deref(),
            Some("https://github.com/deps-rs/badge-maker-rs/pull/36")
        );
    }

    #[test]
    fn ci_run_without_attempt_omits_the_attempt_segment() {
        let mut input = ci_input();
        input.run_attempt = Some(String::new());
        input.event_payload = None;
        input.git_ref = Some("refs/heads/master".into());
        let context = derive(&input);
        assert_eq!(
            context.ci_url.as_deref(),
            Some("https://github.com/deps-rs/badge-maker-rs/actions/runs/38875326061")
        );
        assert!(context.pull_request_number.is_none());
        assert!(context.pull_request_url.is_none());
        assert_eq!(context.revision_note, "merge commit");
    }

    #[test]
    fn ci_run_without_merge_commit_reports_a_plain_checkout() {
        let mut input = ci_input();
        input.repo_state.merge_commit = false;
        let context = derive(&input);
        assert_eq!(context.revision_note, "CI checkout");
    }

    #[test]
    fn ci_run_without_environment_urls_reports_no_links() {
        let mut input = ci_input();
        input.server_url = None;
        input.repository = None;
        let context = derive(&input);
        assert_eq!(context.kind, SourceKind::Ci);
        assert!(context.ci_url.is_none());
        assert!(context.commit_url.is_none());
        assert_eq!(context.pull_request_number, Some(36));
        assert!(context.pull_request_url.is_none());
    }

    #[test]
    fn local_clean_run_uses_cargo_repository_for_the_commit_link() {
        let input = SourceInput {
            cargo_repository: Some("https://github.com/deps-rs/badge-maker-rs".into()),
            commit_sha: "74a8fdbd567337359ca1d7ca6a516ccecefa97d4".into(),
            ..SourceInput::default()
        };
        let context = derive(&input);
        assert_eq!(context.kind, SourceKind::Local);
        assert_eq!(context.ci_label, "Local run");
        assert!(context.ci_url.is_none());
        assert!(context.pull_request_number.is_none());
        assert_eq!(context.commit_short, "74a8fdb");
        assert_eq!(
            context.commit_url.as_deref(),
            Some(
                "https://github.com/deps-rs/badge-maker-rs/commit/74a8fdbd567337359ca1d7ca6a516ccecefa97d4"
            )
        );
        assert!(!context.local_changes);
        assert_eq!(
            context.revision_note,
            "Local run · clean tree, commit link matches HEAD"
        );
    }

    #[test]
    fn local_dirty_run_flags_uncommitted_changes() {
        let dirty = SourceInput {
            commit_sha: "74a8fdbd567337359ca1d7ca6a516ccecefa97d4".into(),
            repo_state: RepoState {
                dirty: true,
                merge_commit: false,
            },
            ..SourceInput::default()
        };
        let context = derive(&dirty);
        assert!(context.local_changes);
        assert_eq!(
            context.revision_note,
            "Local run · commit link is HEAD, uncommitted changes rendered"
        );

        let unknown = derive(&SourceInput::default());
        assert_eq!(unknown.commit_sha, "");
        assert!(unknown.commit_url.is_none());
        assert!(unknown.repository.is_none());
        assert!(unknown.git_ref.is_none());
    }

    #[test]
    fn non_github_remotes_stay_unlinked() {
        let input = SourceInput {
            cargo_repository: Some("https://gitlab.example.com/team/badge-maker-rs.git".into()),
            commit_sha: "74a8fdbd567337359ca1d7ca6a516ccecefa97d4".into(),
            ..SourceInput::default()
        };
        let context = derive(&input);
        assert_eq!(context.repository.as_deref(), Some("team/badge-maker-rs"));
        assert!(context.commit_url.is_none());
    }

    #[test]
    fn non_hex_revisions_are_never_linked() {
        let input = SourceInput {
            cargo_repository: Some("https://github.com/deps-rs/badge-maker-rs".into()),
            commit_sha: "unknown".into(),
            ..SourceInput::default()
        };
        let context = derive(&input);
        assert_eq!(context.commit_sha, "unknown");
        assert!(context.commit_url.is_none());
    }
}
