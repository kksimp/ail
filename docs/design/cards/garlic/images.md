# garlic: images

## images
```
images.MAX_IMAGES := 24  ## Pictures fetched per page, and the largest one taken.
images.MAX_BYTES := 16000000
pub struct images.Bitmap  ## A decoded picture for the window host: its pixel size and its `R, G, B, A`
    pub w int
    pub h int
    pub data str
images.MAX_PIXELS := 4000000  ## Largest picture (in pixels) that is decoded for the host.
pub fn images.prefetch(d html.Doc, url str, from str, cj std.net.jar.Jar, o net.Options, mut sizes Map[str, List[int]], mut bits Map[str, images.Bitmap]) [clock fs io mutates net]  ## The `[width, height]` of every `<img>` on `d` (by resolved address) that
fn images.load(at str, base net.Options, from str, cj std.net.jar.Jar) List[int]? [clock fs io net]  ## The bytes of the picture at `at`, or none.
```
