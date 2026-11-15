# Feature: customer returns

Customers can now bring back units they received. Extend `till` (spec in `DOCS/spec.md`, sources in `src/`) as follows, and keep everything else as specified there.

- A new entry, `return ORDER SKU QTY` (QTY 1 to 5 digits, at least 1), records that QTY units of SKU from a shipped order came back. Its rules, checked in this order:
  1. `unknown order ORDER` if ORDER was never placed.
  2. `order ORDER is STATUS` if the order is not `shipped`.
  3. `SKU not in ORDER` if no line of the order is for SKU.
  4. `too many SKU` if QTY is more than the line's QTY minus the units of that line already returned.
- Effect: the SKU gains QTY units on hand. With R the units of the line returned before this entry and R2 = R + QTY, the return takes back three shares of the line, each computed as `A × R2 / QTY − A × R / QTY` (integer division, QTY being the line's quantity): of the line's net (A = net), of its tax (A = tax) and of its cost (A = the cost recorded when it shipped). The customer's credit grows by the net share plus the tax share, and the SKU's stock value grows by the cost share. Returning every unit of a line, in any number of steps, takes back exactly its net, tax and cost.
- When every unit of every line of the order has been returned, the order's status becomes `returned`. A `returned` order cannot be cancelled (`order ORDER is returned`).
- Reports:
  - `invoice`: after the `total` line, a line `returned` with the sum of the net and tax shares taken back so far, printed only when that sum is above 0.
  - `customers`: SPENT counts `shipped` and `returned` orders, each as its total minus what returns took back.
  - `sales` and `margin`: count the lines of `shipped` and `returned` orders, less what returns took back: UNITS less the returned units, NET less the net shares, TAX less the tax shares, COST less the cost shares. A category is listed if it has a line in such an order, even when everything was returned.
  - `orders`, `history`: show the `returned` status; TOTAL and PAID are unchanged by returns.

Keep `./test` passing; hidden cases cover the whole spec as changed here.
