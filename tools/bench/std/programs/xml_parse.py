import sys
import xml.etree.ElementTree as ET
n = int(sys.argv[1]) if len(sys.argv) > 1 else 50000
text = "<root>" + "".join('<item id="%d" kind="k%d"><name>item %d</name><v>%d</v></item>' % (i, i % 5, i, i % 97) for i in range(n)) + "</root>"
root = ET.fromstring(text)
ids = vs = nc = 0
items = root.findall("item")
for it in items:
    ids += int(it.get("id"))
    for kid in it:
        if kid.tag == "v":
            vs += int(kid.text)
        else:
            nc += len(kid.text)
print("items %d, ids %d, v %d, name chars %d" % (len(items), ids, vs, nc))
