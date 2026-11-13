"""Small Windows drawing backend using only Python's standard library."""
import ctypes as C
from ctypes import wintypes as W
from types import SimpleNamespace

user = C.WinDLL('user32', use_last_error=True)
gdi = C.WinDLL('gdi32', use_last_error=True)
kernel = C.WinDLL('kernel32', use_last_error=True)
LRESULT = C.c_ssize_t
WNDPROC = C.WINFUNCTYPE(LRESULT, W.HWND, W.UINT, W.WPARAM, W.LPARAM)


class WNDCLASS(C.Structure):
    _fields_ = [('style', W.UINT), ('proc', WNDPROC), ('cls_extra', C.c_int),
                ('wnd_extra', C.c_int), ('instance', W.HINSTANCE), ('icon', W.HICON),
                ('cursor', W.HANDLE), ('background', W.HBRUSH),
                ('menu', W.LPCWSTR), ('name', W.LPCWSTR)]


class PAINTSTRUCT(C.Structure):
    _fields_ = [('hdc', W.HDC), ('erase', W.BOOL), ('rect', W.RECT),
                ('restore', W.BOOL), ('update', W.BOOL), ('reserved', W.BYTE * 32)]


def api(dll, name, result, *args):
    fn = getattr(dll, name)
    fn.restype, fn.argtypes = result, args
    return fn


api(kernel, 'GetModuleHandleW', W.HMODULE, W.LPCWSTR)
api(user, 'RegisterClassW', W.WORD, C.POINTER(WNDCLASS))
api(user, 'CreateWindowExW', W.HWND, W.DWORD, W.LPCWSTR, W.LPCWSTR, W.DWORD,
    C.c_int, C.c_int, C.c_int, C.c_int, W.HWND, W.HMENU, W.HINSTANCE, W.LPVOID)
api(user, 'DefWindowProcW', LRESULT, W.HWND, W.UINT, W.WPARAM, W.LPARAM)
api(user, 'LoadCursorW', W.HANDLE, W.HINSTANCE, W.LPCWSTR)
api(user, 'BeginPaint', W.HDC, W.HWND, C.POINTER(PAINTSTRUCT))
api(user, 'EndPaint', W.BOOL, W.HWND, C.POINTER(PAINTSTRUCT))
api(user, 'FillRect', C.c_int, W.HDC, C.POINTER(W.RECT), W.HBRUSH)
api(user, 'DrawTextW', C.c_int, W.HDC, W.LPCWSTR, C.c_int, C.POINTER(W.RECT), W.UINT)
api(user, 'SetTimer', C.c_size_t, W.HWND, C.c_size_t, W.UINT, W.LPVOID)
api(user, 'KillTimer', W.BOOL, W.HWND, C.c_size_t)
api(user, 'InvalidateRect', W.BOOL, W.HWND, C.POINTER(W.RECT), W.BOOL)
api(user, 'DestroyWindow', W.BOOL, W.HWND)
api(user, 'ShowWindow', W.BOOL, W.HWND, C.c_int)
api(user, 'SetWindowTextW', W.BOOL, W.HWND, W.LPCWSTR)
api(user, 'AdjustWindowRect', W.BOOL, C.POINTER(W.RECT), W.DWORD, W.BOOL)
api(user, 'SetWindowPos', W.BOOL, W.HWND, W.HWND, C.c_int, C.c_int, C.c_int, C.c_int, W.UINT)
api(user, 'GetMessageW', W.BOOL, C.POINTER(W.MSG), W.HWND, W.UINT, W.UINT)
api(user, 'PeekMessageW', W.BOOL, C.POINTER(W.MSG), W.HWND, W.UINT, W.UINT, W.UINT)
api(user, 'TranslateMessage', W.BOOL, C.POINTER(W.MSG))
api(user, 'DispatchMessageW', LRESULT, C.POINTER(W.MSG))
api(gdi, 'CreateCompatibleDC', W.HDC, W.HDC)
api(gdi, 'CreateCompatibleBitmap', W.HBITMAP, W.HDC, C.c_int, C.c_int)
api(gdi, 'SelectObject', W.HANDLE, W.HDC, W.HANDLE)
api(gdi, 'DeleteObject', W.BOOL, W.HANDLE)
api(gdi, 'DeleteDC', W.BOOL, W.HDC)
api(gdi, 'CreateSolidBrush', W.HBRUSH, W.DWORD)
api(gdi, 'CreatePen', W.HPEN, C.c_int, C.c_int, W.DWORD)
api(gdi, 'Rectangle', W.BOOL, W.HDC, C.c_int, C.c_int, C.c_int, C.c_int)
api(gdi, 'Ellipse', W.BOOL, W.HDC, C.c_int, C.c_int, C.c_int, C.c_int)
api(gdi, 'Polygon', W.BOOL, W.HDC, C.POINTER(W.POINT), C.c_int)
api(gdi, 'MoveToEx', W.BOOL, W.HDC, C.c_int, C.c_int, C.POINTER(W.POINT))
api(gdi, 'LineTo', W.BOOL, W.HDC, C.c_int, C.c_int)
api(gdi, 'SetTextColor', W.DWORD, W.HDC, W.DWORD)
api(gdi, 'SetBkMode', C.c_int, W.HDC, C.c_int)
api(gdi, 'BitBlt', W.BOOL, W.HDC, C.c_int, C.c_int, C.c_int, C.c_int, W.HDC, C.c_int, C.c_int, W.DWORD)
api(gdi, 'CreateFontW', W.HFONT, *([C.c_int] * 5), *([W.DWORD] * 8), W.LPCWSTR)


def color(value):
    value = value.lstrip('#')
    r, g, b = (int(value[i:i + 2], 16) for i in (0, 2, 4))
    return r | (g << 8) | (b << 16)


class Tk:
    def __init__(self):
        self.bindings = {}
        self.canvas = None
        self.callback = None
        self.closed = False
        self.error = None
        self.fonts = {}
        self._proc = WNDPROC(self._dispatch)
        instance = kernel.GetModuleHandleW(None)
        self.class_name = 'NumberCatch_' + str(id(self))
        wc = WNDCLASS()
        wc.proc, wc.instance, wc.name = self._proc, instance, self.class_name
        wc.cursor = user.LoadCursorW(None, C.cast(C.c_void_p(32512), W.LPCWSTR))
        if not user.RegisterClassW(C.byref(wc)):
            raise C.WinError(C.get_last_error())
        self.hwnd = user.CreateWindowExW(0, self.class_name, 'Number Catch', 0x00CA0000,
                                        150, 100, 816, 679, None, None, instance, None)
        if not self.hwnd:
            raise C.WinError(C.get_last_error())

    def title(self, title):
        user.SetWindowTextW(self.hwnd, title)

    def resizable(self, *_):
        pass

    def bind(self, event, callback):
        self.bindings[event] = callback

    def after(self, milliseconds, callback):
        self.callback = callback
        user.SetTimer(self.hwnd, 1, milliseconds, None)
        user.InvalidateRect(self.hwnd, None, False)

    def destroy(self):
        if not self.closed:
            user.DestroyWindow(self.hwnd)

    def update(self):
        msg = W.MSG()
        while user.PeekMessageW(C.byref(msg), None, 0, 0, 1):
            user.TranslateMessage(C.byref(msg))
            user.DispatchMessageW(C.byref(msg))
        if self.error:
            raise self.error

    def mainloop(self):
        user.ShowWindow(self.hwnd, 5)
        msg = W.MSG()
        while not self.closed and user.GetMessageW(C.byref(msg), None, 0, 0) > 0:
            user.TranslateMessage(C.byref(msg))
            user.DispatchMessageW(C.byref(msg))
        if self.error:
            raise self.error

    def _dispatch(self, hwnd, message, wparam, lparam):
        try:
            if message == 0x000F:  # WM_PAINT
                ps = PAINTSTRUCT()
                dc = user.BeginPaint(hwnd, C.byref(ps))
                try:
                    if self.canvas:
                        self.canvas.paint(dc)
                finally:
                    user.EndPaint(hwnd, C.byref(ps))
                return 0
            if message == 0x0014:  # Background is painted in the back buffer.
                return 1
            if message == 0x0113:
                user.KillTimer(hwnd, 1)
                callback, self.callback = self.callback, None
                if callback:
                    callback()
                return 0
            if message in (0x0100, 0x0101):
                key = {37: 'left', 39: 'right', 32: 'space', 27: 'escape'}.get(wparam, chr(wparam).lower())
                # Ignore key repeat for pause/restart, but keep movement responsive.
                if message == 0x0100 and lparam & (1 << 30):
                    return 0
                event = '<KeyPress>' if message == 0x0100 else '<KeyRelease>'
                if event in self.bindings:
                    self.bindings[event](SimpleNamespace(keysym=key))
                return 0
            if message == 0x0200 and self.canvas:
                handler = self.canvas.bindings.get('<Motion>')
                if handler:
                    handler(SimpleNamespace(x=C.c_short(lparam & 65535).value))
                return 0
            if message == 0x0008 and '<FocusOut>' in self.bindings:
                self.bindings['<FocusOut>'](None)
            if message == 0x0002:
                self.closed = True
                user.KillTimer(hwnd, 1)
                for font in self.fonts.values():
                    gdi.DeleteObject(font)
                self.fonts.clear()
                user.PostQuitMessage(0)
                return 0
        except Exception as error:
            self.error = error
            user.DestroyWindow(hwnd)
            return 0
        return user.DefWindowProcW(hwnd, message, wparam, lparam)


class Canvas:
    def __init__(self, root, width, height, bg, **_):
        self.root, self.width, self.height, self.bg = root, width, height, bg
        self.commands, self.bindings = [], {}
        root.canvas = self
        rect = W.RECT(0, 0, width, height)
        user.AdjustWindowRect(C.byref(rect), 0x00CA0000, False)
        user.SetWindowPos(root.hwnd, None, 0, 0, rect.right - rect.left, rect.bottom - rect.top, 6)

    def pack(self):
        pass

    def bind(self, event, callback):
        self.bindings[event] = callback

    def delete(self, _):
        self.commands.clear()

    def create_text(self, *coords, **options):
        self.commands.append(('text', coords, options))

    def create_rectangle(self, *coords, **options):
        self.commands.append(('rectangle', coords, options))

    def create_oval(self, *coords, **options):
        self.commands.append(('oval', coords, options))

    def create_line(self, *coords, **options):
        self.commands.append(('line', coords, options))

    def create_polygon(self, *coords, **options):
        self.commands.append(('polygon', coords, options))

    def paint(self, screen):
        dc = gdi.CreateCompatibleDC(screen)
        bitmap = gdi.CreateCompatibleBitmap(screen, self.width, self.height)
        old_bitmap = gdi.SelectObject(dc, bitmap)
        background = gdi.CreateSolidBrush(color(self.bg))
        rect = W.RECT(0, 0, self.width, self.height)
        user.FillRect(dc, C.byref(rect), background)
        gdi.DeleteObject(background)
        try:
            for kind, coords, options in self.commands:
                if kind == 'text':
                    self.paint_text(dc, coords, options)
                    continue
                fill = options.get('fill', self.bg)
                outline = options.get('outline', fill) or fill
                brush = gdi.CreateSolidBrush(color(fill))
                pen = gdi.CreatePen(1 if options.get('dash') else 0,
                                    int(options.get('width', 1)), color(outline))
                old_brush, old_pen = gdi.SelectObject(dc, brush), gdi.SelectObject(dc, pen)
                points = [int(v) for v in coords]
                if kind == 'rectangle':
                    gdi.Rectangle(dc, *points)
                elif kind == 'oval':
                    gdi.Ellipse(dc, *points)
                elif kind == 'line':
                    gdi.MoveToEx(dc, points[0], points[1], None)
                    gdi.LineTo(dc, points[2], points[3])
                elif kind == 'polygon':
                    vertices = (W.POINT * (len(points) // 2))(*[W.POINT(*points[i:i+2]) for i in range(0, len(points), 2)])
                    gdi.Polygon(dc, vertices, len(vertices))
                gdi.SelectObject(dc, old_brush)
                gdi.SelectObject(dc, old_pen)
                gdi.DeleteObject(brush)
                gdi.DeleteObject(pen)
            gdi.BitBlt(screen, 0, 0, self.width, self.height, dc, 0, 0, 0x00CC0020)
        finally:
            gdi.SelectObject(dc, old_bitmap)
            gdi.DeleteObject(bitmap)
            gdi.DeleteDC(dc)

    def paint_text(self, dc, coords, options):
        family, size, weight = options['font']
        key = (family, size, weight)
        if key not in self.root.fonts:
            self.root.fonts[key] = gdi.CreateFontW(-round(size * 96 / 72), 0, 0, 0, 700,
                                                  0, 0, 0, 1, 0, 0, 5, 0, family)
        previous = gdi.SelectObject(dc, self.root.fonts[key])
        gdi.SetBkMode(dc, 1)
        gdi.SetTextColor(dc, color(options['fill']))
        text = str(options['text'])
        bounds = W.RECT(0, 0, int(options.get('width', 2000)), 0)
        flags = 0x10 if 'width' in options else 0x20
        user.DrawTextW(dc, text, -1, C.byref(bounds), flags | 0x400)
        width, height = bounds.right, bounds.bottom
        x, y = map(int, coords)
        anchor = options.get('anchor', 'center')
        left = x if anchor == 'w' else x - width if anchor == 'e' else x - width // 2
        bounds = W.RECT(left, y - height // 2, left + width, y - height // 2 + height)
        user.DrawTextW(dc, text, -1, C.byref(bounds), flags | 1)
        gdi.SelectObject(dc, previous)
