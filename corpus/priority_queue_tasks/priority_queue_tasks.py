import heapq


def submit(queue, counter, priority, name):
    counter[0] += 1
    heapq.heappush(queue, (priority, counter[0], name))


def run_all(queue):
    order = []
    while queue:
        p, _, name = heapq.heappop(queue)
        order.append(f"{name}({p})")
    return order


queue = []
counter = [0]
submit(queue, counter, 3, "backup")
submit(queue, counter, 1, "deploy")
submit(queue, counter, 2, "lint")
submit(queue, counter, 1, "migrate")
submit(queue, counter, 3, "report")
submit(queue, counter, 2, "test")
print(f"queued {len(queue)} tasks")
print(f"next up: {queue[0][2]}")
print(" ".join(run_all(queue)))
print("empty now: " + str(len(queue) == 0).lower())
best = []
for score in [42, 7, 19, 88, 63, 5, 71]:
    heapq.heappush(best, -score)
top3 = []
for _ in range(3):
    top3.append(-heapq.heappop(best))
print(f"top 3 scores: {top3}")
