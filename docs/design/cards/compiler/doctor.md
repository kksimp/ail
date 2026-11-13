# compiler: doctor

## doctor
```
struct doctor.Check  ## One check: its name, `ok`, `warn` or `fail`, what was found, the fix to run (empty when ok), and whether a
    name str
    status str
    detail str
    fix str
    required bool
fn doctor.good(name str, detail str) doctor.Check
fn doctor.bad(name str, status str, detail str, fix str, required bool) doctor.Check
fn doctor.norm(p str) str  ## `p` with `/` for each `\`, and no trailing `/`.
fn doctor.first_line(p str) str [fs io]
fn doctor.windows() bool [env]
fn doctor.os_of(target str) str  ## The OS word of a target (`aarch64-macos` is `macos`).
fn doctor.writable(dir str) bool [fs io]  ## A directory `ail` can write to: made if it is missing, probed with a file.
fn doctor.path_line(root str) str [env]  ## The line for a shell profile (tool.path.1): it adds the toolchain's directory, where `ail` is, to PATH.
fn doctor.same_toolchain(found str, root str) bool [env io proc]  ## Is `found`, the first `ail` on PATH, this toolchain's? Its directory is `root` (and so is a symlink's
fn doctor.check_root(root str) doctor.Check [fs io]
fn doctor.check_version(root str) doctor.Check [fs io]
fn doctor.check_language() doctor.Check [fs io]  ## The language version of the project in the current directory (mod.pkg.10).
fn doctor.check_path(root str) doctor.Check [env io proc]
fn doctor.check_git() doctor.Check [io proc]
fn doctor.check_curl() doctor.Check [env io proc]
fn doctor.check_hash() doctor.Check [env io proc]
fn doctor.check_cache(cache str) doctor.Check [fs io]
fn doctor.check_app_host(root str, target str) doctor.Check [env fs io]  ## std.app.10: the host `AIL_APP_HOST` names, else the one built under the toolchain.
fn doctor.check_audio_host(root str, target str) doctor.Check [env fs io]  ## std.audio.9: `AIL_AUDIO_HOST`, else the one built under `tools/audio-host/OS/build`.
fn doctor.checks(root str, cache str, target str) List[doctor.Check] [env fs io proc]
fn doctor.q(s str) str
fn doctor.check_json(k doctor.Check) str
fn doctor.check_text(k doctor.Check) str
pub fn doctor.command(args List[str], root_arg str, cache str, target str) int [env fs io proc]  ## The exit status of `ail doctor ARGS`: 1 when a required check failed (tool.doctor.1).
```
