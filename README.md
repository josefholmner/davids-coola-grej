# Kunskapsträdet / The Learning Tree

Ett fristående spel för Windows, skrivet i Rust. **Dubbelklicka på `dist/NumberCatch.exe` för att starta.** Du behöver inte Python, Rust eller någon separat bildmapp för att spela.

## Välj spel

Startmenyn har två bildknappar: en uträkning för Matematik och en öppen bok för Läsning. Klicka på en bild eller tryck **1** för Matematik, **2** för Läsning eller **S** för Inställningar.

**Svenska är förvalt.** I Inställningar kan du välja **English**. Språkvalet gäller båda spelen och sparas i `NumberCatch.settings` bredvid programmet.

### Matematik

Fånga uträkningar vars svar är lika med måltalet. Varje uträkning räknas för sig.

- Nivå 1, 2, 4, 5, 7, 8 och så vidare använder **bara addition**.
- Var tredje nivå (3, 6, 9, ...) använder **bara subtraktion**.
- Subtraktionsnivåerna har mindre tal, lägre fallhastighet och längre tid mellan uppgifterna. Den första använder bara talen 1–3 och svaren 1–2.
- Addition börjar med en fallhastighet på 45–55 pixlar/sekund och subtraktion med 32–42. Båda ökar gradvis med nivåerna.
- Båda typerna blir gradvis svårare. Hastigheten ökar långsammare än i den tidigare Python-versionen.
- Rätt additionssvar blandas jämnt mellan alla positiva talpar. För målet 10 används 1+9, 2+8, 3+7, 4+6 och 5+5 innan ett talpar upprepas; ordningen på termerna kan också växla. Subtraktionens giltiga varianter blandas på samma sätt.

### Läsning

Titta på bilden högst upp och fånga ordet som hör till bilden. Till exempel: en sten → **sten**, eller **rock** om engelska är valt. Orden faller ned i stället för uträkningar.

Spelet innehåller **220 olika illustrerade saker**, med både svenska och engelska ord. Alla ord har 3–7 bokstäver. Å, ä och ö räknas som en bokstav vardera.

| Nivå | Ordens längd |
| --- | --- |
| 1–2 | 3 bokstäver |
| 3–4 | 4 bokstäver |
| 5–6 | 5 bokstäver |
| 7–8 | 6 bokstäver |
| 9 och högre | 7 bokstäver |

För varje ny bild är det första ordet rätt en gång av fem, och rätt ord finns alltid bland de fem första orden. Därefter kommer 1–3 felord mellan rätta ord. Samma bild får aldrig två rätta ord i följd, och ett nytt försök på samma bild återställer inte mellanrummen. Felorden varierar också.

De andra orden som faller har samma längd som målbildens ord. Bilderna har inga ord eller etiketter som avslöjar svaret.

## Trädet och nivåerna

Trädet börjar med två av sex tillväxtsteg. Rätt svar får det att växa ett steg och ger poäng. Fel svar får det att krympa ett steg. Att låta något falla förbi kostar ingenting.

Ett fullvuxet träd tar dig till nästa nivå, med en ny bakgrundsfärg och ett nytt litet träd. Om trädet krymper helt är spelet slut, även på nivå 1. Resultatskärmen visar din nivå och poäng. Tryck R eller Enter, eller klicka på Igen, för att börja om; du kan också gå tillbaka till menyn. Fjärilarna flyger runt trädet och blir fler när det växer.

## Kontroller

- **Piltangenter**, **A/D** eller **mus**: flytta korgen.
- **F11**: växla helskärm.
- **Esc**: lämna helskärm; annars gå till startmenyn. I startmenyn avslutar Esc spelet.
- **M** eller **Meny-knappen**: gå till startmenyn.
- **Mellanslag**: pausa eller fortsätt.
- **R**: börja om det valda spelet från nivå 1.
- Muspekaren döljs under aktivt spel och visas igen i menyer, vid paus och när spelet är slut.

Spelet pausas automatiskt när du byter till ett annat program.

Rätt fångst ger en kort uppåtgående klang, gröna gnistor och en liten studs vid korgen. Fel fångst ger en lägre ton, rosa partiklar och en kort skakning. Ljuden är inbyggda och fungerar utan separata ljudfiler.

Helskärm täcker även aktivitetsfältet. När du byter till ett annat program släpps fönstrets översta placering, och normal fönsterordning återställs när du lämnar helskärm.

## Grafik

Text, träd, fjärilar och spelobjekt ritas med kantutjämnade vektorformer direkt i fönstrets verkliga pixelupplösning. Helskärm använder bildskärmens upplösning; ingen lågupplöst 1000×700-bild förstoras. Programmet är Per-Monitor V2 DPI-medvetet och behåller proportionerna på olika skärmar. Endast en färdigritad bildruta visas åt gången för att undvika blinkningar. En högupplöst bildruteklocka ger jämnare fallrörelser, och texten i de fallande rutorna kantutjämnas utan att låsas till pixelrader. F11 visas även tillsammans med tangentbordsgenvägarna längst ned.

De genererade ordbilderna är högupplösta och inbyggda i exe-filen. Högst åtta bilder mellanlagras för att hålla minnesanvändningen nere.

## English

Double-click **dist/NumberCatch.exe**. Choose **Inställningar → English** to switch both games and menus to English. The app includes Math and Reading modes, 220 illustrated bilingual objects, tree growth, butterflies, and fullscreen support. No installation or separate assets are needed.

## Build from source

Requires stable Rust for `x86_64-pc-windows-msvc`, Visual Studio C++ build tools, and a Windows SDK. There are no third-party Cargo dependencies.

```powershell
.\build.ps1
```

This runs the Rust tests and creates a self-contained `dist/` folder containing `NumberCatch.exe` and a short player README. Alternatively, use `cargo test` and `cargo build --release` in a Visual Studio developer shell.

### Portable release

Zip the entire `dist/` folder, send it to another **Windows 10/11 x64** computer, extract it, and double-click `NumberCatch.exe`. All 220 illustrations are embedded; no Python, Rust, internet connection, or external asset folder is required.

`dist/` is intentionally allowed in Git. Its executable uses the repository's Git LFS rules. Commit `.gitattributes` together with `dist/`; collaborators need Git LFS to obtain the real executable from a clone. Local language preferences and error logs in `dist/` are ignored.

The Windows C runtime is linked statically. The remaining graphics and windowing DLLs ship with Windows 10/11.

### Source and art

- `src/game.rs`: math/reading rules and difficulty.
- `src/app.rs`: menu and settings navigation.
- `src/render.rs`: native-resolution rendering and image cache.
- `src/language.rs`: Swedish/English selection and portable settings.
- `src/vocabulary.rs`: graded bilingual vocabulary.
- `assets/reading/`: original generated cartoon PNGs, vocabulary metadata and prompts.
- `game.py` and `windows_graphics.py`: the earlier Python version, retained for reference; run the Rust executable for the new modes and rules.

Developer rendering checks:

```powershell
.\dist\NumberCatch.exe --smoke-test smoke.json
.\dist\NumberCatch.exe --verify-assets asset-check.json
.\dist\NumberCatch.exe --render-preview 3840 2160 reading.png reading sv
```

Preview screens: `menu`, `settings`, `math`, `subtraction`, `reading`; languages: `sv`, `en`.

## Credits

By Josef Holmner, with special thanks to David Nilsson.

På svenska: Av Josef Holmner, med särskilt tack till David Nilsson.
