def invert(e):
    return ("del" if e[0] == "ins" else "ins", e[1], e[2])


def apply(doc, e):
    kind, at, text = e
    if kind == "ins":
        return doc[:at] + text + doc[at:]
    return doc[:at] + doc[at + len(text):]


class History:
    def __init__(self):
        self.doc = ""
        self.done = []
        self.undone = []

    def edit(self, e):
        self.doc = apply(self.doc, e)
        self.done.append(e)
        self.undone.clear()

    def undo(self):
        if not self.done:
            return False
        e = self.done.pop()
        self.doc = apply(self.doc, invert(e))
        self.undone.append(e)
        return True

    def redo(self):
        if not self.undone:
            return False
        e = self.undone.pop()
        self.doc = apply(self.doc, e)
        self.done.append(e)
        return True

    def status(self):
        return f'"{self.doc}" (undo {len(self.done)}, redo {len(self.undone)})'


h = History()
h.edit(("ins", 0, "hello world"))
print(h.status())
h.edit(("ins", 5, ","))
h.edit(("del", 7, "world"))
h.edit(("ins", 7, "there"))
print(h.status())
for step in range(1, 4):
    h.undo()
    print(f"undo {step}: {h.status()}")
h.redo()
print(f"redo: {h.status()}")
h.edit(("ins", 0, ">> "))
print(f"new edit: {h.status()}")
print("redo possible: " + str(h.redo()).lower())
while h.undo():
    pass
print(f"all undone: {h.status()}")
