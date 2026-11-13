class OrderError(Exception):
    pass


EVENTS = ["pay", "ship", "deliver", "cancel", "refund"]

RULES = {
    "created": {"pay": "paid", "cancel": "cancelled"},
    "paid": {"ship": "shipped", "cancel": "cancelled", "refund": "refunded"},
    "shipped": {"deliver": "delivered"},
    "delivered": {"refund": "refunded"},
    "cancelled": {},
    "refunded": {},
}


def parse_event(text):
    if text in EVENTS:
        return text
    raise OrderError(f"unknown event '{text}'")


def step(s, ev):
    if ev not in RULES[s]:
        raise OrderError(f"cannot {ev} an order that is {s}")
    return RULES[s][ev]


def run_order(oid, events):
    state = "created"
    trail = ["created"]
    for name in events.split(","):
        try:
            ev = parse_event(name)
            state = step(state, ev)
            trail.append(state)
        except OrderError as err:
            print(f"order {oid}: {err}")
            continue
    print(f"order {oid}: {state} [{'>'.join(trail)}]")


run_order(1, "pay,ship,deliver")
run_order(2, "ship,pay,cancel,pay,refund")
run_order(3, "pay,ship,cancel,deliver,refund,refund")
run_order(4, "pay,teleport,refund")
run_order(5, "deliver")
