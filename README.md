GitRelease for rust is a starter project, which provides a small cli
tool to generate a release summary. It learns from the current commit
history and creates a nice summary of changes since last release

Please note that this is a toy project just for learning rust.

[![Crates.io](https://img.shields.io/crates/v/gitrelease)](https://crates.io/crates/gitrelease)
[![Crates.io](https://img.shields.io/crates/l/gitrelease)](LICENSE)
[![Build status](https://api.travis-ci.org/hengfengli/gitrelease)](https://travis-ci.org/hengfengli/gitrelease)

### Quick example

1. Here is a full working example.

```bash
$ git clone https://github.com/hyperium/tonic
$ cd tonic
$ gitrelease
```

Output:

```markdown
:robot: I have created a release \*beep\* \*boop\*
---
### 0.1.2 / 2020-02-09

---
### Commits since last release:

* [Fix 0.1.1 diff link in changelog (#247)](https://github.com/hyperium/tonic/commit/3e63e05666fcb5a099b96236f4d99ffda25f7d57)


### Files edited since last release:

<pre><code>CHANGELOG.md
</code></pre>
[Compare Changes](https://github.com/hyperium/tonic/compare/66ac4c4049f7135a4f6b6d58600a7f1716e1364f...HEAD)


This PR was generated with [GitRelease](https://github.com/hengfengli/gitrelease).
```

2. There is another working example.

```bash
$ git clone https://github.com/googleapis/google-cloud-ruby.git

# --dir: the absolute path to your git repo directory.
# --subdir: the sub-directory where related changes happen.
# --submodule: the submodule that you want to release, e.g., a commit title "fix(<submodule_name>): fix a bug.".
$ gitrelease --dir=/<abs_path>/google-cloud-ruby/ --subdir=google-cloud-secret_manager --submodule=secret_manager
```

Output:

```markdown
:robot: I have created a release \*beep\* \*boop\*
---
### 0.2.1 / 2020-02-09

#### Documentation

* change a few readme references from language to secret-manager
---
### Commits since last release:

* [docs(secret_manager): change a few readme references from language to secret-manager](https://github.com/googleapis/google-cloud-ruby/commit/ede794ccf7cfa2db1a3eb842fcd43bda276e26c2)

### Files edited since last release:

<pre><code>google-cloud-secret_manager/README.md
</code></pre>
[Compare Changes](https://github.com/googleapis/google-cloud-ruby/compare/fe8f239bd97c2bdadb4da5a3012cc4cd738a7efa...HEAD)


This PR was generated with [GitRelease](https://github.com/hengfengli/gitrelease).
```

### Credits

This project is inspired by [Release Please](https://github.com/googleapis/release-please), which is more sophisticated and has more features than this toy project. You probably should consider to use it in production.
