//! Minimal, typed bindings to Windows APIs. No third-party runtime is required.
#![allow(non_snake_case, dead_code)]
use std::ffi::c_void;
pub type Handle = *mut c_void;
pub type Hwnd = Handle;
pub type Hdc = Handle;
pub type Gp = *mut c_void;
pub const NULL: Handle = std::ptr::null_mut();

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct PointF {
    pub x: f32,
    pub y: f32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[repr(C)]
pub struct WndClass {
    pub style: u32,
    pub proc: Option<unsafe extern "system" fn(Hwnd, u32, usize, isize) -> isize>,
    pub cls_extra: i32,
    pub wnd_extra: i32,
    pub instance: Handle,
    pub icon: Handle,
    pub cursor: Handle,
    pub background: Handle,
    pub menu: *const u16,
    pub name: *const u16,
}
#[repr(C)]
pub struct Msg {
    pub hwnd: Hwnd,
    pub message: u32,
    pub wparam: usize,
    pub lparam: isize,
    pub time: u32,
    pub point: Point,
    pub private: u32,
}
#[repr(C)]
pub struct PaintStruct {
    pub dc: Hdc,
    pub erase: i32,
    pub rect: Rect,
    pub restore: i32,
    pub update: i32,
    pub reserved: [u8; 32],
}
#[repr(C)]
pub struct MonitorInfo {
    pub size: u32,
    pub monitor: Rect,
    pub work: Rect,
    pub flags: u32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Placement {
    pub size: u32,
    pub flags: u32,
    pub show: u32,
    pub min: Point,
    pub max: Point,
    pub normal: Rect,
}
#[repr(C)]
pub struct MinMaxInfo {
    pub reserved: Point,
    pub max_size: Point,
    pub max_position: Point,
    pub min_track: Point,
    pub max_track: Point,
}
#[repr(C)]
pub struct StartupInput {
    pub version: u32,
    pub callback: Handle,
    pub suppress_thread: i32,
    pub suppress_codecs: i32,
}
#[repr(C)]
pub struct Guid {
    pub d1: u32,
    pub d2: u16,
    pub d3: u16,
    pub d4: [u8; 8],
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name: *const u16) -> Handle;
    pub fn GetCurrentProcess() -> Handle;
}
#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassW(class: *const WndClass) -> u16;
    pub fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Handle,
        instance: Handle,
        param: Handle,
    ) -> Hwnd;
    pub fn DefWindowProcW(hwnd: Hwnd, message: u32, wparam: usize, lparam: isize) -> isize;
    pub fn LoadCursorW(instance: Handle, name: *const u16) -> Handle;
    pub fn ShowWindow(hwnd: Hwnd, command: i32) -> i32;
    pub fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> i32;
    pub fn UpdateWindow(hwnd: Hwnd) -> i32;
    pub fn DestroyWindow(hwnd: Hwnd) -> i32;
    pub fn PostQuitMessage(code: i32);
    pub fn GetMessageW(msg: *mut Msg, hwnd: Hwnd, min: u32, max: u32) -> i32;
    pub fn TranslateMessage(msg: *const Msg) -> i32;
    pub fn DispatchMessageW(msg: *const Msg) -> isize;
    pub fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    pub fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    pub fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
    pub fn GetWindowRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
    pub fn AdjustWindowRectExForDpi(
        rect: *mut Rect,
        style: u32,
        menu: i32,
        ex: u32,
        dpi: u32,
    ) -> i32;
    pub fn SetWindowPos(
        hwnd: Hwnd,
        after: Hwnd,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> i32;
    pub fn GetWindowPlacement(hwnd: Hwnd, placement: *mut Placement) -> i32;
    pub fn SetWindowPlacement(hwnd: Hwnd, placement: *const Placement) -> i32;
    pub fn MonitorFromWindow(hwnd: Hwnd, flags: u32) -> Handle;
    pub fn GetMonitorInfoW(monitor: Handle, info: *mut MonitorInfo) -> i32;
    pub fn BeginPaint(hwnd: Hwnd, paint: *mut PaintStruct) -> Hdc;
    pub fn EndPaint(hwnd: Hwnd, paint: *const PaintStruct) -> i32;
    pub fn SetTimer(hwnd: Hwnd, id: usize, period: u32, callback: Handle) -> usize;
    pub fn KillTimer(hwnd: Hwnd, id: usize) -> i32;
    pub fn GetDC(hwnd: Hwnd) -> Hdc;
    pub fn ReleaseDC(hwnd: Hwnd, dc: Hdc) -> i32;
    pub fn GetSystemMetrics(index: i32) -> i32;
    pub fn GetDpiForSystem() -> u32;
    pub fn GetDpiForWindow(hwnd: Hwnd) -> u32;
    pub fn SetProcessDpiAwarenessContext(context: Handle) -> i32;
    pub fn GetThreadDpiAwarenessContext() -> Handle;
    pub fn AreDpiAwarenessContextsEqual(a: Handle, b: Handle) -> i32;
    pub fn MessageBoxW(hwnd: Hwnd, text: *const u16, title: *const u16, flags: u32) -> i32;
    pub fn GetGuiResources(process: Handle, flags: u32) -> u32;
}
#[link(name = "gdi32")]
extern "system" {
    pub fn CreateCompatibleDC(dc: Hdc) -> Hdc;
    pub fn CreateCompatibleBitmap(dc: Hdc, width: i32, height: i32) -> Handle;
    pub fn SelectObject(dc: Hdc, object: Handle) -> Handle;
    pub fn DeleteObject(object: Handle) -> i32;
    pub fn DeleteDC(dc: Hdc) -> i32;
    pub fn BitBlt(
        to: Hdc,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        from: Hdc,
        sx: i32,
        sy: i32,
        rop: u32,
    ) -> i32;
}
#[link(name = "gdiplus")]
extern "system" {
    pub fn GdiplusStartup(token: *mut usize, input: *const StartupInput, output: Handle) -> i32;
    pub fn GdiplusShutdown(token: usize);
    pub fn GdipCreateFromHDC(dc: Hdc, graphics: *mut Gp) -> i32;
    pub fn GdipDeleteGraphics(graphics: Gp) -> i32;
    pub fn GdipCreateBitmapFromStream(stream: Handle, image: *mut Gp) -> i32;
    pub fn GdipDrawImageRect(
        graphics: Gp,
        image: Gp,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> i32;
    pub fn GdipSetInterpolationMode(graphics: Gp, mode: i32) -> i32;
    pub fn GdipGraphicsClear(graphics: Gp, color: u32) -> i32;
    pub fn GdipResetClip(graphics: Gp) -> i32;
    pub fn GdipSetClipRect(graphics: Gp, x: f32, y: f32, width: f32, height: f32, mode: i32)
        -> i32;
    pub fn GdipSetSmoothingMode(graphics: Gp, mode: i32) -> i32;
    pub fn GdipSetTextRenderingHint(graphics: Gp, hint: i32) -> i32;
    pub fn GdipSetPixelOffsetMode(graphics: Gp, mode: i32) -> i32;
    pub fn GdipCreateMatrix2(
        m11: f32,
        m12: f32,
        m21: f32,
        m22: f32,
        dx: f32,
        dy: f32,
        matrix: *mut Gp,
    ) -> i32;
    pub fn GdipDeleteMatrix(matrix: Gp) -> i32;
    pub fn GdipSetWorldTransform(graphics: Gp, matrix: Gp) -> i32;
    pub fn GdipCreateSolidFill(color: u32, brush: *mut Gp) -> i32;
    pub fn GdipDeleteBrush(brush: Gp) -> i32;
    pub fn GdipFillRectangle(
        graphics: Gp,
        brush: Gp,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> i32;
    pub fn GdipFillEllipse(graphics: Gp, brush: Gp, x: f32, y: f32, width: f32, height: f32)
        -> i32;
    pub fn GdipFillPolygon(
        graphics: Gp,
        brush: Gp,
        points: *const PointF,
        count: i32,
        mode: i32,
    ) -> i32;
    pub fn GdipCreatePen1(color: u32, width: f32, unit: i32, pen: *mut Gp) -> i32;
    pub fn GdipDeletePen(pen: Gp) -> i32;
    pub fn GdipDrawLine(graphics: Gp, pen: Gp, x1: f32, y1: f32, x2: f32, y2: f32) -> i32;
    pub fn GdipDrawRectangle(graphics: Gp, pen: Gp, x: f32, y: f32, width: f32, height: f32)
        -> i32;
    pub fn GdipCreatePath(mode: i32, path: *mut Gp) -> i32;
    pub fn GdipAddPathArc(
        path: Gp,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        start: f32,
        sweep: f32,
    ) -> i32;
    pub fn GdipClosePathFigure(path: Gp) -> i32;
    pub fn GdipFillPath(graphics: Gp, brush: Gp, path: Gp) -> i32;
    pub fn GdipDeletePath(path: Gp) -> i32;
    pub fn GdipCreateFontFamilyFromName(name: *const u16, collection: Gp, family: *mut Gp) -> i32;
    pub fn GdipDeleteFontFamily(family: Gp) -> i32;
    pub fn GdipCreateFont(family: Gp, size: f32, style: i32, unit: i32, font: *mut Gp) -> i32;
    pub fn GdipDeleteFont(font: Gp) -> i32;
    pub fn GdipCreateStringFormat(flags: i32, language: u16, format: *mut Gp) -> i32;
    pub fn GdipDeleteStringFormat(format: Gp) -> i32;
    pub fn GdipSetStringFormatAlign(format: Gp, align: i32) -> i32;
    pub fn GdipSetStringFormatLineAlign(format: Gp, align: i32) -> i32;
    pub fn GdipDrawString(
        graphics: Gp,
        text: *const u16,
        length: i32,
        font: Gp,
        rect: *const RectF,
        format: Gp,
        brush: Gp,
    ) -> i32;
    pub fn GdipFlush(graphics: Gp, intention: i32) -> i32;
    pub fn GdipCreateBitmapFromHBITMAP(bitmap: Handle, palette: Handle, image: *mut Gp) -> i32;
    pub fn GdipSaveImageToFile(
        image: Gp,
        path: *const u16,
        encoder: *const Guid,
        params: Handle,
    ) -> i32;
    pub fn GdipDisposeImage(image: Gp) -> i32;
    pub fn GdipGetImageWidth(image: Gp, width: *mut u32) -> i32;
    pub fn GdipGetImageHeight(image: Gp, height: *mut u32) -> i32;
}

pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
pub fn error(context: &str) -> String {
    format!("{context}: {}", std::io::Error::last_os_error())
}
pub fn status(value: i32, context: &str) -> Result<(), String> {
    if value == 0 {
        Ok(())
    } else {
        Err(format!("{context} (graphics status {value})"))
    }
}

#[link(name = "shlwapi")]
extern "system" {
    pub fn SHCreateMemStream(bytes: *const u8, length: u32) -> Handle;
}
#[repr(C)]
struct StreamVtable {
    query: Handle,
    add_ref: unsafe extern "system" fn(Handle) -> u32,
    release: unsafe extern "system" fn(Handle) -> u32,
}
pub unsafe fn release_stream(stream: Handle) {
    if !stream.is_null() {
        let vtable = *(stream as *const *const StreamVtable);
        ((*vtable).release)(stream);
    }
}

#[link(name = "winmm")]
extern "system" {
    pub fn PlaySoundW(sound: *const u16, module: Handle, flags: u32) -> i32;
}
#[link(name = "user32")]
extern "system" {
    pub fn LoadIconW(instance: Handle, name: *const u16) -> Handle;
    pub fn WindowFromPoint(point: Point) -> Hwnd;
}
