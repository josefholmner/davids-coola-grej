# Number Catch

Move a basket and catch falling numbers from 1 to 9 so their sum matches the target at the top.

## Run

From this folder:

```powershell
python game.py
```

Requires Python 3.9 or newer. On Windows, no additional packages are needed: the game uses native Windows graphics if Tkinter is unavailable. Keep `windows_graphics.py` beside `game.py`. On other operating systems, Tkinter is required.

## Play

- Move with **Left/Right**, **A/D**, or the **mouse**.
- Catch an exact sum to earn points and start a new round.
- Going over the target costs one of your three lives and resets the current sum.
- Let unwanted numbers fall past the basket; there is no penalty for missing them.
- Falling numbers get faster as rounds progress.
- **Space** pauses, **R** restarts, and **Esc** quits.

The game pauses automatically when its window loses focus.

## Check the rules

```powershell
python -m unittest -v
```
