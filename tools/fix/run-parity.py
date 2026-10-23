#!/usr/bin/env python3
"""`ail fix` in AIL (compiler fixrun) against the Rust `ail fix` on copies of the given files.

  AIL_RUST=bootstrap/target/release/ail AIL_COMPILER=./compiler-native tools/fix/run-parity.py FILE...
"""
import subprocess, sys, shutil, os, tempfile
R=os.environ.get("AIL_RUST", "bootstrap/target/release/ail"); A=os.environ.get("AIL_COMPILER", "compiler-native")
bad=n=0
files=sys.argv[1:]
for f in files:
  for flags in ([],["--dry-run"],["--dry-run","--json"],["--json"]):
    d1=tempfile.mkdtemp(); d2=tempfile.mkdtemp()
    p1=os.path.join(d1,"t.ail"); p2=os.path.join(d2,"t.ail")
    shutil.copy(f,p1); shutil.copy(f,p2)
    a=subprocess.run([R,"fix",p1]+flags,capture_output=True,text=True)
    b=subprocess.run([A,"fixrun",p2]+flags,capture_output=True,text=True,env=dict(os.environ,AIL_RUST=R))
    n+=1
    norm=lambda s,p:s.replace(p,"T")
    same=(norm(a.stdout,p1),norm(a.stderr,p1),a.returncode,open(p1).read() if '--dry-run' not in flags else '')==(norm(b.stdout,p2),norm(b.stderr,p2),b.returncode,open(p2).read() if '--dry-run' not in flags else '')
    if not same:
      bad+=1
      if bad<4: print("DIFF",f,flags,a.returncode,b.returncode,repr(norm(a.stdout,p1)[:300]),repr(norm(b.stdout,p2)[:300]),repr(norm(a.stderr,p1)[:200]),repr(norm(b.stderr,p2)[:200]))
    shutil.rmtree(d1);shutil.rmtree(d2)
print(n,bad)
