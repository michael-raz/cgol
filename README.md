# Conway's Game of Life
This is a Rust implementation of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway's_Game_of_Life) (**cgol**).
It's intended to be a web app that can also run natively on Linux (maybe Windows and macOS later on).

---

## Features
- [x] Infinite canvas
- [x] Save state (in base64 using clipboard)
- [ ] QOL for the editor
	- [x] Undo and redo
	- [x] Rectangle selection
	- [x] Copy-paste
	- [ ] Blueprints
- [ ] Coloring
- [x] Adjustable play rate with pause/unpause
- [ ] Custom rules via rulestrings (i.e. different "universes")
- [ ] Provide library bindings
- [ ] Reverse search (low priority)
