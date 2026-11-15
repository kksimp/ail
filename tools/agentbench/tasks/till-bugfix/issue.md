# Issue: tier discounts are a cent short

`till` (spec in `DOCS/spec.md`, sources in `src/`) gives silver and gold customers a tier discount that is sometimes one cent smaller than the spec's rule. With

```
item LAMP home 1995
customer ann gold
restock LAMP 5
order o1 ann LAMP:1
```

`./till invoice o1` shows a discount of `1.99`; by the spec (`percent(1995, 10)`, halves up) it is `2.00`, and the line total is `17.95`, not `17.96`. Totals, payments, `orders`, `customers` and `sales` carry the error.

Find the cause and fix it so that every amount follows the spec's "Prices" section, and keep `./test` passing. Hidden cases cover more than this example. Do not change the spec's behaviour elsewhere.
