# Mr. Scroll

[Visit the Ludum 31 entry](http://ludumdare.com/compo/ludum-dare-31/?action=preview&uid=31244)

**Theme: Entire Game on One Screen**

Mr. Scroll is a scrolling platformer in which the player wraps across the screen.
You use the scrolling mechanic to progress towards the exit.

## Controls
* W/A/S/D or Arrow keys: Move
* Up: Climb
* Down: Open chest
* Hold Ctrl: Lock scrolling
* Space: Fire gun

## Notes

This is my entry for the Ludum Dare 31 Jam, written entirely in Rust.
In addition, I used the Tiled map editor and PyxelEdit for tilesets.

All audio is synthesized in-game, and resembles NES-style audio
(there are 4 channels after all: 2 pulses, 1 triangle, and 1 noise).

**Note:** There is a lot of cleanup that needs to be done.
In fact, a lot of the code is simply horrible; I mostly attribute this to reckless code-vomiting in the face of an ever-nearing deadline.
The game logic in particular is very ad-hoc.

## Todo
- [ ] Move `rust-game-platforms` to its own repo
- [ ] Move `synth` to its own repo
- [ ] Make rendering code more polymorphic (or at least more DRY)
- [ ] Clean up Tiled JSON level loader
- [ ] Simplify collision
