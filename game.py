"""Number Catch: move the basket and catch numbers that add up to the target."""
import random
import time
import tkinter as tk
from dataclasses import dataclass, field


@dataclass
class GameState:
    target: int = 0
    total: int = 0
    lives: int = 3
    score: int = 0
    round: int = 1
    caught: list = field(default_factory=list)

    def __post_init__(self):
        if not self.target:
            self.target = random.randint(10, 25)

    def catch(self, value):
        """Apply one catch and return 'caught', 'complete', or 'over'."""
        self.total += value
        self.caught.append(value)
        if self.total > self.target:
            self.lives -= 1
            self.total = 0
            self.caught.clear()
            return "over"
        if self.total == self.target:
            self.score += self.target * 10
            self.round += 1
            self.target = random.randint(10, min(40, 24 + self.round))
            self.total = 0
            self.caught.clear()
            return "complete"
        return "caught"


class NumberCatch:
    WIDTH, HEIGHT = 800, 640
    BASKET_Y, BASKET_WIDTH = 550, 112
    BG, PANEL, TEXT = "#101827", "#1d2a40", "#eaf1ff"

    def __init__(self, root):
        self.root = root
        root.title("Number Catch")
        root.resizable(False, False)
        self.canvas = tk.Canvas(root, width=self.WIDTH, height=self.HEIGHT,
                                bg=self.BG, highlightthickness=0)
        self.canvas.pack()
        self.keys = set()
        self.mouse_x = None
        root.bind("<KeyPress>", self.key_down)
        root.bind("<KeyRelease>", lambda event: self.keys.discard(event.keysym.lower()))
        self.canvas.bind("<Motion>", self.mouse_move)
        root.bind("<FocusOut>", self.focus_out)
        self.reset()
        self.last_time = time.perf_counter()
        self.tick()

    def reset(self):
        self.state = GameState()
        self.basket_x = self.WIDTH / 2
        self.drops = []
        self.keys.clear()
        self.mouse_x = None
        self.spawn_in = 0.4
        self.paused = False
        self.message = "Catch numbers to make the target. Let unwanted numbers fall!"
        self.message_time = 5.0

    def key_down(self, event):
        key = event.keysym.lower()
        if key in ("left", "right", "a", "d"):
            self.keys.add(key)
            self.mouse_x = None
        elif key == "space":
            self.paused = not self.paused
        elif key == "r":
            self.reset()
        elif key == "escape":
            self.root.destroy()

    def mouse_move(self, event):
        self.mouse_x = event.x

    def focus_out(self, _event):
        self.keys.clear()
        self.paused = True

    def spawn(self):
        # Sometimes provide an exact remainder, so every round stays achievable.
        remaining = self.state.target - self.state.total
        value = remaining if remaining <= 9 and random.random() < 0.3 else random.randint(1, 9)
        self.drops.append({"x": random.randint(30, self.WIDTH - 30), "y": 150.0,
                           "value": value, "speed": random.uniform(95, 135) + min(90, (self.state.round - 1) * 7)})

    def update(self, dt):
        direction = int(bool(self.keys & {"right", "d"})) - int(bool(self.keys & {"left", "a"}))
        if self.mouse_x is not None:
            self.basket_x = self.mouse_x
        else:
            self.basket_x += direction * 460 * dt
        half = self.BASKET_WIDTH / 2
        self.basket_x = max(half + 8, min(self.WIDTH - half - 8, self.basket_x))
        self.spawn_in -= dt
        if self.spawn_in <= 0:
            self.spawn()
            self.spawn_in = max(0.45, 1.0 - self.state.round * 0.035)
        self.message_time = max(0, self.message_time - dt)
        for drop in self.drops[:]:
            previous_y = drop["y"]
            drop["y"] += drop["speed"] * dt
            # Test the bottom edge crossing the basket rim, not the whole basket area.
            if previous_y + 20 <= self.BASKET_Y <= drop["y"] + 20 and abs(drop["x"] - self.basket_x) <= half:
                self.drops.remove(drop)
                result = self.state.catch(drop["value"])
                if result == "complete":
                    self.message = "Perfect sum! A new target is ready."
                    self.message_time = 2.5
                    self.drops.clear()
                    self.spawn_in = 0.7
                    break
                if result == "over":
                    self.message = "Too much! One life lost. Try this target again."
                    self.message_time = 3
                    self.drops.clear()
                    self.spawn_in = 0.9
                    break
            elif drop["y"] > self.HEIGHT + 24:
                self.drops.remove(drop)

    def text(self, x, y, text, size=16, color=None, **kwargs):
        self.canvas.create_text(x, y, text=text, fill=color or self.TEXT,
                                font=("Segoe UI", size, "bold"), **kwargs)

    def draw(self):
        c, s = self.canvas, self.state
        c.delete("all")
        c.create_rectangle(0, 0, self.WIDTH, 140, fill=self.PANEL, outline="")
        self.text(28, 28, "NUMBER CATCH", 16, anchor="w")
        self.text(28, 61, f"Score  {s.score}    •    Round  {s.round}", 12, "#9aabc5", anchor="w")
        self.text(765, 30, f"Lives  {s.lives} / 3", 14, "#ffb8bd", anchor="e")
        self.text(400, 25, "TARGET", 11, "#9aabc5")
        self.text(400, 61, str(s.target), 34, "#78e7c1")
        self.text(400, 108, f"Your sum: {s.total}     •     Still need: {s.target - s.total}", 15)
        for x in range(40, self.WIDTH, 80):
            c.create_line(x, 148, x, self.BASKET_Y - 10, fill="#152135", dash=(2, 10))
        for drop in self.drops:
            x, y = drop["x"], drop["y"]
            c.create_oval(x - 20, y - 20, x + 20, y + 20, fill="#30496b", outline="#6d96c7", width=2)
            self.text(x, y - 1, str(drop["value"]), 21)
        x, y, half = self.basket_x, self.BASKET_Y, self.BASKET_WIDTH / 2
        c.create_polygon(x - half, y, x + half, y, x + half - 12, y + 32,
                         x - half + 12, y + 32, fill="#3aab91", outline="#78e7c1", width=3)
        c.create_line(x - half - 3, y, x + half + 3, y, fill="#b8ffe6", width=5)
        self.text(x, y + 15, str(s.total), 13, "#0b2530")
        if self.message_time > 0:
            self.text(400, 169, self.message, 12, "#ffe1a0")
        equation = " + ".join(map(str, s.caught)) or "Catch your first number"
        self.text(400, 605, equation, 12, "#9aabc5", width=740)
        self.text(400, 628, "← → / A D / Mouse: move     Space: pause     R: restart     Esc: quit", 10, "#9aabc5")
        if self.paused or s.lives <= 0:
            c.create_rectangle(145, 235, 655, 425, fill=self.PANEL, outline="#426080", width=2)
            self.text(400, 285, "GAME OVER" if s.lives <= 0 else "PAUSED", 30)
            self.text(400, 339, f"Final score: {s.score}" if s.lives <= 0 else "Catch only the numbers you need.", 16)
            self.text(400, 385, "Press R to play again" if s.lives <= 0 else "Press Space to continue", 14, "#78e7c1")

    def tick(self):
        now = time.perf_counter()
        dt = min(now - self.last_time, 0.05)
        self.last_time = now
        if not self.paused and self.state.lives > 0:
            self.update(dt)
        self.draw()
        self.root.after(16, self.tick)


def main():
    root = tk.Tk()
    NumberCatch(root)
    root.mainloop()


if __name__ == "__main__":
    main()
