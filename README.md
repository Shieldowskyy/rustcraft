# RustCraft — wokselowa gra Minecraft-podobna, wieloplatformowa

Lekki silnik wokselowy napisany w Rust z obsługą **czterech platform**:
desktopu (Linux/Windows), PlayStation Vita, przeglądarki (WebAssembly/HTML5)
i macOS. Renderer programowy wykonuje cały rendering na CPU — rasteryzacja
trójkątów, z-buffer, AO, mgła dystansowa — bez żadnych zależności graficznych.
Tryb GPU (WGPU/Vulkan) i dedykowany renderer GXM na PlayStation Vita
(GPU SGX543MP4) zapewniają płynne 60 FPS na każdej platformie.

---

## Obsługiwane platformy

| Platforma | Renderer | Skrypt / polecenie |
|---|---|---|
| Linux (x86_64) | Software CPU lub WGPU/Vulkan | `./compile_linux.sh` |
| Windows (x86_64) | Software CPU lub WGPU/Vulkan | `cargo build --release` |
| WebAssembly | Software CPU (Rayon wyłączony) | `./build_web.sh` |
| PlayStation Vita | GXM GPU (SGX543MP4) | `./build_vita.sh` |

---

## Wymagania

### Desktop (Linux / Windows / macOS)
- Rust (stable) — zalecany `rustup` (https://rustup.rs), wersja 1.75+
- **Linux**: pakiety deweloperskie X11 (`libx11-dev` na Debian/Ubuntu)
- **Windows / macOS**: nic dodatkowego — `minifb` korzysta z natywnych API okien
- Renderer Vulkan wymaga zainstalowanego loadera Vulkan i urządzenia z kolejką graficzną

### PlayStation Vita
- [VitaSDK](https://vitasdk.org) zainstalowany w `/usr/local/vitasdk`
- Target Rust: `armv7-sony-vita-newlibeabihf`
- Kompilator shaderów Cg: `psp2cgc` (automatycznie wykrywany przez `build.rs`)
- Narzędzia: `vita-elf-create`, `vita-make-fself`, opcjonalnie UPX

### WebAssembly
- `wasm-pack` lub `wasm-bindgen-cli`
- Serwer HTTP (np. `python3 -m http.server`) do podglądu lokalnego

---

## Budowanie i uruchamianie

### Linux / Desktop (domyślny — software renderer)
```bash
./compile_linux.sh          # release + strip + UPX
# lub ręcznie:
cargo run --release
```

Renderer Vulkan można wybrać przy uruchamianiu:
```bash
cargo run --release -- --renderer vulkan
# skróty: --vulkan oraz --software
```

Custom seed można podać przy uruchamianiu:
```bash
cargo run --release -- --seed 8675309
cargo run --release -- --renderer vulkan --seed 8675309
```

### PlayStation Vita
```bash
./build_vita.sh
# Wynik: RustCraft.vpk — instalacja przez VitaShell
```
`build.rs` automatycznie kompiluje shadery Cg (`vita/shaders/*.cg → *.gxp`)
za pomocą `psp2cgc`. Jeśli kompilator nie jest dostępny, używane są shadery
prebuilt z `vita/shaders/prebuilt/`.

### WebAssembly / HTML5
```bash
./build_web.sh
# Następnie otwórz web/index.html przez lokalny serwer HTTP
```

---

## Menu i zapis

Po uruchomieniu pojawia się menu główne. Strzałki `Left/Right` wybierają jeden
z 32 slotów zapisu, `Enter` tworzy nowy świat, `L` wczytuje wybrany zapis,
a `Q` kończy program. `F5` zapisuje aktywny świat, podobnie jak wyjście z pauzy.

Pliki zapisu: `rustcraft_world_01.bin` … `rustcraft_world_32.bin`.
Na PlayStation Vita: `ux0:data/RustCraft/rustcraft_world_01.bin` itd.

---

## Sterowanie — Desktop

| Klawisz / przycisk | Akcja |
|---|---|
| Kliknięcie w okno | zablokuj/przechwyć mysz (rozpocznij grę) |
| Esc | zwolnij mysz; drugi raz — wyjście |
| W / A / S / D | ruch w poziomie |
| Spacja | skok (w trybie lotu: w górę) |
| Lewy Shift | kucanie / w dół (lot) |
| Lewy Ctrl (przytrzymaj) | bieg |
| F | przełącz tryb lotu |
| Mysz | rozglądanie się |
| LMB | zniszcz blok |
| RMB | postaw blok |
| 1–0 | wybierz slot na hotbarze (10 slotów) |
| E | otwórz ekwipunek / crafting 2×2 |
| Tab / F1 | kreatywny selektor bloków (Creative mode) |
| F5 | zapisz świat |
| P | menu pauzy |
| F3 | panel debugowy (FPS, pozycja, biom, renderer) |

## Sterowanie — PlayStation Vita

| Przycisk Vita | Akcja |
|---|---|
| Lewy stick | ruch (WASD) |
| Prawy stick | kamera |
| × | skok |
| ○ | postaw blok |
| □ | zniszcz blok |
| R Trigger (RT) | zniszcz blok (zamiennie) |
| L Trigger (LT) | postaw blok (zamiennie) |
| Start | menu pauzy |
| D-Pad | nawigacja hotbara / UI |
| Panel dotykowy | obrót kamery (delta dotyku) |

## Sterowanie — przeglądarka (WASM)
- Mysz + klawiatura identyczne jak desktop
- Gamepad API: mapowanie analogicznie do PS Vita / kontrolera Xbox

---

## Przechwytywanie myszy

`minifb` nie ma wbudowanego API do blokowania kursora. Projekt implementuje
własny moduł (`src/platform/mouse_capture.rs`):

- **Linux/X11**: `XGrabPointer`, `XWarpPointer`, `XFixesHideCursor` —
  kursor jest warpowany na środek okna co klatkę; prawdziwa blokada bez utraty delty.
- **Windows (Win32)**: `ClipCursor`, `SetCursorPos`, `ShowCursor`.
- **Fallback**: śledzenie różnicy pozycji między klatkami (Wayland, macOS bez obsługi natywnej).
- **Vita**: kursor jest zawsze przechwycony (brak kursora systemowego).

---

## Gamepad (`src/platform/gamepad.rs`)

Obsługa gamepadów via `gilrs`. Mapowanie obejmuje osie analogowe (ruch, kamera),
triggery RT/LT do niszczenia/stawiania bloków, pełną nawigację UI za pomocą D-Pad
i przycisków akcji. Działa na Linuksie, Windows i macOS.

---

## Biomy

Świat zawiera 8 biomów dobieranych proceduralnie na podstawie szumu:

| Biom | Opis |
|---|---|
| Plains | Płaskie równiny, dęby |
| Forest | Pagórki, gęste dęby |
| Desert | Piasek, kaktusy |
| Mountains | Wysokie szczyty kamienne |
| Snowy | Ośnieżony teren, świerki |
| Cherry Blossom | Wiśniowe drzewa z różowymi liśćmi (`CherryLeaves`) |
| Ocean | Płaskie dno morskie, woda do y=20 |
| Savanna | Suche równiny, akacjopodobne drzewa |

Aktualny biom widoczny jest w panelu debugowym (klawisz `F3`).

---

## Tekstury proceduralne

Wszystkie tekstury bloków generowane są proceduralnie w `src/graphics/atlas.rs`
jako bitmapy 16×16 pikseli przy starcie programu — bez żadnych zewnętrznych
plików graficznych. Atlas zawiera 50 warstw tekstur (`TEXTURE_LAYER_COUNT = 50`).
Na Vita atlas jest przesyłany jako pojedyncza tekstura GPU 128×128 px ABGR8888.

---

## Nowe funkcje gameplay (ostatnie commity)

- **Lawa** — opór ruchu gracza w lawie, obrażenia, mgła lawy/efekt winiety
  z cząsteczkami płomieni i dymu
- **Crafting** — stół craftingowy (2×2 w ekwipunku, 3×3 przy crafting table),
  przepisy zdefiniowane w `src/gameplay/recipe.rs`
- **Narzędzia** — kilof, siekiera, łopata ze zwiększoną szybkością kopania
- **Kreatywny selektor bloków** — siatka wszystkich bloków z możliwością
  przewijania, szybki dostęp przez hotbar
- **Efekty obrażeń** — winieta czerwona przy trafieniu, ekran śmierci
- **Pęknięcia przy niszczeniu** — 10-stopniowa animacja crack overlay na bloku
- **Wypadające itemy** — fizyczne 3D itemy na ziemi z animacją unoszenia
- **Ustawienia graficzne** — dystans renderowania, mgła, chmury, FOV,
  cząsteczki, nakładka debugowa, animacja chunk fade, efekty podwodne

---

## Jak to działa (skrót architektury)

- `src/core/math.rs` — własna implementacja `Vec3`, `IVec3`, `Mat4`.
- `src/core/noise.rs` — deterministyczny szum do generowania terenu i biomów.
- `src/core/settings.rs` — `GameSettings`: ustawienia graficzne zapisywane na dysk.
- `src/world/block.rs` — definicje 34+ typów bloków, właściwości, przypisanie tekstur.
- `src/world/mesher.rs` — buduje siatki trójkątów z Ambient Occlusion per wierzchołek.
- `src/world/mod.rs` — chunki 16×64×16, streaming, generacja terenu, biomy, raycasting.
- `src/world/storage.rs` — binarny zapis/odczyt świata (format `RCW1`).
- `src/gameplay/player.rs` — gracz, `GameMode` (Survival/Creative), zdrowie.
- `src/gameplay/physics.rs` — kolizje AABB, skoki, kucanie, tarcie, opór lawy.
- `src/gameplay/inventory.rs` — hotbar (10 slotów) + plecak (27 slotów).
- `src/gameplay/recipe.rs` — receptury craftingowe 2×2/3×3.
- `src/gameplay/item.rs` — fizyczne itemy na ziemi.
- `src/gameplay/mob.rs` — maszyny stanu mobów (Pig, Zombie), pathfinding.
- `src/gameplay/particles.rs` — cząsteczki bloków, lawa, uderzenia.
- `src/graphics/atlas.rs` — proceduralny generator tekstur (50 bloków).
- `src/graphics/software/` — wielowątkowy rasterizer CPU (Rayon, kafelki).
- `src/graphics/gpu/` — backend WGPU (Vulkan/DX12/Metal/WebGPU).
- `src/platform/mouse_capture.rs` — przechwycenie kursora X11/Win32/fallback.
- `src/platform/gamepad.rs` — obsługa gamepadów via gilrs.
- `src/platform/vita.rs` — warstwa platformowa PS Vita (display, input, moduły).
- `src/platform/vita_gxm.rs` — pełny renderer GXM na SGX543MP4 (~2500 linii).
- `src/web.rs` — bindingi wasm-bindgen, blit do `<canvas>`, Pointer Lock API.
- `src/ui/` — HUD, menu, ekwipunek, selektor bloków, ustawienia (rysowane w buforze pikseli).
- `src/app.rs` — główna pętla gry: input, fizyka, AI, render. Współdzielona między desktop, Vita i WASM.

---

## Znane uproszczenia

- Brak oświetlenia dynamicznego/cieni — stałe cieniowanie per ściana + mgła.
- Drzewa mogą być lekko przycinane na granicy chunków.
- Rozmiar okna na desktopie jest stały.
