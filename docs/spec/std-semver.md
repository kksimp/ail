# Standard library: `std.semver`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1219).

Versions in the Semantic Versioning 2.0.0 format (`1.4.2-rc.1+build.5`) and
npm-style ranges (`^1.2.0 || >=3.0.0`). Every function is pure
(`eff.infer.1`) and needs no capability. The module is written in AIL and is
in scope without `use` (`std.use.4`); `semver.parse(...)` names it.

## The module

**std.semver.1** — `std.semver` provides `parse`, `is_valid`, `compare`,
`bump`, `parse_range`, `satisfies`, `max_satisfying` and `resolve`, and the
types `Version`, `VersionRange`, `Part`, `Dep`, `Package`, `Pin`, `SemverError`
and `SemverKind`. A `Version`
has the `int` fields `major`, `minor` and `patch` and the `List[str]` fields
`pre` and `build`, the dot-separated identifiers after `-` and after `+`.
`Version.is_prerelease()` is true when `pre` is not empty. *(ADR 1219)*

```ail tool=run,FILE expect=1,4,2,true rules=std.semver.1
fn main() !
    v := semver.parse("1.4.2-rc.1")?
    print("{v.major},{v.minor},{v.patch},{v.is_prerelease()}")
```

**std.semver.2** — `parse(text str) Version ! SemverError` reads SemVer 2.0.0
exactly: `MAJOR.MINOR.PATCH`, then optionally `-PRERELEASE`, then optionally
`+BUILD`. Each number is digits only with no leading zero and fits an `int`.
Prerelease and build identifiers are `.`-separated, non-empty, and made of
`0-9`, `A-Z`, `a-z` and `-`; a numeric prerelease identifier has no leading
zero (a build identifier may). A prefix (`v1.2.3`, `=1.2.3`), missing parts
(`1.2`), extra parts and spaces are errors: ranges (`std.semver.6`) accept
them, versions do not. `is_valid(text) bool` says whether `parse` succeeds.
*(ADR 1219)*

```ail tool=run,FILE expect=1.4.2-rc.1+build.5 rules=std.semver.2
fn main() !
    v := semver.parse("1.4.2-rc.1+build.5")?
    print("{v} {v.pre.join(",")} {v.build.join(",")}")
```

**std.semver.3** — `compare(a, b) int` is -1, 0 or 1 as `a` has lower, equal
or higher precedence than `b` (SemVer 2.0.0 item 11): by major, minor and
patch as numbers; a version with a prerelease is lower than the same numbers
without; prerelease identifiers compare one by one, a numeric identifier by
value (with no `int` limit), a numeric one below an alphanumeric one, two
alphanumeric ones by ASCII order, and a list that is a prefix of another is
lower. The build is ignored. `Version` implements `Ord` through `compare`, so
`a < b`, `a.cmp(b)`, `vs.sorted()`, `vs.max()` and `vs.min()` follow it. `==`
is the automatic equality of every field, the build included, so
`1.0.0+a == 1.0.0+b` is false while `compare` of the two is 0. *(ADR 1219)*

```ail tool=run,FILE expect=-1,0 rules=std.semver.3
fn main() !
    rc := semver.parse("1.0.0-rc.1")?
    release := semver.parse("1.0.0")?
    print("{semver.compare(rc, release)},{semver.compare(release, release)}")
```

**std.semver.4** — A `Version` shows as its text form, `1.4.2-rc.1+build.5`:
`print(v)`, `"{v}"` and `v.to_str()` give it, and `parse(v.to_str())` gives
back an equal `Version`. *(ADR 1219)*

```ail tool=run,FILE expect=2.0.0-beta rules=std.semver.4
fn main() !
    v := semver.parse("2.0.0-beta")?
    print("{v} {v.is_prerelease()}")
```

**std.semver.5** — `bump(v Version, part Part) Version` raises one number by
one: `Part.Major` sets minor and patch to 0, `Part.Minor` sets patch to 0,
`Part.Patch` changes only itself. The result has no prerelease and no build.
`Part` is an enum, not text. *(ADR 1219)*

```ail tool=run,FILE expect=1.5.0 rules=std.semver.5
fn main() !
    v := semver.parse("1.4.2-rc.1")?
    print(semver.bump(v, semver.Part.Minor))
```

## Ranges

**std.semver.6** — `parse_range(text str) VersionRange ! SemverError` reads
a range. `||` separates alternatives; a version is in the range when it is in
one of them. An alternative is comparators separated by spaces, all of which
must hold, or a hyphen range (`std.semver.9`). A comparator is an operator
(`=`, `>`, `>=`, `<`, `<=`, `~`, `~>`, `^`, or none, which means `=`), written
before its version with or without a space, and a version of one to three
parts (`1`, `1.2`, `1.2.3`, with an optional `v` prefix, prerelease and
build). An empty alternative, and `*`, hold for every version. The text it was
read from is the field `text`. Anything else is a `SemverError` of kind
`BadRange`, `BadNumber`, `BadPre` or `BadBuild`. *(ADR 1219)*

```ail tool=run,FILE expect=true,true,false rules=std.semver.6
fn main() !
    r := semver.parse_range("1.2.3 || >= 4.0.0 <5")?
    a := r.matches(semver.parse("1.2.3")?)
    b := r.matches(semver.parse("4.5.0")?)
    c := r.matches(semver.parse("5.0.0")?)
    print("{a},{b},{c}")
```

**std.semver.7** — A part written `x`, `X`, `*` or left out is a wildcard, and
a comparator with a wildcard stands for the versions it spans: `1.2` and
`1.2.x` are `>=1.2.0 <1.3.0`, `1` and `1.x` are `>=1.0.0 <2.0.0`, `=1.2.3` is
exactly `1.2.3` (the build is ignored). With a partial version `>` means above
everything the version spans (`>1.2` is `>=1.3.0`), `>=` and `<` fill the
missing parts with 0 (`<1.2` is `<1.2.0`), and `<=` includes everything the
version spans (`<=1.2` is `<1.3.0`). `>*` and `<*` hold for no version.
*(ADR 1219)*

**std.semver.8** — `~V` allows patch-level changes: `~1.2.3` and `~1.2` are
`>=V <1.3.0`, `~1` is `>=1.0.0 <2.0.0`. `^V` allows changes that keep the
left-most non-zero part: `^1.2.3` is `>=1.2.3 <2.0.0`, `^0.2.3` is
`>=0.2.3 <0.3.0`, `^0.0.3` is `>=0.0.3 <0.0.4`, `^1.2` is `>=1.2.0 <2.0.0`,
`^0.0` is `>=0.0.0 <0.1.0` and `^0` is `>=0.0.0 <1.0.0`. *(ADR 1219)*

```ail tool=run,FILE expect=true,false,true,false rules=std.semver.8
fn main() !
    a := semver.satisfies(semver.parse("1.9.0")?, "^1.2.3")?
    b := semver.satisfies(semver.parse("2.0.0")?, "^1.2.3")?
    c := semver.satisfies(semver.parse("0.2.9")?, "^0.2.3")?
    d := semver.satisfies(semver.parse("0.3.0")?, "^0.2.3")?
    print("{a},{b},{c},{d}")
```

**std.semver.9** — `A - B` is a hyphen range: `>=A` and `<=B`, with the
wildcard rules of `std.semver.7` for a partial version: `1.2.3 - 2.3.4` is
`>=1.2.3 <=2.3.4`, `1.2.3 - 2` is `>=1.2.3 <3.0.0`. The hyphen is a word of
its own. *(ADR 1219)*

```ail tool=run,FILE expect=true,false rules=std.semver.7,std.semver.9
fn main() !
    a := semver.satisfies(semver.parse("2.9.9")?, "1.2.3 - 2")?
    b := semver.satisfies(semver.parse("3.0.0")?, "1.2.3 - 2")?
    print("{a},{b}")
```

**std.semver.10** — `r.matches(v) bool` is true when `v` is in the range.
A version with a prerelease matches an alternative only when one of that
alternative's comparators has a prerelease of the same `major.minor.patch`:
`1.3.0-beta` is not in `^1.2.3`, and `1.2.3-beta` is in `>=1.2.3-alpha`, but
`1.2.4-beta` is not. `*` matches no prerelease. The build is ignored.
*(ADR 1219)*

```ail tool=run,FILE expect=false,true,false rules=std.semver.10
fn main() !
    r := semver.parse_range(">=1.2.3-alpha")?
    a := r.matches(semver.parse("1.2.2")?)
    b := r.matches(semver.parse("1.2.3-beta")?)
    c := r.matches(semver.parse("1.2.4-beta")?)
    print("{a},{b},{c}")
```

**std.semver.11** — `satisfies(v Version, text str) bool ! SemverError` is
`parse_range(text)?.matches(v)`. `max_satisfying(vs List[Version], r
VersionRange) Version?` is the highest version of `vs` (by `compare`) that is
in `r`, and `none` when none is. *(ADR 1219)*

```ail tool=run,FILE expect=1.9.3 rules=std.semver.11
fn main() !
    r := semver.parse_range("^1.2.0")?
    mut vs := List[semver.Version]()
    for t in ["1.1.0", "1.9.3", "2.0.0", "1.10.0-rc.1"]
        vs.push(semver.parse(t)?)
    zero := semver.parse("0.0.0")?
    print(semver.max_satisfying(vs, r) else zero)
```

## Resolving dependencies

**std.semver.13** — `resolve(roots List[Dep], packages List[Package])
List[Pin] ! SemverError` picks one version of each package so that every
requirement holds. Details: [below](#more-on-stdsemver13). *(ADR 1266)*

```ail tool=run,FILE expect=lib=1.2.0,util=2.1.0 rules=std.semver.13
fn pkg(name str, v str, deps List[semver.Dep]) semver.Package !
    semver.Package(name: name, version: semver.parse(v)?, deps: deps)

fn main() !
    roots := [semver.Dep(name: "lib", range: "^1.0.0")]
    packages := [
        pkg("lib", "1.0.0", [])?,
        pkg("lib", "1.2.0", [semver.Dep(name: "util", range: ">=2.0.0")])?,
        pkg("lib", "2.0.0", [])?,
        pkg("util", "2.1.0", [])?,
        pkg("util", "3.0.0", [semver.Dep(name: "lib", range: "^2.0.0")])?,
    ]
    pins := semver.resolve(roots, packages)?
    print(pins.map(|p| "{p.name}={p.version}").join(","))
```

**std.semver.14** — `resolve` fails with a `SemverError`: the one `parse_range`
gives when a `range` is not a range, kind `NoPackage` when a required package has no entry in `packages`, and kind
`NoSolution` when the requirements cannot all hold. Details: [below](#more-on-stdsemver14). *(ADR 1266)*

```ail tool=run,FILE expect=NoSolution rules=std.semver.14
fn main() !
    v := semver.parse("1.0.0")?
    packages := [
        semver.Package(name: "a", version: v, deps: [semver.Dep(name: "b", range: "^2.0.0")]),
        semver.Package(name: "b", version: v, deps: []),
    ]
    got := semver.resolve([semver.Dep(name: "a", range: "*")], packages) else err
        print(err.kind.name())
        return
    print(got.len())
```

## Errors

**std.semver.12** — `SemverError` is an error with `kind SemverKind` and
`why str`; its text is `why`. The kinds are `NoText` (empty text), `BadShape`
(not three dot-separated parts, or a prefix), `BadNumber` (a number that is
empty, not digits, has a leading zero or is too large), `BadPre` (a prerelease
identifier), `BadBuild` (a build identifier) `BadRange` (a range that has
no version, or more than three parts), and `NoPackage` and `NoSolution`
(`std.semver.14`). Match on `err.kind`, never on `why`.
*(ADR 1219)*

```ail tool=run,FILE expect=BadNumber rules=std.semver.12
fn main()
    v := semver.parse("01.2.3") else err
        print(err.kind.name())
        return
    print(v)
```

### `std.semver` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.semver.13

A `Dep` is a package `name` and a `range` as text; a `Package` is a `name`, a `version` and the `deps` that version requires; a `Pin` is a `name` and the chosen `version`. `roots` are the program's own requirements, `packages` every version that may be chosen. The result holds a pin for each package reached from `roots`, in name order, and none for a package nothing reaches. Each package gets the highest version that fits every range on it so far; when no version fits, the search undoes the latest choice that changed the ranges and tries the next lower version, so the answer is the same on every run. A package is never picked at two versions. Prerelease versions follow `std.semver.10`.

### `std.semver` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.semver.14

The text names the package and the ranges, or the two requirements, of the last failed attempt.
