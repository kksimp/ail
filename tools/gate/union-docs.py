import re,sys
for p in sys.argv[1:]:
    s=open(p).read()
    def fix(m):
        a=m.group(1).splitlines(True); b=m.group(2).splitlines(True)
        return "".join(a+[l for l in b if l not in a])
    s=re.sub(r"<<<<<<< [^\n]*\n(.*?)=======\n(.*?)>>>>>>> [^\n]*\n", fix, s, flags=re.S)
    open(p,'w').write(s)
