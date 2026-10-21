# garlic: jsdom

## jsdom
```
pub struct jsdom.Outcome  ## What running a page's scripts left for the browser to do.
    pub url str  ## A navigation the page asked for (the first one), or "".
    pub method str
    pub body str
    pub console List[str]  ## Console lines and uncaught errors, in order.
    pub missing List[str]  ## External scripts the page wanted that weren't in `scripts`.
    pub changes int  ## How many changes were replayed.
    pub cookies List[str]  ## What scripts assigned to `document.cookie`, in order.
pub fn jsdom.run(mut d html.Doc, url str, scripts Map[str, str], cookies Map[str, str], now_ms float, budget_ms float, max_steps int) jsdom.Outcome [io mutates]  ## Run the scripts of `d` (the page at `url`): inline ones, and external ones
fn jsdom.mirror(d html.Doc) List[garlicjs.vm.DomNode]  ## The engine's copy of the tree: the same nodes with the same ids.
fn jsdom.replay(mut d html.Doc, log List[garlicjs.vm.Mutation]) int [mutates]  ## Apply the engine's log to `d`; new nodes get the same ids in both trees.
```
