# garlic: hush
tests: hush.hush.a_page_about_gdpr_keeps_its_article, hush.hush.banners_ads_and_trackers_are_hidden_and_the_rest_stays

## hush
```
hush.BANNER_WORDS := ...  ## Words in an element's `id` or `class` that mark a consent banner.
hush.AD_WORDS := ...  ## Words in an element's `id` or `class` that mark an advertisement.
hush.TRACKERS := ...  ## Hosts whose frames and images are trackers.
hush.KEEP := ...  ## Elements never hidden even if a class says so (pages put state classes on them).
fn hush.has_word(s str, words List[str]) bool
fn hush.tracker_src(src str) bool
pub fn hush.is_noise(d html.Doc, id int) bool  ## Whether element `id` is a banner, an ad or a tracker.
pub fn hush.apply(mut d html.Doc) int [mutates]  ## Hides every banner, ad and tracker of `d` (not those inside one already
```
