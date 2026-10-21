//! Git dependencies and `ail.lock` (mod.pkg.6-9, eff.cap.4), with a local
//! bare repository as the remote: no network. Skipped when `git` is missing.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "user.name=ail", "-c", "user.email=ail@example.com", "-c", "init.defaultBranch=main"])
        .args(["-c", "tag.gpgSign=false", "-c", "commit.gpgSign=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

struct Setup {
    dir: PathBuf,
    app: PathBuf,
    work: PathBuf,
    url: String,
}

impl Setup {
    fn new(name: &str) -> Option<Setup> {
        if Command::new("git").arg("--version").output().is_err() {
            eprintln!("skipped: no `git`");
            return None;
        }
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("git-deps-{name}"));
        if dir.exists() {
            make_writable(&dir);
            fs::remove_dir_all(&dir).unwrap();
        }
        let (work, app) = (dir.join("work"), dir.join("app"));
        fs::create_dir_all(&work).unwrap();
        fs::create_dir_all(&app).unwrap();
        git(&dir, &["init", "-q", "--bare", "remote.git"]);
        git(&work, &["init", "-q"]);
        let url = format!("file://{}", dir.join("remote.git").display());
        let s = Setup { dir, app, work, url };
        s.commit("w * h", "v1");
        fs::write(s.app.join("main.ail"), "use geo\n\nfn main()\n    print(geo.area(3, 4))\n    print(geo.cached())\n")
            .unwrap();
        s.manifest("tag v1 caps fs");
        Some(s)
    }

    /// Commit a `geo` package whose `area` is `body`, tagged `tag`, and
    /// push it; returns the commit.
    fn commit(&self, body: &str, tag: &str) -> String {
        fs::write(self.work.join("ail.pkg"), "package geo\n").unwrap();
        fs::write(
            self.work.join("geo.ail"),
            format!("use std.fs\n\npub fn area(w int, h int) int\n    {body}\n\npub fn cached() bool\n    fs.exists(\"no-such-file\")\n"),
        )
        .unwrap();
        git(&self.work, &["add", "-A"]);
        git(&self.work, &["commit", "-qm", body]);
        git(&self.work, &["tag", "-f", tag]);
        git(&self.work, &["push", "-q", "-f", &self.url, "HEAD:refs/heads/main", &format!("refs/tags/{tag}")]);
        git(&self.work, &["rev-parse", "HEAD"])
    }

    fn manifest(&self, rest: &str) {
        fs::write(self.app.join("ail.pkg"), format!("package app\ndep geo git \"{}\" {rest}\n", self.url)).unwrap();
    }

    fn ail(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ail"))
            .args(args)
            .current_dir(&self.dir)
            .env("AIL_HOME", self.dir.join("home"))
            .output()
            .unwrap()
    }

    fn lock(&self) -> String {
        fs::read_to_string(self.app.join("ail.lock")).unwrap_or_default()
    }
}

impl Drop for Setup {
    fn drop(&mut self) {
        make_writable(&self.dir);
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// The cache's files are read-only (mod.pkg.7); make them writable to
/// clean up on every platform.
fn make_writable(dir: &Path) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            make_writable(&p);
        } else if let Ok(meta) = fs::metadata(&p) {
            let mut perm = meta.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perm.set_readonly(false);
            let _ = fs::set_permissions(&p, perm);
        }
    }
}

fn text(o: &Output) -> (String, String) {
    (String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
}

#[test]
fn fetch_lock_and_update() {
    let Some(s) = Setup::new("lock") else { return };
    let first = git(&s.work, &["rev-parse", "HEAD"]);

    // mod.pkg.7-8: the first run fetches into the cache and writes the lock.
    let out = s.ail(&["run", "app"]);
    assert_eq!(text(&out).0, "12\nfalse\n", "{}", text(&out).1);
    assert!(s.lock().contains(&format!("dep geo git {} tag v1 commit {first}\n", s.url)), "{}", s.lock());
    let cached = s.dir.join("home/deps").join(format!("geo-{first}"));
    assert!(fs::metadata(cached.join("geo.ail")).unwrap().permissions().readonly());
    assert!(!cached.join(".git").exists());

    // A moved tag changes nothing while the lock pins the commit.
    let second = s.commit("w * h + 1", "v1");
    assert_eq!(text(&s.ail(&["run", "app"])).0, "12\nfalse\n");

    // mod.pkg.9: `ail pkg update geo` locks what the tag names now.
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(["pkg", "update", "geo"])
        .current_dir(&s.app)
        .env("AIL_HOME", s.dir.join("home"))
        .output()
        .unwrap();
    assert_eq!(text(&out).0, format!("locked geo {second}\n"), "{}", text(&out).1);
    assert_eq!(text(&s.ail(&["run", "app"])).0, "13\nfalse\n");
    assert!(s.lock().contains(&second));

    // `--deterministic` and `--deny net` use the lock and the cache.
    assert_eq!(text(&s.ail(&["run", "--deterministic", "app"])).0, "13\nfalse\n");

    // mod.pkg.8: a manifest that no longer matches the lock is stale.
    s.manifest(&format!("rev {first} caps fs"));
    let out = s.ail(&["run", "--deterministic", "app"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out).1.contains("error[mod.pkg.8]") && text(&out).1.contains("ail pkg update geo"),
        "{}",
        text(&out).1
    );
    assert!(s.lock().contains(&second), "a refused run leaves the lock alone");
    // Without `--deterministic` the run relocks.
    assert_eq!(text(&s.ail(&["run", "app"])).0, "12\nfalse\n");
    assert!(s.lock().contains(&format!("rev {first} commit {first}")), "{}", s.lock());

    // eff.cap.4: grants apply to git dependencies as to path ones.
    s.manifest(&format!("rev {first}"));
    let out = s.ail(&["check", "app"]);
    assert!(text(&out).1.contains("error[eff.cap.4]"), "{}", text(&out).1);
}

#[test]
fn fetch_errors_name_the_dependency() {
    let Some(s) = Setup::new("errors") else { return };

    // mod.pkg.7: `--deny net` with an empty cache fetches nothing.
    let out = s.ail(&["check", "--deny", "net", "app"]);
    assert!(text(&out).1.contains("error[mod.pkg.7]: dependency `geo` isn't in the cache"), "{}", text(&out).1);
    assert!(!s.app.join("ail.lock").exists());

    // An unknown tag.
    s.manifest("tag v9 caps fs");
    let out = s.ail(&["run", "app"]);
    assert!(text(&out).1.contains("dependency `geo`") && text(&out).1.contains("has no tag `v9`"), "{}", text(&out).1);
    assert!(!s.app.join("ail.lock").exists(), "a failed fetch writes no lock");

    // An unreachable repository.
    fs::write(s.app.join("ail.pkg"), "package app\ndep geo git \"file:///no/such/repo.git\" tag v1\n").unwrap();
    let out = s.ail(&["run", "app"]);
    assert!(text(&out).1.contains("error[mod.pkg.7]: can't fetch dependency `geo`"), "{}", text(&out).1);

    // No `git` on the path.
    s.manifest("tag v1 caps fs");
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(["run", "app"])
        .current_dir(&s.dir)
        .env("AIL_HOME", s.dir.join("home"))
        .env("PATH", s.dir.join("empty"))
        .output()
        .unwrap();
    assert!(text(&out).1.contains("error[mod.pkg.7]: can't run `git` to fetch dependency `geo`"), "{}", text(&out).1);

    // mod.pkg.9: updating a name that isn't a git dependency.
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(["pkg", "update", "nope"])
        .current_dir(&s.app)
        .env("AIL_HOME", s.dir.join("home"))
        .output()
        .unwrap();
    assert!(text(&out).1.contains("error[mod.pkg.9]"), "{}", text(&out).1);
}
