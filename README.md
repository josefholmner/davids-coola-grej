# Number Catch — Growing Tree

Catch falling additions whose answers equal the target. Every correct catch grows a tree beside the play area; wrong catches shrink it.

## Run

```powershell
python game.py
```

Requires Python 3.9 or newer. No extra packages are needed on Windows: `windows_graphics.py` provides native graphics when Tkinter is unavailable. Keep it beside `game.py`. On other operating systems, Tkinter is required.

## Controls

- **Left/Right**, **A/D**, or **mouse**: move the basket.
- **F11**: toggle full screen. The game scales to fit the display while preserving its proportions.
- **Esc**: leave full screen; in a window, quit.
- **Space**: pause or continue. Changing to another app also pauses the game.
- **R**: restart from level 1.

## Grow your tree

Each addition is checked individually. If the target is **5**, catch **2+3** or **4+1**, and skip additions with other answers. Missing an addition has no penalty.

Your tree starts with two of six growth steps. A correct catch grows it one step, earns points, and changes the target. A wrong catch shrinks it one step.

A fully grown tree advances you to the next level, with a new background color and a fresh seedling. If the tree shrinks to nothing, you return to the previous level with a half-grown tree. On level 1, a small seedling always remains, so you can keep practicing. There are no lives or game-over screen.

Level 1 uses operands from 1–3, level 2 from 1–4, and level 3 from 1–5. Later levels introduce increasingly large numbers. Falling speed and frequency rise gradually; both have limits so the game remains playable.

## Check the rules

```powershell
python -m unittest -v
```
