# edit: themes

## themes
```
pub themes.NAMES := ...  ## The theme names, in the order Ctrl-Shift-T cycles through them.
pub fn themes.named(name str) Map[str, int]?  ## The theme called `name`, or `none` for an unknown name.
themes.LIGHT := ...
themes.SOLARIZED := ...
themes.HIGH_CONTRAST := ...
```
