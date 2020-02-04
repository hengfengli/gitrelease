//! # gitrelease
//!
//! `gitrelease` is a small tool to make git release more convenient.
//! It generates a brief summary for a release.

use git2::{
    ObjectType,
    Commit,
    Repository,
    Time
};
use std::str;
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use docopt::Docopt;
use serde::Deserialize;

static NAME: &str = "gitrelease";
static VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {
    static ref VERSION_REGEX: Regex = Regex::new(r"^(\d+)\.(\d+)\.(\d+)(-\w+)?(-SNAPSHOT)?$").unwrap();
    static ref GITHUB_URL_REGEX: Regex = Regex::new(r"^git@([\w.]*):([\w/-]*)\.git$").unwrap();
}

/// Finds out the most recent commit in the repository.
fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

/// Finds out all commits in the range [start, end). Left side is inclusive
/// and right side is exclusive.
fn find_commits_in_range(repo: &Repository, start: git2::Oid, end: git2::Oid) -> Vec<Commit> {
    let mut commits: Vec<Commit> = Vec::new();
    // list recent commits
    let mut revwalk = repo.revwalk().expect("Failed to get revwalk");

    // left commit is hidden and right commit is shown.
    let range = format!("{}..{}", end, start);
    revwalk.push_range(&range[..]).expect("Failed to push range");

    for oid in revwalk {
        let oid = oid.expect("Failed to get oid");
        let commit = repo.find_commit(oid).expect("Failed to find the commit");
        commits.push(commit);
    }

    commits
}

/// Returns a string for printing all commits since last release.
fn get_commits(commits: &Vec<Commit>, submodule: &str, repo_url: &str) -> String {
    let mut result = String::from("### Commits since last release:\n\n");

    for commit in commits {
        let commit_messages = String::from_utf8_lossy(commit.message_bytes());
        let commit_title = commit_messages.lines().next().unwrap();

        let commit_url = format!("{}/commit/{}", repo_url, commit.id());

        if commit_title.starts_with("Release") {
            continue;
        }

        if submodule == "" || commit_title.contains(&format!("({})", submodule)) {
            result.push_str(&format!("* [{}]({})\n", commit_title, commit_url));
        }
    }

    result.push_str("\n\n");
    result
}

/// A git tag.
#[derive(Clone)]
struct Tag {
    name: String,
    time: Time,
    oid: git2::Oid,
}

/// Finds out the tag of the last release.
fn find_commit_for_last_release(repo: &Repository, folder: &str) -> Option<Tag> {
    let mut latest_tag: Option<Tag> = None;

    let pattern = match folder {
        "" => String::from("v*"),
        _ => format!("{}/*", folder)
    };
    let names = repo.tag_names(Some(&pattern)).expect("Couldn't find any matching tags.");
    for name in &names {
        let name = name.expect("Couldn't parse tag name.");

        // Result
        let obj = repo.revparse_single(name).expect("Failed to get obj");

        if let Some(commit) = obj.as_commit() {
            let commit_time = commit.author().when();

            // Here, the ownership of Tag in `latest_tag` has been moved to `tag`
            if let Some(tag) = latest_tag {
                if commit_time.seconds() > tag.time.seconds() {
                    latest_tag = Some(Tag{
                        name: name.to_string(),
                        time: commit_time,
                        oid: commit.id(),
                    });
                }
                else {
                    // move the ownership back
                    latest_tag = Some(tag);
                }
            } else {
                latest_tag = Some(Tag{
                    name: name.to_string(),
                    time: commit_time,
                    oid: commit.id(),
                });
            }
        }
    }

    latest_tag
}

/// Returns a hash table of categorized commits based on the change type, e.g.,
/// feat, fix, docs, etc.
fn get_category_table(commits: &Vec<Commit>, submodule: &str) -> HashMap<String, Vec<String>> {
    let mut table: HashMap<String, Vec<String>> = HashMap::new();

    for commit in commits {
        let commit_messages = String::from_utf8_lossy(commit.message_bytes());
        let commit_title = commit_messages.lines().next().expect("Couldn't read the commit's title.");
        if commit_title.starts_with("Release") {
            continue;
        }

        if submodule == "" || commit_title.contains(&format!("({})", submodule)) {
            if let Some(index) = commit_title.find(':') {
                let title = &commit_title[index+1..].trim();

                let end_index = match commit_title.find("(") {
                    Some(i) => i,
                    None => index
                };
                let doc_type = &commit_title[..end_index];

                if let Some(text_list) = table.get_mut(doc_type) {
                    text_list.push(title.to_string());
                } else {
                    table.insert(doc_type.to_string(), vec![title.to_string()]);
                }
            }
        }
    }

    table
}

/// Returns a string of header info.
fn get_header(commits: &Vec<Commit>, last_tag: &Tag, submodule: &str) -> String {
    let mut result = String::from("");

    let version: &str = last_tag.name.split('/').last().expect("Couldn't find the version.");
    let version = match version.chars().next().expect("Couldn't read the next char from version.") {
        'v' => &version[1..],
        _ => version
    };


    let mut version = Version::parse(version).expect("Couldn't parse the version string.");
    let date = chrono::Local::now();
    let table = get_category_table(commits, submodule);
    let bump_type = match table.get("feat") {
        Some(_) => "minor",
        None => "patch"
    };
    version.bump(bump_type);

    result.push_str(&format!(":robot: I have created a release \\*beep\\* \\*boop\\*
---
### {} / {}\n\n", version.to_string(), date.format("%Y-%m-%d")));

    result
}

/// A semantic version.
#[derive(Debug)]
struct Version {
    major: i32,
    minor: i32,
    patch: i32,
    extra: String,
    snapshot: bool
}

impl Version {
    /// Parses a string into a version.
    fn parse(version: &str) -> Option<Version> {
        for cap in VERSION_REGEX.captures_iter(version) {
            // println!("{:?}", cap);
            return Some(Version{
                major: cap[1].parse::<i32>().expect("Can't parse major number."),
                minor: cap[2].parse::<i32>().expect("Can't parse minor number."),
                patch: cap[3].parse::<i32>().expect("Can't parse patch number."),
                extra: String::new(),
                snapshot: false
            });
        }
        None
    }

    /// Bumps a version.
    fn bump(&mut self, bump_type: &str) {
        match bump_type {
            "major" => {
                self.major += 1;
                self.minor = 0;
                self.patch = 0;
                self.snapshot = false;
            },
            "minor" => {
                self.minor += 1;
                self.patch = 0;
                self.snapshot = false;
            },
            "patch" => {
                self.patch += 1;
                self.snapshot = false;
            },
            "snapshot" => {
                self.patch += 1;
                self.snapshot = true;
            }
            _ => ()
        }
    }

    /// Returns a display string.
    fn to_string(&self) -> String {
        let snapshot = if self.snapshot {
            "-SNAPSHOT"
        } else {
            ""
        };
        format!("{}.{}.{}{}{}", self.major, self.minor, self.patch, self.extra, snapshot)
    }
}

/// Returns a string of categorized changes.
fn get_categorized_changes(commits: &Vec<Commit>, submodule: &str) -> String {
    let mut result = String::from("");
    let table = get_category_table(commits, submodule);

    // `HashMap::iter()` returns an iterator that yields
    // (&'a key, &'a value) pairs in arbitrary order.
    for (key, values) in table.iter() {

        let (category, is_skip) = match key.as_str() {
            "feat" => ("Features", false),
            "fix" => ("Bug Fixes", false),
            "docs" => ("Documentation", false),
            "style" => ("Styles", true),
            "refactor" => ("Code Refactoring", true),
            "test" => ("Test Refactoring", true),
            "chore" => ("Miscellaneous Chores", true),
            "perf" => ("Performance Improvements", false),
            _ => ("Other", true)
        };

        if is_skip {
            continue;
        }

        result.push_str(&format!("#### {}\n\n", category));
        for text in values {
            result.push_str(&format!("* {}\n", text));
        }
        result.push_str("\n");
    }

    result.push_str("---\n");
    result
}

/// Returns a string of files edited since last release.
fn get_edited_files(repo: &Repository, old: &Commit, new: &Commit, folder: &str) -> String {
    let mut result = String::from("");

    let old_tree = &old.tree().expect("Couldn't find the old tree.");
    let new_tree = &new.tree().expect("Couldn't find the new tree.");
    let diff = repo.diff_tree_to_tree(Some(old_tree), Some(new_tree), None).expect("Couldn't diff two trees.");

    let deltas = diff.deltas();

    result.push_str("### Files edited since last release:\n\n<pre><code>");

    for delta in deltas {
        let filename = delta.old_file().path().expect("Couldn't find the path of old file.");
        let filename = filename.to_str().expect("Couldn't parse file path.");
        let pattern = match folder {
            "" => String::from(""),
            _ => format!("{}/", folder)
        };
        if filename.starts_with(&pattern) {
            result.push_str(&format!("{}\n", filename));
        }
    }

    result.push_str("</code></pre>\n");
    result
}

/// Returns a string of a link to compare changes.
fn get_compare_changes(repo_url: &str, oid: git2::Oid) -> String {
    format!("[Compare Changes]({}/compare/{}...HEAD)", repo_url, oid)
}

/// Returns a string of the footer.
fn get_footer() -> String {
    format!("\n\n\nThis PR was generated with [GitRelease](https://github.com/hengfengli/gitrelease).\n")
}

/// Finds out the url of the `origin` remote.
fn find_origin_remote_url(repo: &Repository) -> String {
    let origin_remote = repo.find_remote("origin").expect("Couldn't find the `origin` remote.");
    let origin_url = origin_remote.url().expect("Failed to read the remote's url.");

    if origin_url.starts_with("https://") {
        return origin_url.to_string();
    }
    let cap = GITHUB_URL_REGEX.captures_iter(origin_url).next().expect("Failed to read origin url.");
	format!("https://{}/{}", &cap[1], &cap[2])
}

const USAGE: &'static str = "
Generate a summary of git release.

Usage:
  gitrelease
  gitrelease [--dir=<path>] [--subdir=<path>] [--submodule=<name>]
  gitrelease (-h | --help)
  gitrelease (-v | --version)

Options:
  -h --help     Show this screen.
  -v --version  Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_dir: String,
    flag_subdir: String,
    flag_submodule: String,
    flag_version: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{} {}", NAME, VERSION);
        return;
    }

    // unwrap_or: unwrap it, if not exist, use "." current directory.
    // std::string::String
    //
    // TODO(hengfeng): when flag_dir is not given, use the current
    // directory.
    let repo_root: String = match args.flag_dir.as_str() {
        "" => String::from("."),
        _ => args.flag_dir,
    };

    let subdir = args.flag_subdir;
    let submodule = args.flag_submodule;

    let repo = Repository::open(repo_root.as_str()).expect("Couldn't open repository");
    let repo_url = &find_origin_remote_url(&repo);

    if let Some(last_release_tag) = find_commit_for_last_release(&repo, &subdir) {
        let last_commit = find_last_commit(&repo).expect("Failed to find the last commit");
        let commits = find_commits_in_range(&repo, last_commit.id(), last_release_tag.oid);
        let last_release_tag_commit = repo.find_commit(last_release_tag.oid).expect("Failed to find the commit for the last tag.");

        print!("{}", get_header(&commits, &last_release_tag, &submodule));
        print!("{}", get_categorized_changes(&commits, &submodule));
        print!("{}", get_commits(&commits, &submodule, repo_url));
        print!("{}", get_edited_files(&repo, &last_release_tag_commit, &last_commit, &subdir));
        print!("{}", get_compare_changes(repo_url, last_release_tag.oid));
        print!("{}", get_footer());
    }
}
