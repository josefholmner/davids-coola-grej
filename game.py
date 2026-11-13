"""Catch matching sums and differences to grow a tree and explore harder levels."""
import colorsys
import random
import math
import time
from dataclasses import dataclass
try:
    import tkinter as tk
    NATIVE = False
except ModuleNotFoundError:
    import windows_graphics as tk
    NATIVE = True

TREE_FULL = 6


@dataclass(frozen=True)
class Difficulty:
    level: int

    @property
    def max_operand(self):
        return self.level + 2 if self.level <= 3 else 2 * self.level - 2

    @property
    def min_target(self):
        return max(2, self.max_operand - 1)

    @property
    def subtraction_chance(self):
        return min(0.60, 0.10 + (self.level - 1) * 0.05)

    @property
    def speed(self):
        return min(330, 65 + (self.level - 1) * 14)

    @property
    def interval(self):
        return max(0.65, 1.9 - (self.level - 1) * 0.12)


def level_color(level):
    rgb = colorsys.hsv_to_rgb((0.56 + (level - 1) * 0.13) % 1, 0.55, 0.19)
    return '#' + ''.join(f'{round(channel * 255):02x}' for channel in rgb)


@dataclass
class GameState:
    level: int = 1
    growth: int = 2
    target: int = 0
    score: int = 0
    last_answer: str = ''

    def __post_init__(self):
        if not self.target:
            self.new_target()

    @property
    def difficulty(self):
        return Difficulty(self.level)

    def new_target(self):
        difficulty = self.difficulty
        low, high = difficulty.min_target, difficulty.max_operand * 2
        # Choose a different target without building an ever larger list.
        target = random.randint(low, high)
        if target == self.target:
            target = low if target == high else target + 1
        self.target = target

    def catch(self, left, right, operator='+'):
        if operator not in ('+', '-'):
            raise ValueError('Unsupported operator')
        answer = left + right if operator == '+' else left - right
        symbol = '+' if operator == '+' else '−'
        self.last_answer = f'{left} {symbol} {right} = {answer}'
        if answer == self.target:
            self.score += self.target * 10
            self.growth += 1
            if self.growth >= TREE_FULL:
                self.level += 1
                self.growth = 2
                result = 'level_up'
            else:
                result = 'correct'
            self.new_target()
            return result
        self.growth -= 1
        if self.growth <= 0:
            if self.level > 1:
                self.level -= 1
                self.growth = TREE_FULL // 2
                self.new_target()
                return 'level_down'
            self.growth = 1  # A seedling always remains on the first level.
        return 'wrong'


def make_addition(target, max_operand):
    if random.random() < 0.5:
        left = random.randint(max(1, target - max_operand), min(max_operand, target - 1))
        return left, target - left
    left, right = random.randint(1, max_operand), random.randint(1, max_operand)
    if left + right == target:
        right = right % max_operand + 1
    return left, right


def make_problem(target, difficulty):
    """Subtraction starts rare and becomes more common; answers stay positive."""
    if random.random() >= difficulty.subtraction_chance:
        left, right = make_addition(target, difficulty.max_operand)
        return left, right, '+'
    answer = target
    if random.random() >= 0.5:
        answer = random.randint(difficulty.min_target, difficulty.max_operand * 2)
        if answer == target:
            answer = difficulty.min_target if answer == difficulty.max_operand * 2 else answer + 1
    right = random.randint(1, difficulty.max_operand)
    return answer + right, right, '-'


class LogicalCanvas:
    """Scale Tkinter drawings and mouse input to a fixed game coordinate system."""
    def __init__(self, root, width, height, bg):
        self.width, self.height = width, height
        self.scale, self.ox, self.oy = 1, 0, 0
        self.canvas = tk.Canvas(root, width=width, height=height, bg=bg, highlightthickness=0)
        self.canvas.pack(fill='both', expand=True)
        self.canvas.bind('<Configure>', self.resize)

    def resize(self, event):
        self.scale = min(event.width / self.width, event.height / self.height)
        self.ox = (event.width - self.width * self.scale) / 2
        self.oy = (event.height - self.height * self.scale) / 2

    def bind(self, event, callback):
        self.canvas.bind(event, lambda e: callback(type('Mouse', (), {'x': (e.x - self.ox) / self.scale})()))

    def delete(self, tag):
        self.canvas.delete(tag)

    def __getattr__(self, name):
        if not name.startswith('create_'):
            raise AttributeError(name)
        def create(*coords, **options):
            scaled = [v * self.scale + (self.ox if i % 2 == 0 else self.oy) for i, v in enumerate(coords)]
            options = dict(options)
            if 'font' in options:
                family, size, weight = options['font']
                options['font'] = (family, max(1, round(size * self.scale)), weight)
            if 'width' in options:
                options['width'] *= self.scale
            return getattr(self.canvas, name)(*scaled, **options)
        return create


class NumberCatch:
    WIDTH, HEIGHT = 1000, 700
    PLAY_RIGHT = 720
    BASKET_Y, BASKET_WIDTH = 590, 112
    PANEL, TEXT = '#1d2a40', '#eaf1ff'

    def __init__(self, root):
        self.root = root
        root.title('Number Catch — Growing Tree')
        root.resizable(True, True)
        if NATIVE:
            self.canvas = tk.Canvas(root, width=self.WIDTH, height=self.HEIGHT,
                                    bg='#111c28', highlightthickness=0)
            self.canvas.pack()
        else:
            self.canvas = LogicalCanvas(root, self.WIDTH, self.HEIGHT, '#111c28')
        self.fullscreen = False
        self.keys = set()
        self.mouse_x = None
        root.bind('<KeyPress>', self.key_down)
        root.bind('<KeyRelease>', lambda event: self.keys.discard(event.keysym.lower()))
        self.canvas.bind('<Motion>', self.mouse_move)
        root.bind('<FocusOut>', self.focus_out)
        self.reset()
        self.last_time = time.perf_counter()
        self.tick()

    def reset(self):
        self.state = GameState()
        self.display_growth = float(self.state.growth)
        self.transition_time = 0
        self.animation_time = 0
        self.basket_x = self.PLAY_RIGHT / 2
        self.drops = []
        self.keys.clear()
        self.mouse_x = None
        self.spawn_in = 1.2
        self.paused = False
        self.message = 'Catch matching + and − problems to grow your tree!'
        self.message_time = 5

    def toggle_fullscreen(self):
        self.fullscreen = not self.fullscreen
        self.root.attributes('-fullscreen', self.fullscreen)
        self.keys.clear()
        self.mouse_x = None

    def key_down(self, event):
        key = event.keysym.lower()
        if key in ('left', 'right', 'a', 'd'):
            self.keys.add(key)
            self.mouse_x = None
        elif key == 'f11':
            self.toggle_fullscreen()
        elif key == 'space':
            self.paused = not self.paused
        elif key == 'r':
            self.reset()
        elif key == 'escape':
            if self.fullscreen:
                self.toggle_fullscreen()
            else:
                self.root.destroy()

    def mouse_move(self, event):
        self.mouse_x = event.x

    def focus_out(self, _event):
        self.keys.clear()
        self.paused = True

    def spawn(self):
        difficulty = self.state.difficulty
        left, right, operator = make_problem(self.state.target, difficulty)
        symbol = '+' if operator == '+' else '−'
        label = f'{left}{symbol}{right}'
        half_width = max(44, len(label) * 9 + 12)
        self.drops.append({'x': random.randint(half_width + 8, self.PLAY_RIGHT - half_width - 8),
                           'y': 170.0, 'left': left, 'right': right, 'operator': operator, 'half_width': half_width,
                           'speed': random.uniform(difficulty.speed, difficulty.speed + 15)})

    def update(self, dt):
        self.animation_time += dt
        self.message_time = max(0, self.message_time - dt)
        if self.transition_time > 0:
            self.transition_time = max(0, self.transition_time - dt)
            return
        self.display_growth += (self.state.growth - self.display_growth) * min(1, dt * 5)
        direction = int(bool(self.keys & {'right', 'd'})) - int(bool(self.keys & {'left', 'a'}))
        self.basket_x = self.mouse_x if self.mouse_x is not None else self.basket_x + direction * 460 * dt
        half = self.BASKET_WIDTH / 2
        self.basket_x = max(half + 8, min(self.PLAY_RIGHT - half - 8, self.basket_x))
        self.spawn_in -= dt
        if self.spawn_in <= 0:
            self.spawn()
            self.spawn_in = self.state.difficulty.interval
        for drop in self.drops[:]:
            previous_y = drop['y']
            drop['y'] += drop['speed'] * dt
            if previous_y + 20 <= self.BASKET_Y <= drop['y'] + 20 and abs(drop['x'] - self.basket_x) <= half:
                result = self.state.catch(drop['left'], drop['right'], drop.get('operator', '+'))
                if result == 'level_up':
                    self.message = f'Fully grown! Welcome to level {self.state.level}.'
                    self.display_growth, self.transition_time = TREE_FULL, 1.6
                elif result == 'level_down':
                    self.message = f'Try level {self.state.level} again. Your tree can regrow!'
                    self.display_growth, self.transition_time = 0, 1.3
                elif result == 'correct':
                    self.message = f'{self.state.last_answer} — your tree grows!'
                else:
                    self.message = f'{self.state.last_answer} — tree shrinks. Target: {self.state.target}'
                self.message_time = 3.5
                self.drops.clear()
                self.spawn_in = 1.0
                break
            elif drop['y'] > self.HEIGHT + 24:
                self.drops.remove(drop)

    def text(self, x, y, text, size=16, color=None, **kwargs):
        self.canvas.create_text(x, y, text=text, fill=color or self.TEXT,
                                font=('Segoe UI', size, 'bold'), **kwargs)

    def draw_tree(self):
        c, s = self.canvas, self.state
        x, ground = 860, 586
        fraction = max(0, min(1, self.display_growth / TREE_FULL))
        c.create_oval(758, ground - 14, 962, ground + 20, fill='#315844', outline='')
        if fraction > 0.01:
            height = 35 + 230 * fraction
            top = ground - height
            trunk_half = 4 + 10 * fraction
            c.create_polygon(x - trunk_half, ground, x + trunk_half, ground,
                             x + 4, top, x - 4, top, fill='#ac7950', outline='')
            for direction in (-1, 1):
                c.create_line(x, top + height * 0.6, x + direction * 49 * fraction,
                              top + height * 0.3, fill='#ac7950', width=3 + 4 * fraction)
            radius = 12 + 55 * fraction
            for dx, dy, shade in [(-0.7, 0.4, '#32986c'), (0.7, 0.4, '#46b77a'),
                                   (0, -0.2, '#63d88a')]:
                cx, cy = x + dx * radius, top + dy * radius
                c.create_oval(cx - radius, cy - radius, cx + radius, cy + radius,
                              fill=shade, outline='')
            if fraction > 0.8:
                for dx, dy in [(-40, 0), (32, -25), (12, 35)]:
                    cy = top + dy
                    c.create_oval(x + dx - 7, cy - 7, x + dx + 7, cy + 7, fill='#ffcd75', outline='')
        if fraction > 0.01:
            self.draw_butterflies(x, top, fraction)
        self.text(x, 203, 'YOUR TREE', 17, '#96e9b8')
        self.text(x, 237, 'Correct: grow   Wrong: shrink', 10, '#c3d7cb')
        for i in range(TREE_FULL):
            c.create_rectangle(778 + i * 28, 625, 799 + i * 28, 636,
                               fill='#70daa3' if self.display_growth >= i + 0.8 else '#304737', outline='')
        self.text(x, 666, 'Grow it fully to level up!', 11, '#c3d7cb')

    def draw_butterflies(self, tree_x, canopy_y, fraction):
        c = self.canvas
        colors = [('#ffd77a', '#ffad66'), ('#d5b0ff', '#a684eb'),
                  ('#ffb1d2', '#e779ae'), ('#97e5ff', '#59bcdf'), ('#fff0a5', '#e9c768')]
        for i in range(1 + int(fraction * 4)):
            phase = self.animation_time * (0.7 + i * 0.08) + i * 2.3
            x = tree_x + math.sin(phase) * (55 + i * 5)
            y = canopy_y + 12 + math.cos(phase * 1.4) * 35 + math.sin(phase * 2.1) * 9
            wing = 3 + abs(math.sin(self.animation_time * 8 + i)) * 10
            upper, lower = colors[i]
            for side in (-1, 1):
                cx = x + side * (wing / 2 + 1)
                c.create_oval(cx - wing / 2, y - 11, cx + wing / 2, y + 2,
                              fill=upper, outline='')
                c.create_oval(cx - wing * 0.4, y, cx + wing * 0.4, y + 9,
                              fill=lower, outline='')
                c.create_line(x, y - 5, x + side * 4, y - 13, fill='#ece5cf', width=1)
            c.create_line(x, y - 6, x, y + 8, fill='#403842', width=3)
            c.create_oval(x - 2, y - 9, x + 2, y - 5, fill='#403842', outline='')

    def draw(self):
        c, s = self.canvas, self.state
        c.delete('all')
        c.create_rectangle(0, 0, self.WIDTH, self.HEIGHT, fill=level_color(s.level), outline='')
        c.create_rectangle(self.PLAY_RIGHT, 145, self.WIDTH, self.HEIGHT, fill='#192e2b', outline='')
        c.create_rectangle(0, 0, self.WIDTH, 145, fill=self.PANEL, outline='')
        self.text(25, 31, 'NUMBER CATCH', 17, anchor='w')
        self.text(25, 68, f'Level {s.level}    •    Score {s.score}', 13, '#b7c9df', anchor='w')
        self.text(360, 25, 'TARGET', 11, '#b7c9df')
        self.text(360, 68, str(s.target), 36, '#78e7c1')
        self.text(360, 119, 'Catch a + or − problem equal to the target', 15)
        self.text(860, 35, 'F11: full screen', 13, '#b7c9df')
        self.text(860, 73, f'Numbers up to {s.difficulty.max_operand * 3}', 12, '#b7c9df')
        for drop in self.drops:
            x, y, half = drop['x'], drop['y'], drop['half_width']
            c.create_rectangle(x - half, y - 20, x + half, y + 20, fill='#30496b', outline='#6d96c7', width=2)
            self.text(x, y - 1, f"{drop['left']}{'−' if drop.get('operator') == '-' else '+'}{drop['right']}", 21)
        x, y, half = self.basket_x, self.BASKET_Y, self.BASKET_WIDTH / 2
        c.create_polygon(x - half, y, x + half, y, x + half - 12, y + 32,
                         x - half + 12, y + 32, fill='#3aab91', outline='#78e7c1', width=3)
        c.create_line(x - half - 3, y, x + half + 3, y, fill='#b8ffe6', width=5)
        self.text(x, y + 15, 'CATCH', 13, '#0b2530')
        if self.message_time > 0:
            self.text(360, 163, self.message, 12, '#ffe1a0', width=680)
        self.text(360, 652, 'Skip unwanted problems. Missing one costs nothing.', 11, '#c3d0e2')
        self.text(360, 682, '← → / A D / Mouse: move   Space: pause   R: restart   Esc: exit', 10, '#c3d0e2')
        self.draw_tree()
        if self.paused:
            c.create_rectangle(110, 260, 610, 440, fill=self.PANEL, outline='#426080', width=2)
            self.text(360, 310, 'PAUSED', 30)
            self.text(360, 365, 'Grow your tree with matching answers.', 15)
            self.text(360, 408, 'Press Space to continue', 14, '#78e7c1')

    def tick(self):
        now = time.perf_counter()
        dt = min(now - self.last_time, 0.05)
        self.last_time = now
        if not self.paused:
            self.update(dt)
        self.draw()
        self.root.after(16, self.tick)


def main():
    root = tk.Tk()
    NumberCatch(root)
    root.mainloop()


if __name__ == '__main__':
    main()
