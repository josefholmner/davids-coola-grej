# Number Catch — Growing Tree

Catch falling additions and subtractions whose answers equal the target. Every correct catch grows a tree beside the play area; wrong catches shrink it.

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

Each problem is checked individually. If the target is **5**, catch **2+3**, **4+1**, or **7−2**, and skip problems with other answers. Missing a problem has no penalty. Subtractions always have positive answers.

Your tree starts with two of six growth steps. A correct catch grows it one step, earns points, and changes the target. A wrong catch shrinks it one step.

A fully grown tree advances you to the next level, with a new background color and a fresh seedling. If the tree shrinks to nothing, you return to the previous level with a half-grown tree. On level 1, a small seedling always remains, so you can keep practicing. There are no lives or game-over screen.

Addition operands start at 1–3 in level 1, 1–4 in level 2, and 1–5 in level 3. Subtraction problems use larger starting numbers so their answers can match the same targets. Later levels introduce increasingly large numbers.

Subtractions make up 10% of falling problems on level 1, increasing by 5 percentage points per level up to 60%. Colorful butterflies flutter around the tree, with more appearing as it grows. Falling speed and frequency rise gradually; both have limits so the game remains playable.

## Check the rules

```powershell
python -m unittest -v
```
