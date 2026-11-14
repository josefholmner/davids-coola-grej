#![cfg_attr(not(test), windows_subsystem = "windows")]
mod app;
mod audio;
mod game;
mod language;
mod render;
mod vocabulary;
mod win32;
use app::{Action, Application, Screen};
use game::{Content, Drop, Mode, Op, Problem};
use language::Language;
use render::{Renderer, Viewport};
use std::{
    collections::{HashSet, VecDeque},
    time::Instant,
};
use win32::*;
#[derive(Clone, Copy)]
enum Event {
    Tick,
    Key(u32, bool, bool),
    Mouse(i32, i32),
    Click(i32, i32),
    FocusLost,
    FocusGained,
    Resize,
    Dpi(Rect),
}
struct Host {
    events: VecDeque<Event>,
    redraw: bool,
    closed: bool,
}
unsafe extern "system" fn window_proc(
    hwnd: Hwnd,
    message: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    if message == 0x0014 {
        return 1;
    }
    if message == 0x0002 {
        let pointer = GetWindowLongPtrW(hwnd, -21) as *mut Host;
        if !pointer.is_null() {
            (*pointer).closed = true;
        }
        KillTimer(hwnd, 1);
        PostQuitMessage(0);
        return 0;
    }
    if message == 0x0024 {
        let limits = &mut *(lparam as *mut MinMaxInfo);
        let dpi = GetDpiForWindow(hwnd).max(96) as f32 / 96.0;
        limits.min_track = Point {
            x: (700.0 * dpi) as i32,
            y: (490.0 * dpi) as i32,
        };
        return 0;
    }
    let pointer = GetWindowLongPtrW(hwnd, -21) as *mut Host;
    if pointer.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let host = &mut *pointer;
    match message {
        0x000f => {
            let mut paint: PaintStruct = std::mem::zeroed();
            BeginPaint(hwnd, &mut paint);
            EndPaint(hwnd, &paint);
            host.redraw = true;
            0
        }
        0x0113 => {
            host.events.push_back(Event::Tick);
            0
        }
        0x0100 | 0x0101 => {
            host.events.push_back(Event::Key(
                wparam as u32,
                message == 0x0100,
                lparam & (1 << 30) != 0,
            ));
            0
        }
        0x0200 | 0x0201 => {
            let x = lparam as i16 as i32;
            let y = (lparam >> 16) as i16 as i32;
            host.events.push_back(if message == 0x0201 {
                Event::Click(x, y)
            } else {
                Event::Mouse(x, y)
            });
            0
        }
        0x0007 => {
            host.events.push_back(Event::FocusGained);
            0
        }
        0x0008 => {
            host.events.push_back(Event::FocusLost);
            0
        }
        0x0005 => {
            host.events.push_back(Event::Resize);
            0
        }
        0x02e0 => {
            host.events.push_back(Event::Dpi(*(lparam as *const Rect)));
            0
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}
struct Fullscreen {
    active: bool,
    style: isize,
    placement: Placement,
}
impl Fullscreen {
    fn new() -> Self {
        Self {
            active: false,
            style: 0,
            placement: Placement::default(),
        }
    }
    fn set_active(&self, hwnd: Hwnd, active: bool) {
        if self.active {
            unsafe {
                SetWindowPos(
                    hwnd,
                    if active { -1isize } else { -2isize } as Hwnd,
                    0,
                    0,
                    0,
                    0,
                    0x13,
                );
            }
        }
    }
    fn toggle(&mut self, hwnd: Hwnd) -> Result<(), String> {
        unsafe {
            if !self.active {
                self.placement.size = std::mem::size_of::<Placement>() as u32;
                if GetWindowPlacement(hwnd, &mut self.placement) == 0 {
                    return Err(error("Saving the window position"));
                }
                self.style = GetWindowLongPtrW(hwnd, -16);
                let mut monitor = MonitorInfo {
                    size: std::mem::size_of::<MonitorInfo>() as u32,
                    monitor: Rect::default(),
                    work: Rect::default(),
                    flags: 0,
                };
                if GetMonitorInfoW(MonitorFromWindow(hwnd, 2), &mut monitor) == 0 {
                    return Err(error("Reading monitor resolution"));
                }
                let r = monitor.monitor;
                SetWindowLongPtrW(hwnd, -16, 0x90000000);
                SetWindowPos(
                    hwnd,
                    -1isize as Hwnd,
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    0x260,
                );
            } else {
                SetWindowLongPtrW(hwnd, -16, self.style);
                SetWindowPlacement(hwnd, &self.placement);
                SetWindowPos(hwnd, -2isize as Hwnd, 0, 0, 0, 0, 0x233);
            }
            self.active = !self.active;
            Ok(())
        }
    }
}
fn client_rect(hwnd: Hwnd) -> Rect {
    unsafe {
        let mut r = Rect::default();
        GetClientRect(hwnd, &mut r);
        r
    }
}
fn apply_action(
    action: Action,
    app: &mut Application,
    full: &mut Fullscreen,
    hwnd: Hwnd,
    held: &mut HashSet<u32>,
) -> Result<bool, String> {
    match action {
        Action::Quit => return Ok(true),
        Action::Fullscreen => {
            full.toggle(hwnd)?;
            app.game.clear_input();
            held.clear();
        }
        Action::SaveLanguage => {
            app.settings_error = app.language.save().is_err();
            unsafe {
                SetWindowTextW(
                    hwnd,
                    wide(app.language.text("Kunskapsträdet", "The Learning Tree")).as_ptr(),
                );
            }
        }
        Action::None => {}
    }
    Ok(false)
}
fn run_window(smoke: Option<&str>) -> Result<(), String> {
    unsafe {
        SetProcessDpiAwarenessContext(-4isize as Handle);
        let instance = GetModuleHandleW(std::ptr::null());
        let class_name = wide("NumberCatchRustWindow");
        let class = WndClass {
            style: 3,
            proc: Some(window_proc),
            cls_extra: 0,
            wnd_extra: 0,
            instance,
            icon: LoadIconW(instance, 1usize as *const u16),
            cursor: LoadCursorW(NULL, 32512usize as *const u16),
            background: NULL,
            menu: std::ptr::null(),
            name: class_name.as_ptr(),
        };
        if RegisterClassW(&class) == 0 {
            return Err(error("Registering the game window"));
        }
        let dpi = GetDpiForSystem();
        let screen_width = GetSystemMetrics(0);
        let screen_height = GetSystemMetrics(1);
        let width = (screen_width as f32 * 0.74).clamp(700.0, 1400.0) as i32;
        let height = ((width as f32 * 0.7) as i32)
            .min(screen_height - 120)
            .max(490);
        let mut rect = Rect {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        AdjustWindowRectExForDpi(&mut rect, 0x00cf0000, 0, 0, dpi);
        let outer_w = rect.right - rect.left;
        let outer_h = rect.bottom - rect.top;
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide("Kunskapsträdet / The Learning Tree").as_ptr(),
            0x00cf0000,
            (screen_width - outer_w) / 2,
            (screen_height - outer_h) / 2,
            outer_w,
            outer_h,
            NULL,
            NULL,
            instance,
            NULL,
        );
        if hwnd.is_null() {
            return Err(error("Opening the game"));
        }
        let mut host = Box::new(Host {
            events: VecDeque::new(),
            redraw: true,
            closed: false,
        });
        SetWindowLongPtrW(hwnd, -21, (&mut *host as *mut Host) as isize);
        let audio = audio::Audio::new();
        let mut renderer = Renderer::new()?;
        let mut app = Application::new(if smoke.is_some() {
            Language::Swedish
        } else {
            Language::load()
        });
        SetWindowTextW(
            hwnd,
            wide(app.language.text("Kunskapsträdet", "The Learning Tree")).as_ptr(),
        );
        let mut full = Fullscreen::new();
        let mut held = HashSet::new();
        let mut last_time = Instant::now();
        let mut smoke_ticks = 0;
        let initial = client_rect(hwnd);
        let mut full_size = (0, 0);
        let mut restored_size = (0, 0);
        let mut max_buffer = (0, 0);
        let mut min_gdi = u32::MAX;
        let mut max_gdi = 0;
        let mut modes_verified = false;
        let mut fullscreen_covered = false;
        let mut restored_z_order = false;
        let mut sounds_requested = 0usize;
        let mut sounds_accepted = 0usize;
        let icon_loaded = !class.icon.is_null();
        if smoke.is_none() {
            ShowWindow(hwnd, 5);
        } else {
            ShowWindow(hwnd, 4);
            ShowWindow(hwnd, 4);
        }
        UpdateWindow(hwnd);
        if SetTimer(hwnd, 1, 16, NULL) == 0 {
            DestroyWindow(hwnd);
            return Err(error("Starting the animation clock"));
        }
        let mut msg: Msg = std::mem::zeroed();
        let mut failure = None;
        loop {
            let result = GetMessageW(&mut msg, NULL, 0, 0);
            if result <= 0 {
                if result < 0 {
                    failure = Some(error("Reading window events"));
                }
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
            if host.closed {
                break;
            }
            let events: Vec<Event> = host.events.drain(..).collect();
            let mut quit = false;
            for event in events {
                match event {
                    Event::Tick => {
                        let now = Instant::now();
                        let dt = (now - last_time).as_secs_f32().min(0.05);
                        last_time = now;
                        app.game.left = held.contains(&37) || held.contains(&65);
                        app.game.right = held.contains(&39) || held.contains(&68);
                        app.update(dt);
                        if let Some(kind) = app.game.pending_sound.take() {
                            sounds_requested += 1;
                            sounds_accepted += audio.play(kind) as usize;
                        }
                        host.redraw = true;
                        if let Some(report) = smoke {
                            smoke_ticks += 1;
                            if smoke_ticks == 2 {
                                app.key(49);
                            }
                            if smoke_ticks == 8 {
                                full.toggle(hwnd)?;
                                let r = client_rect(hwnd);
                                full_size = (r.right, r.bottom);
                            }
                            if smoke_ticks == 10 {
                                let mut rect = Rect::default();
                                GetWindowRect(hwnd, &mut rect);
                                fullscreen_covered = GetWindowLongPtrW(hwnd, -20) & 8 != 0
                                    && WindowFromPoint(Point {
                                        x: (rect.left + rect.right) / 2,
                                        y: rect.bottom - 2,
                                    }) == hwnd;
                            }
                            if smoke_ticks == 20 {
                                full.toggle(hwnd)?;
                                let r = client_rect(hwnd);
                                restored_size = (r.right, r.bottom);
                                restored_z_order = GetWindowLongPtrW(hwnd, -20) & 8 == 0;
                            }
                            if smoke_ticks == 25 {
                                app.menu();
                                app.key(83);
                                app.key(69);
                            }
                            if smoke_ticks == 28 {
                                app.key(27);
                                app.key(50);
                            }
                            if smoke_ticks == 35 {
                                let word = app.game.rules.target_word;
                                let result = app.game.rules.catch_word(word, &mut app.game.rng);
                                modes_verified = app.game.rules.language == Language::English
                                    && result == game::Outcome::Correct
                                    && app.game.rules.growth == 3;
                            }
                            if smoke_ticks == 40 {
                                app.start(Mode::Math);
                                app.game.rules.level = 3;
                                app.game.rules.new_target(&mut app.game.rng);
                            }
                            if smoke_ticks == 45 || smoke_ticks == 49 {
                                app.game.paused = false;
                                let correct = smoke_ticks == 45;
                                let mut problem = game::make_problem(
                                    app.game.rules.target,
                                    app.game.rules.difficulty(),
                                    &mut app.game.rng,
                                );
                                while (problem.answer() == app.game.rules.target) != correct {
                                    problem = game::make_problem(
                                        app.game.rules.target,
                                        app.game.rules.difficulty(),
                                        &mut app.game.rng,
                                    );
                                }
                                app.game.drops.clear();
                                app.game.drops.push(Drop {
                                    x: app.game.basket_x,
                                    y: game::BASKET_Y - 22.0,
                                    half_width: 48.0,
                                    speed: 100.0,
                                    content: Content::Math(problem),
                                });
                            }
                            if smoke_ticks > 55 {
                                let n = GetGuiResources(GetCurrentProcess(), 0);
                                min_gdi = min_gdi.min(n);
                                max_gdi = max_gdi.max(n);
                            }
                            if smoke_ticks == 90 {
                                let aware = AreDpiAwarenessContextsEqual(
                                    GetThreadDpiAwarenessContext(),
                                    -4isize as Handle,
                                ) != 0;
                                let passed = aware
                                    && fullscreen_covered
                                    && restored_z_order
                                    && icon_loaded
                                    && sounds_requested == 2
                                    && modes_verified
                                    && full_size.0 > 0
                                    && full_size.1 > 0
                                    && restored_size == (initial.right, initial.bottom)
                                    && max_buffer == full_size
                                    && max_gdi - min_gdi < 4;
                                let text=format!("{{\n  \"passed\": {passed},\n  \"menu_settings_and_both_modes\": {modes_verified},\n  \"per_monitor_v2\": {aware},\n  \"window\": [{}, {}],\n  \"fullscreen\": [{}, {}],\n  \"restored\": [{}, {}],\n  \"largest_native_buffer\": [{}, {}],\n  \"gdi_handles_min\": {min_gdi},\n  \"gdi_handles_max\": {max_gdi},\n  \"frames\": {smoke_ticks}\n}}\n",initial.right,initial.bottom,full_size.0,full_size.1,restored_size.0,restored_size.1,max_buffer.0,max_buffer.1);
                                let text = text.replacen("{", &format!("{{\n  \"fullscreen_covers_taskbar\": {fullscreen_covered},\n  \"windowed_z_order_restored\": {restored_z_order},\n  \"icon_loaded\": {icon_loaded},\n  \"feedback_sound_requests\": {sounds_requested},\n  \"audio_requests_accepted\": {sounds_accepted},"), 1);
                                std::fs::write(report, text).map_err(|e| e.to_string())?;
                                if !passed {
                                    failure=Some("The automated Windows rendering check failed. See its report.".into());
                                }
                                quit = true;
                            }
                        }
                    }
                    Event::Key(key, down, repeated) => {
                        if down {
                            held.insert(key);
                        } else {
                            held.remove(&key);
                        }
                        if down && [37, 39, 65, 68].contains(&key) && app.screen == Screen::Playing
                        {
                            app.game.mouse_x = None;
                        }
                        if !down || repeated {
                            continue;
                        }
                        if key == 27 && full.active {
                            full.toggle(hwnd)?;
                            app.game.clear_input();
                            held.clear();
                            host.redraw = true;
                            continue;
                        }
                        let action = app.key(key);
                        quit |= apply_action(action, &mut app, &mut full, hwnd, &mut held)?;
                        host.redraw = true;
                    }
                    Event::Mouse(x, _y) => {
                        if app.screen == Screen::Playing {
                            let r = client_rect(hwnd);
                            app.game.mouse_x =
                                Some(Viewport::new(r.right, r.bottom).logical_x(x as f32));
                        }
                    }
                    Event::Click(x, y) => {
                        let r = client_rect(hwnd);
                        let view = Viewport::new(r.right, r.bottom);
                        let action = app.click(view.logical_x(x as f32), view.logical_y(y as f32));
                        quit |= apply_action(action, &mut app, &mut full, hwnd, &mut held)?;
                        host.redraw = true;
                    }
                    Event::FocusGained => {
                        full.set_active(hwnd, true);
                        host.redraw = true;
                    }
                    Event::FocusLost => {
                        full.set_active(hwnd, false);
                        audio.stop();
                        held.clear();
                        app.game.clear_input();
                        if app.screen == Screen::Playing {
                            app.game.paused = true;
                        }
                        host.redraw = true;
                    }
                    Event::Resize => host.redraw = true,
                    Event::Dpi(r) => {
                        if !full.active {
                            SetWindowPos(
                                hwnd,
                                NULL,
                                r.left,
                                r.top,
                                r.right - r.left,
                                r.bottom - r.top,
                                0x214,
                            );
                        }
                        host.redraw = true;
                    }
                }
            }
            if quit {
                DestroyWindow(hwnd);
                continue;
            }
            if host.redraw {
                host.redraw = false;
                let r = client_rect(hwnd);
                if r.right > 0 && r.bottom > 0 {
                    let screen = GetDC(hwnd);
                    let result = renderer
                        .draw(screen, r.right, r.bottom, &app)
                        .and_then(|_| renderer.present(screen));
                    ReleaseDC(hwnd, screen);
                    if let Err(error) = result {
                        failure = Some(error);
                        DestroyWindow(hwnd);
                        continue;
                    }
                    if r.right * r.bottom > max_buffer.0 * max_buffer.1 {
                        max_buffer = (r.right, r.bottom);
                    }
                }
            }
        }
        if let Some(error) = failure {
            Err(error)
        } else {
            Ok(())
        }
    }
}
fn preview(
    width: i32,
    height: i32,
    path: &str,
    kind: &str,
    language: Language,
) -> Result<(), String> {
    if !(320..=7680).contains(&width) || !(240..=4320).contains(&height) {
        return Err("Invalid preview dimensions".into());
    }
    unsafe {
        SetProcessDpiAwarenessContext(-4isize as Handle);
    }
    let mut app = Application::new(language);
    let (kind, feedback_kind) = if let Some(kind) = kind.strip_prefix("correct-") {
        (kind, Some(game::FeedbackKind::Correct))
    } else if let Some(kind) = kind.strip_prefix("wrong-") {
        (kind, Some(game::FeedbackKind::Wrong))
    } else {
        (kind, None)
    };
    match kind {
        "menu" => {}
        "settings" => app.screen = Screen::Settings,
        "reading" => {
            app.start(Mode::Reading);
            app.game.rules.level = 3;
            app.game.rules.target_word = 0;
            let mut indices = vec![0];
            indices.extend(
                vocabulary::pool(3, language)
                    .into_iter()
                    .filter(|id| *id != 0)
                    .take(3),
            );
            for (index, (x, y)) in indices.into_iter().zip([
                (150.0, 330.0),
                (445.0, 365.0),
                (275.0, 450.0),
                (610.0, 510.0),
            ]) {
                app.game.drops.push(Drop {
                    x,
                    y,
                    half_width: 65.0,
                    speed: 60.0,
                    content: Content::Word(index),
                });
            }
        }
        "math" | "subtraction" => {
            app.start(Mode::Math);
            let sub = kind == "subtraction";
            app.game.rules.level = if sub { 3 } else { 1 };
            app.game.rules.target = if sub { 2 } else { 5 };
            for (x, y, a, b) in [
                (
                    150.0,
                    260.0,
                    if sub { 3 } else { 2 },
                    if sub { 1 } else { 3 },
                ),
                (
                    445.0,
                    345.0,
                    if sub { 2 } else { 1 },
                    if sub { 1 } else { 2 },
                ),
                (275.0, 440.0, 3, if sub { 2 } else { 2 }),
                (610.0, 500.0, 3, if sub { 1 } else { 3 }),
            ] {
                app.game.drops.push(Drop {
                    x,
                    y,
                    half_width: 58.0,
                    speed: 60.0,
                    content: Content::Math(Problem {
                        left: a,
                        right: b,
                        op: if sub { Op::Subtract } else { Op::Add },
                    }),
                });
            }
        }
        _ => return Err("Unknown preview screen".into()),
    }
    app.game.rules.growth = 5;
    app.game.display_growth = 5.0;
    app.game.animation = 1.4;
    if let Some(kind) = feedback_kind {
        app.game.feedback = Some(game::Feedback {
            kind,
            x: app.game.basket_x,
            age: 0.18,
        });
    }
    let mut renderer = Renderer::new()?;
    let dc = unsafe { GetDC(NULL) };
    let result = renderer
        .draw(dc, width, height, &app)
        .and_then(|_| renderer.save_png(path));
    unsafe {
        ReleaseDC(NULL, dc);
    }
    result
}
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--verify-assets") if args.len() == 3 => {
            let mut renderer = Renderer::new()?;
            let (count, width, height) = renderer.verify_images()?;
            let report = format!("{{\n  \"illustrations\": {count},\n  \"minimum_width\": {width},\n  \"minimum_height\": {height},\n  \"all_images_decoded\": true\n}}\n");
            std::fs::write(&args[2], report).map_err(|error| error.to_string())
        }

        Some("--render-preview") if (5..=7).contains(&args.len()) => preview(
            args[2].parse().map_err(|_| "Invalid width")?,
            args[3].parse().map_err(|_| "Invalid height")?,
            &args[4],
            args.get(5).map(String::as_str).unwrap_or("math"),
            if args.get(6).map(String::as_str) == Some("en") {
                Language::English
            } else {
                Language::Swedish
            },
        ),
        Some("--smoke-test") if args.len() == 3 => run_window(Some(&args[2])),
        None => run_window(None),
        _ => Err("Unknown arguments. Double-click NumberCatch.exe to play.".into()),
    }
}
fn main() {
    if let Err(error) = run() {
        if std::env::args().len() > 1 {
            let _ = std::fs::write("NumberCatch-error.log", &error);
        } else {
            unsafe {
                MessageBoxW(
                    NULL,
                    wide(&error).as_ptr(),
                    wide("Kunskapsträdet / The Learning Tree").as_ptr(),
                    0x10,
                );
            }
        }
        std::process::exit(1);
    }
}
