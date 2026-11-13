# Number Catch

Move a basket and catch falling additions, such as `6+2` or `7+6`, whose answer equals the target at the top. Each operand is between 1 and 9; targets range from 2 to 18.

## Run

From this folder:

```powershell
python game.py
```

Requires Python 3.9 or newer. On Windows, no additional packages are needed: the game uses native Windows graphics if Tkinter is unavailable. Keep `windows_graphics.py` beside `game.py`. On other operating systems, Tkinter is required.

## Play

- Move with **Left/Right**, **A/D**, or the **mouse**.
- Each addition is checked on its own. For target **8**, catch **6+2** or **5+3**.
- Catch a matching addition to earn points and start a new round with a different target.
- Catching any wrong addition costs one of your three lives; the target stays the same.
- Let unwanted additions fall past the basket; there is no penalty for missing them.
- Falling additions get faster as rounds progress.
- **Space** pauses, **R** restarts, and **Esc** quits.

The game pauses automatically when its window loses focus.

## Check the rules

```powershell
python -m unittest -v
```
