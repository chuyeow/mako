# Link hints

Press F on a page and every link and control in view gets a short label, in the active trail's colours. Typing a label clicks what it marks, with a real click, so links open beneath the page as a clicked link does, buttons press, text fields take focus and download links download. ⇧F labels only web links, and the chosen one opens beneath the page without leaving it, like a middle-click. F only starts hints when the page and its text fields leave the key alone: typing F in a field types it, and a page that handles F itself (a video player's full-screen key) keeps it. History → Jump to Link… (⌘J) and Open Link Beneath… (⇧⌘J) work on any page. Labels use Q W E A S D Z X C first, then R T G V B, so the right hand can stay on the pointer; never F, which starts hints. They are as short as the count allows, and none is the start of another, so a label acts the moment its last letter is typed.

## Sub-features

- `hints-show` F or ⌘J labels exactly the links and controls in view: not hidden, disabled (a disabled `<fieldset>` included), covered or scrolled-out ones. A partly covered link is labelled, and its label clicks the part that shows.
- `hints-follow` typing a label, or pressing its label through AX, clicks the element with a real, trusted click.
- `hints-beneath` ⇧F or ⇧⌘J labels only web links (not `javascript:` or download links) and opens the chosen one beneath, staying on the page; it loads at once, quietly, as a middle-clicked link does.
- `hints-narrow` the first letter of a two-letter label hides the labels that do not start with it; Delete takes a letter back; a letter no label has beeps and changes nothing.
- `hints-end` Escape, a shortcut (which then acts), a scroll, a click anywhere but a label, opening the address bar, switching pages and any resize of the page (Hide Sidebar, window resize) end hints. A key typed after a menu focused another field (Rename Page…) ends hints and goes to that field. A page with nothing to label beeps.
- `hints-keys` F in a text field types; a page that handles F keeps it; ⌘J labels anyway.

## How to get to it (user POV)

- Press F (or ⇧F) with a page focused, or History → Jump to Link… / Open Link Beneath….
- Type the letters on the label. Escape cancels.

## Driving it with makoctl

Serve `fixtures/hints/` with `python3 -m http.server`. `index.html` has two links, a half-covered link, a button in a disabled fieldset, a button that titles the page `pressed trusted` on a trusted click, a text field that titles it `typing` on focus and `typed <text>` on input, a `javascript:` link, a download link, and a hidden link, a disabled button, a covered link and a link below the fold that must get no label. `many.html` has 20 links, so labels 14–20 are two letters (`bq`…`bz`). `eats-f.html` handles F itself.

- **Labels.** `menu History "Jump to Link…"`, then `tree 30 | grep mako.hint`: `s` two.html, `a` three.html, `d` Press, `f` Type here, `j` Script, `k` Save, `l` three.html?half (a link whose right part is covered), nothing else (no label for the button in the disabled fieldset). `shot` shows them.
- **Follow.** `press-id mako.hint.q`; `wait-title Two`; in `trails.json` Two's parent is Hints.
- **Beneath.** `menu History "Open Link Beneath…"`: only `q`, `w` and `e`. `press-id mako.hint.w`: `three.html` appears in `menu-items Trails`, `title` unchanged.
- **Keys (focus).** `key f`, `key q`; on `many.html` `key f`, `key b` (7 labels), `key f` (still 7: F is no label), `key delete` (20), `key b`, `key q` (the active page is `two.html?14`); `key escape`; `key shift+f`; on `eats-f.html` `key f` titles it `page got f` with no labels, `key cmd+j` labels anyway.
- **Scripts.** `scripts/hints-ax` (background) and `scripts/hints-keys-headed` (keys only).

## Gotchas

- Labels exist in the AX tree only while hints show, under `mako.page`.
- The page script reads the main frame only: links inside iframes and closed shadow roots get no label.
- Hints refuse with a beep on a pinch-zoomed page (the page's own zoom, ⌘= and ⌘-, is fine).
- A label press goes through AX without focus; F and typed letters need focus.
