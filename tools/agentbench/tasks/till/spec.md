# till

`till` is a command-line program for a small shop's back office. It reads a journal on stdin and prints one report on stdout.

## Fields

Fields are separated by exactly one space. Numbers may have leading zeros (`007` is 7).
- SKU: 1 to 12 characters of `A-Z 0-9 -`, the first one a letter `A-Z`.
- CAT (category): 1 to 12 letters `a-z`.
- ID (customer): 1 to 12 characters of `a-z 0-9`, the first one a letter `a-z`.
- ORDER: `o` followed by 1 to 6 digits. Orders are compared as text (`o1` and `o01` are different orders).
- PO (purchase order): `p` followed by 1 to 6 digits, compared as text.
- CARD (gift card): `g` followed by 1 to 6 digits, compared as text.
- CODE (promo code): 3 to 10 characters of `A-Z 0-9`, the first one a letter `A-Z`.
- TIER: `basic`, `silver` or `gold`.
- QTY: 1 to 5 digits, at least 1 (`adjust` also allows 0).
- CENTS: 1 to 7 digits, an amount in cents.
- PCT: 1 or 2 digits, a percentage from 0 to 99.
- USES: 1 to 3 digits, at least 1.

## Journal

One entry per line. An empty line or a line starting with `#` is ignored, but still counts for line numbers. A trailing newline at the end of input is not an extra line. A carriage return is part of the line, so a line ending in `\r` is not a valid entry.

A line that is not one of the entries below is a bad line. An entry is checked in the order its rules are listed; the first rule it breaks decides its error, and a rejected entry changes nothing. The error goes to stderr as `error: line K: MESSAGE`, where K is the 1-based line number. In the messages below, the capitalised words are replaced by the entry's fields.

- `item SKU CAT CENTS`: adds SKU to the catalog in category CAT at price CENTS, or changes the price of a SKU already there.
  1. A SKU already in the catalog keeps its category: a different CAT is `category changed SKU`.
- `tax CAT PCT`: sets the tax rate of category CAT to PCT percent. Every category starts at 0. The category does not need to have items.
- `restock SKU QTY`: adds QTY units to the SKU's on-hand quantity, at no cost (its stock value does not change). Every SKU starts with 0 on hand and a stock value of 0.
  1. `unknown item SKU` if SKU is not in the catalog.
- `adjust SKU QTY` (QTY may be 0): a stocktake; sets the on-hand quantity to QTY. If the old on-hand quantity was above 0, the stock value becomes value × QTY / old quantity (rounded down); otherwise it is unchanged.
  1. `unknown item SKU` if SKU is not in the catalog.
  2. `below reserved SKU` if QTY is less than the SKU's reserved quantity.
- `buy PO BUYS`: a purchase order to a supplier. BUYS is one or more `SKU:QTY@CENTS` separated by commas, e.g. `A1:10@250,B-7:5@1000`: QTY units of SKU at CENTS each. A SKU listed twice makes it a bad line. The purchase order's status is `ordered`.
  1. `duplicate purchase PO` if PO was already entered.
  2. `unknown item SKU` for the first listed SKU not in the catalog.
- `receive PO`: the goods of a purchase order arrive. Each listed SKU gains QTY units on hand and QTY × CENTS of stock value. The status becomes `received`.
  1. `unknown purchase PO` if PO was never entered.
  2. `purchase PO is received` if it was already received.
- `gift CARD CENTS`: sells a gift card holding CENTS (at least 1, else a bad line).
  1. `duplicate gift CARD` if CARD was already sold.
- `customer ID TIER`: registers a customer, or changes the tier of one already registered. A new customer has 0 credit.
- `promo CODE CAT PCT USES`: defines a promo code that takes PCT percent off lines of category CAT, usable on at most USES orders.
  1. `duplicate promo CODE` if CODE is already defined.
- `order ORDER ID ITEMS`: places an order for customer ID. ITEMS is one or more `SKU:QTY` separated by commas, e.g. `A1:2,B-7:1`. A SKU listed twice makes it a bad line.
  1. `duplicate order ORDER` if ORDER was already placed.
  2. `unknown customer ID` if ID is not registered.
  3. `unknown item SKU` for the first listed SKU not in the catalog.
  4. `short SKU` for the first listed SKU whose available quantity (on hand minus reserved) is less than its QTY.

  Otherwise the order is placed with status `open`, and each listed SKU's reserved quantity grows by its QTY. Each line keeps the item's price, its category's tax rate and the customer's tier discount as they are now; later `item`, `tax` and `customer` entries do not change a placed order.
- `apply ORDER CODE`: puts a promo on an order.
  1. `unknown order ORDER` if ORDER was never placed.
  2. `order ORDER is STATUS` if the order is not `open` (STATUS is its status).
  3. `unknown promo CODE` if CODE is not defined.
  4. `promo already on ORDER` if the order already has a promo.
  5. `promo CODE used up` if CODE is already on USES orders.
  6. `promo CODE does not apply` if no line of the order is in the promo's category.

  Otherwise the order gets the promo, which counts one use, and its lines are priced again. If the order's payments now cover its total, the payments above the total go to the customer's credit and the order becomes `paid`.
- `pay ORDER CENTS`, `pay ORDER credit` or `pay ORDER CARD`: a payment towards an order.
  1. `unknown order ORDER` if ORDER was never placed.
  2. `order ORDER is STATUS` if the order is not `open`.
  3. For `credit`: `no credit ID` if the order's customer has 0 credit. For CARD: `unknown gift CARD` if CARD was never sold, then `gift CARD is empty` if its balance is 0.

  With CENTS (which must be at least 1, else a bad line), the amount is CENTS. With `credit`, the amount is the customer's credit or the amount due, whichever is smaller, and it is taken from the credit. With CARD, the amount is the card's balance or the amount due, whichever is smaller, and it is taken from the card. The amount due is the order's total minus what has been paid on it. The payment counts up to the amount due, and any amount above it goes to the customer's credit. When what has been paid equals the total, the order becomes `paid`.
- `ship ORDER`: ships an order.
  1. `unknown order ORDER` if ORDER was never placed.
  2. `order ORDER is STATUS` if the order is not `paid`.

  Line by line, in the listed order, each line's cost is the SKU's stock value × QTY / its on-hand quantity (rounded down); the stock value falls by that cost, and the on-hand and reserved quantities both fall by QTY. The order becomes `shipped`.
- `cancel ORDER`: cancels an order.
  1. `unknown order ORDER` if ORDER was never placed.
  2. `order ORDER is STATUS` if the order is `shipped` or `cancelled`.

  Each line's SKU loses QTY units from its reserved quantity. Everything paid on the order goes to the customer's credit and the order's paid amount becomes 0. If the order has a promo, the promo gets that use back (the order keeps showing the promo and its prices). The order becomes `cancelled`.

## Prices

All money is whole cents. `percent(A, P)` is A × P / 100 rounded to the nearest cent, halves up: `(A × P + 50) / 100` with integer division. Each line of an order is priced as follows:
- gross = price × QTY
- tier discount = percent(gross, T), where T is 0 for `basic`, 5 for `silver` and 10 for `gold`
- promo discount = percent(gross − tier discount, promo PCT) if the order has a promo and the line is in its category, else 0
- discount = tier discount + promo discount
- net = gross − discount
- tax = percent(net, the line's tax rate)
- line total = net + tax

The order's total is the sum of its line totals.

Money prints cents as dollars with exactly two decimals and a leading `-` for negatives: `1234.05`, `0.50`, `0.00`.

## Reports

The first argument picks the report. Values are separated by one tab. "Ascending" is ascending byte order.
- `orders`: one line per placed order, in the order they were placed: `ORDER ID STATUS TOTAL PAID`.
- `invoice ORDER`: the order's header `order ORDER ID STATUS`, then one line per order line in the listed order, `SKU QTY PRICE GROSS DISCOUNT TAX TOTAL`, then `promo CODE` if the order has a promo, then `subtotal` (the sum of gross), `discount`, `tax`, `total`, `paid` and `due`, each followed by its amount. `due` is the amount due for an `open` order and 0 otherwise.
- `stock`: one line per catalog SKU, ascending: `SKU CAT PRICE ONHAND RESERVED AVAILABLE VALUE`, VALUE being the stock value.
- `low N` (N is 1 to 5 digits): one line per catalog SKU whose available quantity is below N, ascending: `SKU AVAILABLE`.
- `customers`: one line per registered customer, ascending: `ID TIER ORDERS SPENT CREDIT`. ORDERS counts the customer's orders that are not `cancelled`; SPENT is the sum of the totals of their `shipped` orders.
- `history ID`: one line per order of customer ID, in the order they were placed, `ORDER STATUS TOTAL`, then `credit` and the customer's credit.
- `sales`: one line per category that has a line in a `shipped` order, ascending: `CAT UNITS NET TAX`, the sums of QTY, net and tax over those lines. Then `total UNITS NET TAX` over every shipped line (all zero if there is none).
- `margin`: one line per category that has a line in a `shipped` order, ascending: `CAT NET COST MARGIN`, the sums of net and cost over those lines and MARGIN = NET − COST. Then `total NET COST MARGIN` over every shipped line.
- `purchases`: one line per purchase order, in the order they were entered: `PO STATUS UNITS COST`, the sums of QTY and QTY × CENTS over its SKUs.
- `gifts`: one line per gift card, ascending: `CARD SOLD BALANCE`.
- `promos`: one line per promo code, ascending: `CODE CAT PCT% USED/USES`, e.g. `SPRING	toys	10%	1/3`.

PRICE, VALUE, the amounts and the money columns print as money; quantities and counts print as plain integers.

## Exit codes and stderr

- 1 for a missing, unknown or malformed command or argument: print `usage: till orders|invoice ORDER|stock|low N|customers|history ID|sales|margin|purchases|gifts|promos` to stderr and do not read stdin.
- 3 if `invoice` names an order that was never placed (`error: no order ORDER` on stderr, after any line errors, and nothing on stdout) or `history` names a customer never registered (`error: no customer ID`).
- 2 if any line was rejected (the report is still printed).
- 0 otherwise.

Nothing else is printed to stderr.

Layout: the program lives in `src/`; `./build` builds `./till`; `./test` runs the visible cases.
