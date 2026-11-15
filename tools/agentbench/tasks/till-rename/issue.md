# Change: customers are now accounts

The shop now calls its customers *accounts*, in the journal, in the reports and in the code. Change `till` (spec in `DOCS/spec.md`, sources in `src/`) as follows, and keep everything else as specified there.

- The journal entry `customer ID TIER` becomes `account ID TIER`, with the same rules and effect. A line starting with `customer` is now a bad line.
- The report `customers` becomes `accounts`, with the same lines. `customers` is now an unknown command.
- The usage line becomes `usage: till orders|invoice ORDER|stock|low N|accounts|history ID|sales|margin|purchases|gifts|promos`.
- The messages `unknown customer ID` and `error: no customer ID` become `unknown account ID` and `error: no account ID`.
- In the code, rename every type, function, variable, field, module and file that names a customer, and update comments to match: when you are done, no file under `src/` may contain `cust` in any letter case, in its name or its text (a hidden check fails otherwise). Do not change behaviour in any other way.

Keep `./test` passing; hidden cases cover the whole spec as changed here.
