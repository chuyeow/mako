# Mako design rules

Rules for how Mako looks and behaves, each with its reason. Read this before building or changing UI. The comment under each rule holds the date and the owner's words, so a later rule that conflicts shows which is newer.

## Color

- **Every surface is cream paper with brown ink. Never system greys, white or black.** Use the adaptive tokens in `app/Sources/Mako/Sidebar.swift`: `NSColor.sidebarPaper` (`0xf1e8d6`), the card paper `0xfbf7ee`, `NSColor.ink`, `NSColor.caution`, `NSColor.scrim`, `NSColor.cardShadow`. Colours are defined only there; a new colour is a new token with a light and a dark value.
  Why: the owner likes cream colours like watercolor paper.
  <!-- 2026-11-13: "I like cream colors like watercolor paper" -->

- **Anything drawn over the page takes the active trail's colours.** The omnibox, link-hint chips, find and any future overlay use `NSColor.card(slot)` for the fill and `NSColor.trail(slot)` for the ink. With no trail, they fall back to `NSColor.sidebarPaper` and `NSColor.ink`.
  Why: the chrome should match the sidebar, so it shows which trail you're in.
  <!-- 2026-11-14: "make the address bar more colorful, matching the sidebar" -->

## Spacing & layout

- **No toolbar, tab strip, title bar or other permanent bar.** A new control goes in the sidebar, in a menu item with a keyboard shortcut, or in an overlay that appears on demand. Moving and resizing the window must keep working without the title bar.
  Why: the owner wants the window as chromeless as possible, while still being able to move and resize it.
  <!-- 2026-11-14: "I wanna make the browser window as chromeless as possible ... but still provide affordances to manipulate the window and drag the window" -->

## Interaction

- **Every mode is obvious, but subtly.** When Mako is in a mode (find vs go-to in the omnibox, F vs ⇧F link hints, any future mode), show a cue you can't miss but that doesn't shout: a placeholder, an icon, a tint or a short label. Never a second surface for the same job, and never a loud colour or animation.
  Why: you should know which mode you're in before you type, without the cue pulling your eye.
  <!-- 2026-11-14: "make Cmd+F look different from entering a URL (subtly)"; 2026-11-15: "it's a more general principle to make [modes] obvious, but subtly" -->

- **Link-hint labels never use F, and start with `qweasdzxc`.** `core/src/hints.rs` holds the alphabet and its tests enforce both.
  Why: hints are a one-handed operation; the other hand is usually on the mouse or not used. F starts hints, so it can't also pick one.
  <!-- 2026-11-15: "dont re-use F. prioritize these chars: qweasdzxc"; "so it can be a one-handed operation. the other hand [is] usually on the mouse or not used" -->
