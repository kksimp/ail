import json, os, shutil, tempfile

PEOPLE = """
[
  {"name": "Ann", "age": 31, "tags": ["admin", "dev"], "email": "ann@example.com"},
  {"name": "Bob", "age": 27, "tags": []},
  {"name": "Cy", "age": 45, "tags": ["ops"], "email": null}
]
"""


def decode(j):
    return {"name": j["name"], "age": j["age"], "tags": j["tags"], "email": j.get("email")}


def birthday(people):
    return [dict(p, age=p["age"] + 1) for p in people]


def describe(p):
    tags = "-" if not p["tags"] else "+".join(p["tags"])
    email = p["email"] if p["email"] is not None else "none"
    return f"{p['name']:<4}{p['age']:>3} tags={tags} email={email}"


def main():
    d = tempfile.mkdtemp(prefix="corpus_json_roundtrip")
    file = os.path.join(d, "people.json")
    with open(file, "w", newline="") as f:
        f.write(PEOPLE)
    with open(file, newline="") as f:
        people = [decode(j) for j in json.loads(f.read())]
    print(f"loaded {len(people)} people")
    for p in people:
        print(describe(p))
    older = birthday(people)
    with open(file, "w", newline="") as f:
        f.write(json.dumps(older, separators=(",", ":")))
    with open(file, newline="") as f:
        again = [decode(j) for j in json.loads(f.read())]
    print(f"round trip equal: {str(again == older).lower()}")
    print(f"ages: [{', '.join(str(q['age']) for q in again)}]")
    doc = json.loads('{"id": "a1", "ok": true, "names": ["x", "y"], "gone": null}')
    print(json.dumps(doc, separators=(",", ":")))
    print(f"id={doc.get('id', '?')} ok={str(doc.get('ok', False)).lower()} missing={doc.get('nope', 'dflt')}")
    bad = {"name": "Dee", "tags": []}
    if "age" in bad:
        print("unexpected")
    else:
        print("rejected a person without an age")
    shutil.rmtree(d)


main()
