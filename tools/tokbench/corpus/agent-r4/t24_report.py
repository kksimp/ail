sales = {"north": 12.5, "south": 7.25}
total = sum(sales.values())
for region, amount in sorted(sales.items()):
    print(f"{region:<8}{amount:>8.2f} {amount / total:>6.1%}")
