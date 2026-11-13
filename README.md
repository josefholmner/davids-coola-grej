# Number Catch

A small Python game: move a basket and catch falling numbers from 1 to 9 so their sum matches the target at the top.

## Run

Use Python 3.9 or newer with Tkinter (included in standard Python installations on Windows):

```sh
python game.py
```

On Linux, your distribution may require installing `python3-tk`.

## Play

- Move with **Left/Right**, **A/D**, or the **mouse**.
- Catch an exact sum to earn points and start a new round.
- Going over the target costs one of your three lives and resets the current sum.
- Let unwanted numbers fall past the basket; there is no penalty for missing them.
- Falling numbers get faster as rounds progress.
- **Space** pauses, **R** restarts, and **Esc** quits.

The game pauses automatically when its window loses focus.

### On this computer

The default Python installation is missing Tkinter. An installed Python that can run the game is:

```powershell
& 'C:\opt\ros2-algoryx\install\python.exe' .\game.py
```

To use `python game.py` instead, modify your standard Python installation and enable its **Tcl/Tk and IDLE** component.

## Check the rules

```sh
python -m unittest -v
```

Use a Python installation with Tkinter for these tests too.
