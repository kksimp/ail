#!/usr/bin/env python3
"""Writes tests/cases/std_toml_suite.ail and std_yaml_suite.ail from the official suites.

    gen_suite_tests.py TOML_TEST_CHECKOUT YAML_TEST_SUITE_DATA_CHECKOUT

TOML_TEST_CHECKOUT is a clone of github.com/toml-lang/toml-test (the files listed in
tests/files-toml-1.0.0); YAML_TEST_SUITE_DATA_CHECKOUT is the `data` branch of
github.com/yaml/yaml-test-suite. Both are MIT licensed (see NOTICE). The cases are
written out verbatim, so the expected results come from the suites, never from std.

YAML cases are sorted into three lists by what std.yaml (a subset parser, std.yaml.7)
is required to do with them: STRICT must read the exact JSON of the suite, LENIENT
(valid YAML outside the subset) must either refuse or read that exact JSON, and
INVALID must be refused. yaml_strict.txt and yaml_invalid.txt name the case ids of the
first and third list; every other valid case is LENIENT, and an invalid case that
is in neither list is a known gap (listed in the header of std_yaml_suite.ail). Rerun
after changing a list or after the suites change, then run `ail fmt` on both files.
"""
import json
import os
import sys
import unicodedata

CHUNK = 40


def esc(s):
    out = []
    for c in s:
        o = ord(c)
        if c == '\\':
            out.append('\\\\')
        elif c == '"':
            out.append('\\"')
        elif c == '{':
            out.append('\\{')
        elif c == '\n':
            out.append('\\n')
        elif c == '\r':
            out.append('\\r')
        elif c == '\t':
            out.append('\\t')
        elif o < 32 or o == 127 or (unicodedata.category(c)[0] in 'CZ' and c != ' ') or 0xd800 <= o <= 0xdfff:
            out.append('\\u{%x}' % o)
        else:
            out.append(c)
    return ''.join(out)


def chunks(name, cases):
    """AIL functions `name_K` of CHUNK cases each, and the list of their names."""
    fns, names = [], []
    for k in range(0, len(cases), CHUNK):
        fn = '%s_%d' % (name, k // CHUNK)
        names.append(fn)
        fns.append('fn %s() List[Case]\n    [\n%s    ]\n' % (fn, ''.join(
            '        Case(name: "%s", src: "%s", want: "%s"),\n' % (esc(n), esc(s), esc(w)) for n, s, w in cases[k:k + CHUNK])))
    return fns, names


def toml_cases(root):
    valid, invalid = [], []
    for line in open(root + '/tests/files-toml-1.0.0', encoding='utf-8'):
        f = line.strip()
        if not f.endswith('.toml'):
            continue
        raw = open(root + '/tests/' + f, 'rb').read()
        try:
            src = raw.decode('utf-8')
        except UnicodeDecodeError:
            continue  # not representable in a source string
        name = f[:-5]
        if f.startswith('valid/'):
            want = json.dumps(json.load(open(root + '/tests/' + name + '.json', encoding='utf-8')), ensure_ascii=False, separators=(',', ':'))
            valid.append((name, src, want))
        else:
            invalid.append((name, src, ''))
    return valid, invalid


TOML_HEAD = '''## std.toml against toml-test (TOML 1.0.0): every valid document reads as the suite's JSON,
## every invalid one is refused (std.toml.1 to std.toml.14). The cases are the suite's files
## verbatim; tools/conformance/gen_suite_tests.py writes this file. toml-test is MIT licensed,
## Copyright (c) 2018 TOML authors (NOTICE).
use std.toml

## A case: its name, the TOML text and, for a valid one, the suite's typed JSON ("" if invalid).
struct Case
    name str
    src str
    want str

## Whether `got` is the value the typed JSON `want` describes: a leaf is {"type": T, "value": V}
## with V a string. Dates and times are kept as text, so only their kind is checked.
fn reads_as(want Json, got Json) bool
    match want
        Json.Obj(ms) =>
            t := want.str("type")
            v := want.str("value")
            if ms.len() == 2 and t != none and v != none
                return leaf(want.str("type") else "", want.str("value") else "", got)
            have := got.as_obj() else return false
            if have.len() != ms.len()
                return false
            for m in ms
                g := got.get(m.key) else return false
                if not reads_as(m.value, g)
                    return false
            true
        Json.Arr(items) =>
            have := got.as_arr() else return false
            if have.len() != items.len()
                return false
            for i in 0..items.len()
                if not reads_as(items[i], have[i])
                    return false
            true
        Json.Null | Json.Bool(_) | Json.Num(_) | Json.Str(_) => false

fn leaf(t str, v str, got Json) bool
    match got
        Json.Str(s) => if t == "string" then s == v else t.starts_with("date") or t.starts_with("time")
        Json.Bool(b) => t == "bool" and (v == "true") == b
        Json.Num(f) => number_reads_as(t, v, f)
        Json.Null | Json.Arr(_) | Json.Obj(_) => false

fn number_reads_as(t str, v str, f float) bool
    if t != "integer" and t != "float"
        return false
    if v.ends_with("inf")
        return f == (if v.starts_with("-") then -1.0 / 0.0 else 1.0 / 0.0)
    if v.ends_with("nan")
        return f != f
    plain := if v.starts_with("+") then v.slice(1, v.len()) else v
    want := plain.to_float() else return true
    f == want

fn check_valid(c Case) bool
    doc := toml.parse(c.src) else err
        print("FAIL valid {c.name}: refused: {err}")
        return false
    want := parse_json(c.want) else return false
    if not reads_as(want, doc)
        print("FAIL valid {c.name}: read as {doc.to_text()}")
        return false
    true

fn check_invalid(c Case) bool
    doc := toml.parse(c.src) else return true
    print("FAIL invalid {c.name}: accepted as {doc.to_text()}")
    false

'''

YAML_HEAD = '''## std.yaml against yaml-test-suite: the cases inside the subset (std.yaml.7) read as the
## suite's JSON, valid YAML outside it is refused or read exactly, and invalid YAML is refused.
## The cases are the suite's files verbatim; tools/conformance/gen_suite_tests.py writes this
## file. yaml-test-suite is MIT licensed (NOTICE).
## Known gaps, invalid documents std.yaml still reads: CML9 (a comment between two words of a flow
## entry) and Y79Y/000 (a tab-only line after a block scalar header); neither is in a list.
use std.yaml

## A case: its id, the YAML text and the suite's JSON ("" when it has none to compare).
struct Case
    name str
    src str
    want str

## The text of the document `c` reads as, or "" when std.yaml refuses it.
fn reads(c Case) str
    doc := yaml.parse(c.src) else return ""
    doc.to_text()

fn expected(c Case) str
    j := parse_json(c.want) else return ""
    j.to_text()

fn check_strict(c Case) bool
    got := reads(c)
    if got == "" or got != expected(c)
        print("FAIL strict {c.name}: read as {got}")
        return false
    true

fn check_lenient(c Case) bool
    got := reads(c)
    if got != "" and got != expected(c)
        print("FAIL lenient {c.name}: read as {got}")
        return false
    true

fn check_invalid(c Case) bool
    got := reads(c)
    if got != ""
        print("FAIL invalid {c.name}: accepted as {got}")
        return false
    true

'''


def yaml_cases(root, strict_ids, invalid_ids):
    strict, lenient, invalid = [], [], []
    for d in sorted(os.listdir(root)):
        p = os.path.join(root, d)
        if not os.path.isdir(p) or d.startswith('.') or d in ('name', 'tags'):
            continue
        subs = [(d, p)] if os.path.exists(p + '/in.yaml') else [
            (d + '/' + e, p + '/' + e) for e in sorted(os.listdir(p)) if os.path.exists(p + '/' + e + '/in.yaml')]
        for name, q in subs:
            src = open(q + '/in.yaml', encoding='utf-8').read()
            if name in invalid_ids:
                invalid.append((name, src, ''))
            elif os.path.exists(q + '/in.json') and not os.path.exists(q + '/error'):
                txt = open(q + '/in.json', encoding='utf-8').read()
                try:
                    want = json.dumps(json.loads(txt), ensure_ascii=False, separators=(',', ':'))
                except ValueError:
                    # no document: std.yaml reads Null; several documents: it must refuse
                    want = 'null' if not txt.strip() else ''
                (strict if name in strict_ids else lenient).append((name, src, want))
    return strict, lenient, invalid


def listed(path):
    return set(l.strip() for l in open(path, encoding='utf-8') if l.strip() and not l.startswith('#'))


def runner(title, lists, check):
    """`main` text: every list is read into `all`, then each case is checked."""
    body = 'fn main()\n'
    for var, names, fn, label in lists:
        body += '    mut %s := List[Case]()\n' % var
        body += ''.join('    for c in %s()\n        %s.push(c)\n' % (n, var) for n in names)
    body += '    mut failed := 0\n'
    for var, names, fn, label in lists:
        body += '    mut %s_bad := 0\n    for c in %s\n        if not %s(c)\n            %s_bad += 1\n' % (var, var, fn, var)
        body += '    print("%s {%s.len() - %s_bad}/{%s.len()} %s")\n' % (title, var, var, var, label)
        body += '    failed += %s_bad\n' % var
    body += '    if failed > 0\n        print("{failed} cases failed")\n'
    return body


def main():
    toml_root, yaml_root = sys.argv[1], sys.argv[2]
    here = os.path.dirname(os.path.abspath(__file__))
    out = here + '/../../tests/cases/'
    valid, invalid = toml_cases(toml_root)
    vf, vn = chunks('valid', valid)
    nf, nn = chunks('invalid', invalid)
    body = TOML_HEAD + '\n'.join(vf + nf) + '\n' + runner('toml-test', [
        ('good', vn, 'check_valid', 'valid documents read as the suite says'),
        ('bad', nn, 'check_invalid', 'invalid documents refused')], None)
    open(out + 'std_toml_suite.ail', 'w', encoding='utf-8').write(body)
    open(out + 'std_toml_suite.out', 'w', encoding='utf-8').write(
        'toml-test %d/%d valid documents read as the suite says\ntoml-test %d/%d invalid documents refused\n' % (
            len(valid), len(valid), len(invalid), len(invalid)))

    strict, lenient, bad = yaml_cases(yaml_root, listed(here + '/yaml_strict.txt'), listed(here + '/yaml_invalid.txt'))
    sf, sn = chunks('strict', strict)
    lf, ln = chunks('lenient', lenient)
    bf, bn = chunks('invalid', bad)
    body = YAML_HEAD + '\n'.join(sf + lf + bf) + '\n' + runner('yaml-test-suite', [
        ('inside', sn, 'check_strict', 'cases in the subset read as the suite says'),
        ('outside', ln, 'check_lenient', 'valid cases outside the subset refused or read exactly'),
        ('invalid', bn, 'check_invalid', 'invalid cases refused')], None)
    open(out + 'std_yaml_suite.ail', 'w', encoding='utf-8').write(body)
    open(out + 'std_yaml_suite.out', 'w', encoding='utf-8').write(
        'yaml-test-suite %d/%d cases in the subset read as the suite says\n'
        'yaml-test-suite %d/%d valid cases outside the subset refused or read exactly\n'
        'yaml-test-suite %d/%d invalid cases refused\n' % (len(strict), len(strict), len(lenient), len(lenient), len(bad), len(bad)))


if __name__ == '__main__':
    main()
