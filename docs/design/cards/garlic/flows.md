# garlic: flows
tests: flows.flows.flows_are_saved_per_site_replaced_by_name_and_found_again

## flows
```
pub struct flows.Flow  ## `{host: {name: steps}}`.
    pub host str
    pub name str
    pub steps List[str]
pub fn flows.default_path() str [env]  ## Where flows are kept: `$GARLIC_FLOWS`, else `~/.garlic-flows.json`.
pub fn flows.load(path str) List[flows.Flow] [fs io]  ## Every saved flow in the file at `path`; none when it is missing or unreadable.
fn flows.render(all List[flows.Flow]) str
pub fn flows.save(path str, host str, name str, steps List[str]) !fs.IoError [fail fs io]  ## Saves `steps` as flow `name` of `host`, replacing one of that name.
    fails with fs.IoError
pub fn flows.find(all List[flows.Flow], host str, name str) flows.Flow?  ## The flow `name` for `host`; else the only flow of that name on any site.
pub fn flows.list_json(all List[flows.Flow], host str) str  ## `{"flows":[{"host","name","steps"}]}` for `host`, or for every site when it is "".
pub fn flows.split_step(step str) (str, str)  ## A step split into its kind and the rest.
```
