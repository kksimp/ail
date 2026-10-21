import subprocess
r = subprocess.run(["git", "status", "--short"], capture_output=True, text=True, timeout=10)
print(r.returncode, r.stdout.strip())
