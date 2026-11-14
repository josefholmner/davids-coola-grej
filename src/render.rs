use crate::app::{Application, Screen};
use crate::game::{
    FeedbackKind, Game, Mode, BASKET_HALF, BASKET_Y, FULL, HEIGHT, PLAY_RIGHT, WIDTH,
};
use crate::language::Language;
use crate::vocabulary::{IMAGES, WORDS};
use crate::win32::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub scale: f32,
    pub x: f32,
    pub y: f32,
}
impl Viewport {
    pub fn new(width: i32, height: i32) -> Self {
        let scale = (width.max(1) as f32 / WIDTH).min(height.max(1) as f32 / HEIGHT);
        Self {
            scale,
            x: (width as f32 - WIDTH * scale) / 2.0,
            y: (height as f32 - HEIGHT * scale) / 2.0,
        }
    }
    pub fn logical_x(self, x: f32) -> f32 {
        (x - self.x) / self.scale
    }
    pub fn logical_y(self, y: f32) -> f32 {
        (y - self.y) / self.scale
    }
}

struct Picture {
    image: Gp,
    stream: Handle,
}
impl Drop for Picture {
    fn drop(&mut self) {
        unsafe {
            GdipDisposeImage(self.image);
            release_stream(self.stream);
        }
    }
}

pub struct Renderer {
    token: usize,
    dc: Hdc,
    bitmap: Handle,
    original: Handle,
    graphics: Gp,
    family: Gp,
    formats: [Gp; 3],
    fonts: HashMap<u32, Gp>,
    pictures: HashMap<usize, Picture>,
    pub width: i32,
    pub height: i32,
}
impl Renderer {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let input = StartupInput {
                version: 1,
                callback: NULL,
                suppress_thread: 0,
                suppress_codecs: 0,
            };
            let mut token = 0;
            status(
                GdiplusStartup(&mut token, &input, NULL),
                "Starting Windows graphics",
            )?;
            let mut r = Self {
                token,
                dc: NULL,
                bitmap: NULL,
                original: NULL,
                graphics: NULL,
                family: NULL,
                formats: [NULL; 3],
                fonts: HashMap::new(),
                pictures: HashMap::new(),
                width: 0,
                height: 0,
            };
            status(
                GdipCreateFontFamilyFromName(wide("Segoe UI").as_ptr(), NULL, &mut r.family),
                "Loading the font",
            )?;
            for align in 0..3 {
                status(
                    GdipCreateStringFormat(0x1000, 0, &mut r.formats[align]),
                    "Creating text layout",
                )?;
                GdipSetStringFormatAlign(r.formats[align], align as i32);
                GdipSetStringFormatLineAlign(r.formats[align], 1);
            }
            Ok(r)
        }
    }
    unsafe fn clear_buffer(&mut self) {
        if !self.graphics.is_null() {
            GdipDeleteGraphics(self.graphics);
            self.graphics = NULL;
        }
        if !self.dc.is_null() {
            SelectObject(self.dc, self.original);
            DeleteObject(self.bitmap);
            DeleteDC(self.dc);
            self.dc = NULL;
            self.bitmap = NULL;
            self.original = NULL;
        }
        self.width = 0;
        self.height = 0;
    }
    fn resize(&mut self, reference: Hdc, width: i32, height: i32) -> Result<(), String> {
        if self.width == width && self.height == height {
            return Ok(());
        }
        unsafe {
            self.clear_buffer();
            self.dc = CreateCompatibleDC(reference);
            if self.dc.is_null() {
                return Err(error("Creating a frame buffer"));
            }
            self.bitmap = CreateCompatibleBitmap(reference, width, height);
            if self.bitmap.is_null() {
                self.clear_buffer();
                return Err(error("Allocating a full-resolution frame"));
            }
            self.original = SelectObject(self.dc, self.bitmap);
            status(
                GdipCreateFromHDC(self.dc, &mut self.graphics),
                "Creating the vector renderer",
            )?;
            status(
                GdipSetSmoothingMode(self.graphics, 4),
                "Enabling antialiasing",
            )?;
            GdipSetTextRenderingHint(self.graphics, 3);
            GdipSetPixelOffsetMode(self.graphics, 2);
            GdipSetInterpolationMode(self.graphics, 7);
            self.width = width;
            self.height = height;
            Ok(())
        }
    }
    fn picture(&mut self, index: usize) -> Result<Option<Gp>, String> {
        if let Some(picture) = self.pictures.get(&index) {
            return Ok(Some(picture.image));
        }
        let bytes = IMAGES[index];
        if bytes.is_empty() {
            return Ok(None);
        }
        unsafe {
            let stream = SHCreateMemStream(bytes.as_ptr(), bytes.len() as u32);
            if stream.is_null() {
                return Err(error("Loading the vocabulary picture"));
            }
            let mut image = NULL;
            if let Err(error) = status(
                GdipCreateBitmapFromStream(stream, &mut image),
                "Decoding the vocabulary picture",
            ) {
                release_stream(stream);
                return Err(error);
            }
            if self.pictures.len() >= 8 {
                if let Some(key) = self.pictures.keys().next().copied() {
                    self.pictures.remove(&key);
                }
            }
            self.pictures.insert(index, Picture { image, stream });
            Ok(Some(image))
        }
    }
    pub fn draw(
        &mut self,
        reference: Hdc,
        width: i32,
        height: i32,
        app: &Application,
    ) -> Result<(), String> {
        if width <= 0 || height <= 0 {
            return Ok(());
        }
        self.resize(reference, width, height)?;
        let view = Viewport::new(width, height);
        let picture = if app.screen == Screen::Playing && app.game.rules.mode == Mode::Reading {
            self.picture(app.game.rules.target_word)?
        } else {
            None
        };
        unsafe {
            GdipResetClip(self.graphics);
            GdipGraphicsClear(self.graphics, 0xff09121b);
            let mut matrix = NULL;
            status(
                GdipCreateMatrix2(
                    view.scale,
                    0.0,
                    0.0,
                    view.scale,
                    view.x,
                    view.y,
                    &mut matrix,
                ),
                "Scaling vector coordinates",
            )?;
            GdipSetWorldTransform(self.graphics, matrix);
            GdipDeleteMatrix(matrix);
            GdipSetClipRect(self.graphics, 0.0, 0.0, WIDTH, HEIGHT, 0);
            let mut painter = Painter {
                g: self.graphics,
                family: self.family,
                formats: self.formats,
                fonts: &mut self.fonts,
            };
            match app.screen {
                Screen::Playing => scene(&mut painter, &app.game, picture),
                Screen::Menu => menu(&mut painter, app.language),
                Screen::Settings => settings(&mut painter, app.language, app.settings_error),
            };
            status(GdipFlush(self.graphics, 1), "Finishing the frame")?;
        }
        Ok(())
    }
    pub fn present(&self, screen: Hdc) -> Result<(), String> {
        unsafe {
            if BitBlt(
                screen,
                0,
                0,
                self.width,
                self.height,
                self.dc,
                0,
                0,
                0x00cc0020,
            ) == 0
            {
                return Err(error("Displaying the frame"));
            }
        }
        Ok(())
    }
    pub fn verify_images(&mut self) -> Result<(usize, u32, u32), String> {
        let mut minimum_width = u32::MAX;
        let mut minimum_height = u32::MAX;
        for index in 0..IMAGES.len() {
            let image = self
                .picture(index)?
                .ok_or_else(|| format!("Missing illustration for {}", WORDS[index].english))?;
            let (mut width, mut height) = (0, 0);
            unsafe {
                status(GdipGetImageWidth(image, &mut width), "Reading image width")?;
                status(
                    GdipGetImageHeight(image, &mut height),
                    "Reading image height",
                )?;
            }
            if width < 512 || height < 512 {
                return Err(format!(
                    "Illustration is too small: {} ({}x{})",
                    WORDS[index].english, width, height
                ));
            }
            minimum_width = minimum_width.min(width);
            minimum_height = minimum_height.min(height);
        }
        Ok((IMAGES.len(), minimum_width, minimum_height))
    }
    pub fn save_png(&self, path: &str) -> Result<(), String> {
        unsafe {
            let mut image = NULL;
            status(
                GdipCreateBitmapFromHBITMAP(self.bitmap, NULL, &mut image),
                "Preparing the preview image",
            )?;
            let encoder = Guid {
                d1: 0x557cf406,
                d2: 0x1a04,
                d3: 0x11d3,
                d4: [0x9a, 0x73, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e],
            };
            let result = status(
                GdipSaveImageToFile(image, wide(path).as_ptr(), &encoder, NULL),
                "Saving the preview image",
            );
            GdipDisposeImage(image);
            result
        }
    }
}
impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.clear_buffer();
            for font in self.fonts.values() {
                GdipDeleteFont(*font);
            }
            for format in self.formats {
                if !format.is_null() {
                    GdipDeleteStringFormat(format);
                }
            }
            if !self.family.is_null() {
                GdipDeleteFontFamily(self.family);
            }
            self.pictures.clear();
            GdiplusShutdown(self.token);
        }
    }
}

struct Painter<'a> {
    g: Gp,
    family: Gp,
    formats: [Gp; 3],
    fonts: &'a mut HashMap<u32, Gp>,
}
impl Painter<'_> {
    fn brush(&self, color: u32) -> Gp {
        unsafe {
            let mut brush = NULL;
            GdipCreateSolidFill(0xff000000 | color, &mut brush);
            brush
        }
    }
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        unsafe {
            let b = self.brush(color);
            GdipFillRectangle(self.g, b, x, y, w, h);
            GdipDeleteBrush(b);
        }
    }
    fn ellipse(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        unsafe {
            let b = self.brush(color);
            GdipFillEllipse(self.g, b, x, y, w, h);
            GdipDeleteBrush(b);
        }
    }
    fn polygon(&mut self, points: &[(f32, f32)], color: u32) {
        unsafe {
            let b = self.brush(color);
            let points: Vec<PointF> = points.iter().map(|&(x, y)| PointF { x, y }).collect();
            GdipFillPolygon(self.g, b, points.as_ptr(), points.len() as i32, 0);
            GdipDeleteBrush(b);
        }
    }
    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: u32) {
        unsafe {
            let mut pen = NULL;
            GdipCreatePen1(0xff000000 | color, width, 2, &mut pen);
            GdipDrawLine(self.g, pen, x1, y1, x2, y2);
            GdipDeletePen(pen);
        }
    }
    fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, color: u32) {
        unsafe {
            let mut path = NULL;
            GdipCreatePath(0, &mut path);
            let d = r * 2.0;
            for (ax, ay, start) in [
                (x, y, 180.0),
                (x + w - d, y, 270.0),
                (x + w - d, y + h - d, 0.0),
                (x, y + h - d, 90.0),
            ] {
                GdipAddPathArc(path, ax, ay, d, d, start, 90.0);
            }
            GdipClosePathFigure(path);
            let brush = self.brush(color);
            GdipFillPath(self.g, brush, path);
            GdipDeleteBrush(brush);
            GdipDeletePath(path);
        }
    }
    fn text(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        value: &str,
        size: u32,
        color: u32,
        align: usize,
    ) {
        unsafe {
            let font = *self.fonts.entry(size).or_insert_with(|| {
                let mut font = NULL;
                GdipCreateFont(self.family, size as f32, 1, 2, &mut font);
                font
            });
            let text: Vec<u16> = value.encode_utf16().collect();
            let brush = self.brush(color);
            GdipDrawString(
                self.g,
                text.as_ptr(),
                text.len() as i32,
                font,
                &RectF {
                    x,
                    y,
                    width: w,
                    height: h,
                },
                self.formats[align],
                brush,
            );
            GdipDeleteBrush(brush);
        }
    }
    fn center(&mut self, x: f32, y: f32, w: f32, value: &str, size: u32, color: u32) {
        self.text(x - w / 2.0, y - 24.0, w, 48.0, value, size, color, 1);
    }
}

fn level_color(level: u32) -> u32 {
    let hue = (0.56 + (level - 1) as f32 * 0.13).fract() * 6.0;
    let c = 0.19 * 0.55;
    let x = c * (1.0 - (hue % 2.0 - 1.0).abs());
    let m = 0.19 - c;
    let (r, g, b) = match hue as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (((r + m) * 255.0).round() as u32) << 16
        | (((g + m) * 255.0).round() as u32) << 8
        | ((b + m) * 255.0).round() as u32
}

fn menu(p: &mut Painter, language: Language) {
    p.rect(0.0, 0.0, WIDTH, HEIGHT, 0x102a2b);
    p.ellipse(-140.0, -100.0, 430.0, 430.0, 0x163b34);
    p.ellipse(815.0, 390.0, 420.0, 420.0, 0x1b3b3c);
    p.center(
        500.0,
        85.0,
        940.0,
        language.text("KUNSKAPSTRÄDET", "THE LEARNING TREE"),
        43,
        0x99edbd,
    );
    p.center(
        500.0,
        143.0,
        900.0,
        language.text(
            "Räkna, läs och låt ditt träd växa.",
            "Count, read and watch your tree grow.",
        ),
        21,
        0xc9ddd6,
    );
    p.round_rect(220.0, 220.0, 560.0, 100.0, 22.0, 0x294b62);
    p.center(
        500.0,
        253.0,
        520.0,
        language.text("1   Matematik", "1   Math"),
        30,
        0xf1f6ff,
    );
    p.center(
        500.0,
        292.0,
        520.0,
        language.text(
            "Fånga rätt svar · addition och subtraktion",
            "Catch matching answers · addition and subtraction",
        ),
        17,
        0xc9dcf0,
    );
    p.round_rect(220.0, 345.0, 560.0, 100.0, 22.0, 0x2c604c);
    p.center(
        500.0,
        378.0,
        520.0,
        language.text("2   Läsning", "2   Reading"),
        30,
        0xf1fff5,
    );
    p.center(
        500.0,
        417.0,
        520.0,
        language.text(
            "Bilder och ord · 120 saker att upptäcka",
            "Pictures and words · 120 objects to discover",
        ),
        17,
        0xc7edda,
    );
    p.round_rect(300.0, 480.0, 400.0, 62.0, 17.0, 0x253f43);
    p.center(
        500.0,
        511.0,
        370.0,
        language.text("Inställningar  [S]", "Settings  [S]"),
        23,
        0xd1e8dd,
    );
    p.center(
        500.0,
        595.0,
        300.0,
        language.text("Avsluta  [Esc]", "Quit  [Esc]"),
        19,
        0xb2c6c5,
    );
    p.center(
        500.0,
        669.0,
        900.0,
        language.text(
            "Klicka på ett spel eller tryck 1 / 2.   F11: helskärm",
            "Click a game or press 1 / 2.   F11: full screen",
        ),
        15,
        0x9fbab7,
    );
}

fn settings(p: &mut Painter, language: Language, error: bool) {
    p.rect(0.0, 0.0, WIDTH, HEIGHT, 0x102a2b);
    p.center(
        500.0,
        110.0,
        900.0,
        language.text("INSTÄLLNINGAR", "SETTINGS"),
        42,
        0x99edbd,
    );
    p.center(
        500.0,
        205.0,
        900.0,
        language.text(
            "Välj språk för båda spelen",
            "Choose the language for both games",
        ),
        24,
        0xeaf1ff,
    );
    for (x, chosen, title, key) in [
        (220.0, Language::Swedish, "Svenska", "S / ←"),
        (515.0, Language::English, "English", "E / →"),
    ] {
        let selected = language == chosen;
        p.round_rect(
            x,
            285.0,
            265.0,
            120.0,
            22.0,
            if selected { 0x38775b } else { 0x253f43 },
        );
        p.center(x + 132.5, 326.0, 250.0, title, 30, 0xf1fff5);
        let label = if selected {
            language.text("Valt språk", "Selected")
        } else {
            key
        };
        p.center(x + 132.5, 374.0, 250.0, label, 18, 0xc9ddd6);
    }
    p.center(
        500.0,
        453.0,
        900.0,
        language.text(
            "Svenska är förvalt. Ditt språkval sparas.",
            "Swedish is the default. Your language choice is saved.",
        ),
        17,
        0xbcd7cd,
    );
    p.round_rect(350.0, 500.0, 300.0, 65.0, 18.0, 0x294b62);
    p.center(
        500.0,
        532.0,
        280.0,
        language.text("Tillbaka", "Back"),
        25,
        0xf1f6ff,
    );
    if error {
        p.center(
            500.0,
            608.0,
            920.0,
            language.text(
                "Språket ändrades men kunde inte sparas i den här mappen.",
                "The language changed but could not be saved in this folder.",
            ),
            16,
            0xffd38a,
        );
    }
    p.center(
        500.0,
        666.0,
        900.0,
        language.text(
            "Esc: tillbaka   F11: helskärm",
            "Esc: back   F11: full screen",
        ),
        15,
        0x9fbab7,
    );
}

fn scene(p: &mut Painter, game: &Game, picture: Option<Gp>) {
    let state = &game.rules;
    let d = state.difficulty();
    let language = state.language;
    let reading = state.mode == Mode::Reading;
    let header = if reading { 250.0 } else { 150.0 };
    p.rect(0.0, 0.0, WIDTH, HEIGHT, level_color(state.level));
    p.rect(
        PLAY_RIGHT,
        150.0,
        WIDTH - PLAY_RIGHT,
        HEIGHT - 150.0,
        0x162b28,
    );
    p.rect(0.0, 0.0, WIDTH, header, 0x1b293b);
    let title = if reading {
        language.text("ORDTRÄDET", "WORD TREE")
    } else {
        language.text("MATTETRÄDET", "MATH TREE")
    };
    p.text(25.0, 15.0, 270.0, 40.0, title, 23, 0xeaf1ff, 0);
    p.text(
        25.0,
        62.0,
        270.0,
        30.0,
        &format!(
            "{} {}   •   {} {}",
            language.text("Nivå", "Level"),
            state.level,
            language.text("Poäng", "Score"),
            state.score
        ),
        17,
        0xb7c9df,
        0,
    );
    p.round_rect(25.0, 101.0, 140.0, 31.0, 9.0, 0x294356);
    p.center(
        95.0,
        116.0,
        140.0,
        language.text("Meny  [Esc]", "Menu  [Esc]"),
        14,
        0xc6d9e5,
    );
    if reading {
        p.round_rect(272.0, 10.0, 176.0, 178.0, 20.0, 0xfff8ed);
        if let Some(image) = picture {
            unsafe {
                GdipDrawImageRect(p.g, image, 280.0, 17.0, 160.0, 160.0);
            }
        } else {
            p.center(
                360.0,
                95.0,
                160.0,
                language.text("Bild", "Picture"),
                22,
                0x38554d,
            );
        }
        p.center(
            360.0,
            216.0,
            680.0,
            language.text(
                "Vilket ord hör till bilden?",
                "Which word matches the picture?",
            ),
            21,
            0xeaf1ff,
        );
        let length = WORDS[state.target_word].text(language).chars().count();
        p.text(
            25.0,
            151.0,
            240.0,
            35.0,
            &format!("{} {}", length, language.text("bokstäver", "letters")),
            18,
            0x99edbd,
            0,
        );
    } else {
        p.round_rect(306.0, 14.0, 108.0, 87.0, 20.0, 0x253d49);
        p.center(
            360.0,
            29.0,
            120.0,
            language.text("MÅL", "TARGET"),
            12,
            0x9fc9bd,
        );
        p.center(360.0, 68.0, 160.0, &state.target.to_string(), 48, 0x8af0c8);
        p.center(
            360.0,
            125.0,
            520.0,
            language.text(
                "Fånga uträkningen med rätt svar",
                "Catch the problem with the matching answer",
            ),
            17,
            0xeaf1ff,
        );
    }
    p.center(
        860.0,
        39.0,
        260.0,
        language.text("F11: helskärm", "F11: full screen"),
        17,
        0xb7c9df,
    );
    let detail = if reading {
        language.text("Läs och fånga ordet", "Read and catch the word")
    } else if d.subtraction() {
        language.text("Subtraktion · lugnare tempo", "Subtraction · slower pace")
    } else {
        language.text("Addition", "Addition")
    };
    p.center(860.0, 77.0, 270.0, detail, 15, 0xb7c9df);
    p.line(PLAY_RIGHT, header, PLAY_RIGHT, HEIGHT, 1.0, 0x35504a);
    for x in (48..700).step_by(80) {
        for y in ((header as i32 + 55)..575).step_by(65) {
            p.ellipse(x as f32, y as f32, 2.0, 2.0, 0x32434d);
        }
    }
    for drop in &game.drops {
        let width = drop.half_width * 2.0;
        p.round_rect(
            drop.x - drop.half_width,
            drop.y - 19.0,
            width,
            46.0,
            13.0,
            0x0d1826,
        );
        p.round_rect(
            drop.x - drop.half_width,
            drop.y - 23.0,
            width,
            44.0,
            13.0,
            0x3b5778,
        );
        p.center(
            drop.x,
            drop.y - 1.0,
            width - 4.0,
            &drop.content.label(language),
            27,
            0xf3f7ff,
        );
    }
    let effect = game.feedback.as_ref();
    let shake = effect
        .filter(|f| f.kind == FeedbackKind::Wrong)
        .map_or(0.0, |f| (f.age * 65.0).sin() * 6.0 * (1.0 - f.age / 0.85));
    let bounce = effect
        .filter(|f| f.kind == FeedbackKind::Correct)
        .map_or(0.0, |f| {
            (f.age * 12.0).sin().max(0.0) * 7.0 * (1.0 - f.age / 0.85)
        });
    let x = game.basket_x + shake;
    let y = BASKET_Y - bounce;
    p.ellipse(x - 61.0, y + 29.0, 122.0, 12.0, 0x0d1c24);
    p.polygon(
        &[
            (x - BASKET_HALF, y),
            (x + BASKET_HALF, y),
            (x + BASKET_HALF - 12.0, y + 34.0),
            (x - BASKET_HALF + 12.0, y + 34.0),
        ],
        0x3aab91,
    );
    p.line(
        x - BASKET_HALF - 2.0,
        y,
        x + BASKET_HALF + 2.0,
        y,
        5.0,
        0xb8ffe6,
    );
    p.center(
        x,
        y + 16.0,
        110.0,
        language.text("FÅNGA", "CATCH"),
        16,
        0x0b2530,
    );
    if game.message_time > 0.0 {
        p.center(360.0, header + 20.0, 695.0, &game.message, 15, 0xffe1a0);
    }
    p.center(
        360.0,
        650.0,
        700.0,
        language.text(
            "Låt fel svar falla förbi. Det kostar ingenting.",
            "Skip unwanted answers. Missing one costs nothing.",
        ),
        14,
        0xc3d0e2,
    );
    p.center(
        360.0,
        681.0,
        715.0,
        language.text(
            "Pilar / A D / mus: flytta   Mellanslag: paus   R: omstart   Esc: meny",
            "Arrows / A D / Mouse: move   Space: pause   R: restart   Esc: menu",
        ),
        12,
        0xc3d0e2,
    );
    catch_feedback(p, game);
    tree(p, game);
    if game.paused {
        p.round_rect(104.0, 285.0, 512.0, 196.0, 22.0, 0x0e1c29);
        p.round_rect(110.0, 291.0, 500.0, 184.0, 18.0, 0x23384b);
        p.center(
            360.0,
            337.0,
            450.0,
            language.text("PAUS", "PAUSED"),
            38,
            0xeaf1ff,
        );
        p.center(
            360.0,
            394.0,
            470.0,
            language.text("Ditt träd väntar på dig.", "Your tree is waiting for you."),
            18,
            0xc3d0e2,
        );
        p.center(
            360.0,
            439.0,
            470.0,
            language.text(
                "Tryck mellanslag för att fortsätta",
                "Press Space to continue",
            ),
            18,
            0x78e7c1,
        );
    }
}

fn catch_feedback(p: &mut Painter, game: &Game) {
    let Some(feedback) = &game.feedback else {
        return;
    };
    let t = (feedback.age / 0.85).clamp(0.0, 1.0);
    let correct = feedback.kind == FeedbackKind::Correct;
    let color = if correct { 0x9cf5be } else { 0xffa3a7 };
    let x = feedback.x;
    let y = BASKET_Y - 12.0;
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let radius = 15.0 + t * 43.0;
        let px = x + angle.cos() * radius;
        let py = y + angle.sin() * radius * 0.48 - if correct { t * 35.0 } else { -t * 5.0 };
        let size = (1.0 - t) * 5.0;
        if correct {
            p.line(px - size, py, px + size, py, 2.0, color);
            p.line(px, py - size, px, py + size, 2.0, color);
        } else {
            p.ellipse(px - size / 2.0, py - size / 2.0, size, size, color);
        }
    }
    let center_y = y - 34.0 - t * 24.0;
    if t < 0.65 {
        let gx = x.clamp(80.0, PLAY_RIGHT - 80.0);
        if correct {
            p.line(gx - 8.0, center_y, gx - 2.0, center_y + 6.0, 3.0, color);
            p.line(
                gx - 2.0,
                center_y + 6.0,
                gx + 10.0,
                center_y - 8.0,
                3.0,
                color,
            );
        } else {
            p.line(
                gx - 7.0,
                center_y - 7.0,
                gx + 7.0,
                center_y + 7.0,
                3.0,
                color,
            );
            p.line(
                gx - 7.0,
                center_y + 7.0,
                gx + 7.0,
                center_y - 7.0,
                3.0,
                color,
            );
        }
    }
    let language = game.rules.language;
    let label = if correct {
        language.text("Rätt!", "Correct!")
    } else {
        language.text("Försök igen", "Try again")
    };
    p.center(
        x.clamp(85.0, PLAY_RIGHT - 85.0),
        center_y - 29.0,
        170.0,
        label,
        18,
        color,
    );
}

fn tree(p: &mut Painter, game: &Game) {
    let language = game.rules.language;
    let x = 860.0;
    let ground = 586.0;
    let fraction = (game.display_growth / FULL as f32).clamp(0.0, 1.0);
    p.ellipse(758.0, ground - 12.0, 204.0, 34.0, 0x315844);
    if fraction > 0.01 {
        let height = 35.0 + 230.0 * fraction;
        let top = ground - height;
        let half = 4.0 + 10.0 * fraction;
        p.polygon(
            &[
                (x - half, ground),
                (x + half, ground),
                (x + 4.0, top),
                (x - 4.0, top),
            ],
            0xac7950,
        );
        p.line(
            x - 2.0,
            ground - 8.0,
            x - 2.0,
            top + height * 0.3,
            2.0,
            0xc49465,
        );
        for side in [-1.0, 1.0] {
            p.line(
                x,
                top + height * 0.6,
                x + side * 49.0 * fraction,
                top + height * 0.3,
                3.0 + 4.0 * fraction,
                0xac7950,
            );
        }
        let radius = 12.0 + 55.0 * fraction;
        for (dx, dy, color) in [
            (-0.7, 0.4, 0x32986c),
            (0.7, 0.4, 0x46b77a),
            (0.0, -0.2, 0x63d88a),
        ] {
            p.ellipse(
                x + dx * radius - radius,
                top + dy * radius - radius,
                radius * 2.0,
                radius * 2.0,
                color,
            );
        }
        // Small leaf highlights stay smooth even on a 4K display.
        for (dx, dy) in [(-0.6, -0.1), (0.25, -0.7), (0.8, 0.1)] {
            p.ellipse(
                x + dx * radius - 5.0,
                top + dy * radius - 3.0,
                13.0 * fraction,
                7.0 * fraction,
                0x9aefad,
            );
        }
        if fraction > 0.8 {
            for (dx, dy) in [(-40.0, 0.0), (32.0, -25.0), (12.0, 35.0)] {
                p.ellipse(x + dx - 7.0, top + dy - 7.0, 14.0, 14.0, 0xffcd75);
            }
        }
        butterflies(p, game.animation, x, top, fraction);
    }
    p.center(
        x,
        204.0,
        260.0,
        language.text("DITT TRÄD", "YOUR TREE"),
        22,
        0x96e9b8,
    );
    p.center(
        x,
        238.0,
        270.0,
        language.text(
            "Rätt: växer   Fel: krymper",
            "Correct: grow   Wrong: shrink",
        ),
        13,
        0xc3d7cb,
    );
    for i in 0..FULL {
        p.round_rect(
            778.0 + i as f32 * 28.0,
            625.0,
            21.0,
            11.0,
            4.0,
            if game.display_growth >= i as f32 + 0.8 {
                0x70daa3
            } else {
                0x304737
            },
        );
    }
    p.center(
        x,
        666.0,
        260.0,
        language.text("Fullvuxet träd = nästa nivå!", "Grow it fully to level up!"),
        14,
        0xc3d7cb,
    );
}

fn butterflies(p: &mut Painter, time: f32, tree_x: f32, canopy_y: f32, fraction: f32) {
    let colors = [
        (0xffd77a, 0xffad66),
        (0xd5b0ff, 0xa684eb),
        (0xffb1d2, 0xe779ae),
        (0x97e5ff, 0x59bcdf),
        (0xfff0a5, 0xe9c768),
    ];
    for (i, &(upper, lower)) in colors
        .iter()
        .enumerate()
        .take(1 + (fraction * 4.0) as usize)
    {
        let phase = time * (0.7 + i as f32 * 0.08) + i as f32 * 2.3;
        let x = tree_x + phase.sin() * (55.0 + i as f32 * 5.0);
        let y = canopy_y + 12.0 + (phase * 1.4).cos() * 35.0 + (phase * 2.1).sin() * 9.0;
        let wing = 3.0 + (time * 8.0 + i as f32).sin().abs() * 10.0;
        for side in [-1.0, 1.0] {
            let cx = x + side * (wing / 2.0 + 1.0);
            p.ellipse(cx - wing / 2.0, y - 11.0, wing, 13.0, upper);
            p.ellipse(cx - wing * 0.4, y, wing * 0.8, 9.0, lower);
            p.line(x, y - 5.0, x + side * 4.0, y - 13.0, 1.0, 0xece5cf);
        }
        p.line(x, y - 6.0, x, y + 8.0, 3.0, 0x403842);
        p.ellipse(x - 2.0, y - 9.0, 4.0, 4.0, 0x403842);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_resolution_mapping() {
        for (w, h) in [(1000, 700), (1920, 1080), (3840, 2160), (1600, 1200)] {
            let v = Viewport::new(w, h);
            assert!((v.logical_x(v.x + 360.0 * v.scale) - 360.0).abs() < 0.01);
            assert!(WIDTH * v.scale <= w as f32 + 0.01);
            assert!(HEIGHT * v.scale <= h as f32 + 0.01);
        }
    }
}
