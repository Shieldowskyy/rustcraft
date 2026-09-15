# RustCraft — wokselowa gra Minecraft-podobna z wybieralnym rendererem

Mały silnik wokselowy w Rust z dwoma trybami uruchomienia. Renderer programowy
wykonuje **cały rendering na CPU**:
transformacje 3D, przycinanie (clipping), rasteryzacja trójkątów, bufor
głębokości (z-buffer), oświetlenie per-face i mgła dystansowa — wszystko
liczone ręcznie, piksel po pikselu. Tryb Vulkan inicjalizuje urządzenie Vulkan,
zachowując tę samą prezentację okna i obsługę wejścia przez `minifb`.

Biblioteka `minifb` jest użyta **wyłącznie** do stworzenia okna systemowego
i odczytu klawiatury/myszy — nie renderuje niczego sama. Program co klatkę
sam wypełnia bufor pikseli (`Vec<u32>`) i przekazuje go do `minifb`, które
tylko kopiuje go na ekran (`update_with_buffer`).

## Wymagania

- Rust (stable) — zalecany `rustup` (https://rustup.rs), wersja 1.75+
- Linux: pakiety deweloperskie X11 (najczęściej już są, ale w razie błędów
  linkera zainstaluj np. `libx11-dev` na Debian/Ubuntu)
- Windows / macOS: nic dodatkowego — `minifb` korzysta z natywnych API okien

## Budowanie i uruchamianie

```bash
cd rustcraft
cargo run --release
```

Domyślnie używany jest renderer programowy. Renderer Vulkan można wybrać przy
uruchamianiu:

```bash
cargo run --release -- --renderer vulkan
# skróty: --vulkan oraz --software
```

Tryb Vulkan wymaga zainstalowanego loadera Vulkan i urządzenia z kolejką
graficzną. Bez obsługi Vulkan użyj `--renderer software`.

Pierwsza kompilacja pobierze i skompiluje zależności, w tym `wgpu` z backendem Vulkan dla trybu
Vulkan oraz `minifb` dla okna i wejścia.

Na Linuksie/X11 dodatkowo używamy bezpośrednio Xlib (przez `x11-dl`, bez
GPU) do **prawdziwego przechwycenia i ukrycia kursora myszy** w oknie gry —
patrz sekcja "Przechwytywanie myszy" niżej.

## Wymagania

- Rust (stable) — zalecany `rustup` (https://rustup.rs), wersja 1.75+
- Linux: pakiety deweloperskie X11 (najczęściej już są; w razie błędów
  linkera zainstaluj np. `libx11-dev` na Debian/Ubuntu)
- Windows / macOS: nic dodatkowego — `minifb` korzysta z natywnych API okien
  (na tych platformach mysz działa w trybie "śledzenia delty" — patrz niżej)

## Budowanie i uruchamianie

```bash
cd rustcraft
cargo run --release
```

## Sterowanie

| Klawisz / przycisk | Akcja |
|---|---|
| Kliknięcie w okno | zablokuj/przechwyć mysz (rozpocznij grę) |
| Esc | zwolnij mysz; drugi raz (gdy mysz już zwolniona) — wyjście |
| W / A / S / D | ruch w poziomie |
| Spacja | skok (w trybie lotu: w górę) |
| Lewy Shift | w dół (tylko w trybie lotu) |
| Lewy Ctrl (przytrzymaj) | bieg |
| F | przełącz tryb lotu (bez grawitacji/kolizji — wygodne do budowania) |
| Mysz | rozglądanie się (tylko gdy przechwycona) |
| Lewy przycisk myszy | zniszcz blok, na który patrzysz |
| Prawy przycisk myszy | postaw wybrany blok |
| 1–6 | wybierz typ bloku do stawiania |

## Przechwytywanie myszy

`minifb` nie ma żadnego wbudowanego API do "zablokowania"/"warpowania"
kursora na żadnej platformie. Żeby mysz naprawdę nie uciekała z okna
podczas rozglądania się, projekt implementuje własny moduł
(`mouse_capture.rs`):

- **Na Linuksie/X11**: po kliknięciu w okno program pobiera surowy uchwyt
  Xlib z okna `minifb` (przez `raw-window-handle`) i wywołuje bezpośrednio
  `XGrabPointer` (przechwycenie kursora), ukrywa kursor (`XDefineCursor` z
  niewidocznym kursorem 1×1) i co klatkę odczytuje pozycję kursora
  względem środka okna (`XQueryPointer`), po czym **warpuje go z powrotem
  na środek** (`XWarpPointer`) — dzięki temu kursor nigdy nie dotrze do
  krawędzi okna, a delta ruchu jest liczona bez ograniczeń.
- **Na Windows / macOS lub jeśli X11 nie jest dostępne** (np. czysty
  Wayland bez XWayland): program automatycznie przechodzi na tryb
  zapasowy — śledzenie różnicy pozycji kursora między klatkami. W tym
  trybie kursor może dotrzeć do krawędzi okna, ale gra pozostaje w pełni
  grywalna.

Naciśnięcie Esc zwalnia przechwycenie (kursor wraca, widoczny), drugie Esc
(gdy mysz jest już zwolniona) zamyka grę.

## Tekstury proceduralne

Zamiast plików graficznych, wszystkie tekstury bloków (trawa, ziemia,
kamień, piasek, drewno, liście, woda) są **generowane proceduralnie w
`texture.rs`** jako bitmapy 16×16 pikseli, przy starcie programu — przy
użyciu deterministycznego szumu (hash-based), bez żadnych zewnętrznych
plików graficznych czy assetów. Każdy blok ma osobne tekstury dla różnych
ścian (np. trawa: zielony, cętkowany wierzch, ziemisty spód, i "przejściowy"
bok z pasem trawy nad ziemią — jak w oryginalnym Minecrafcie).

Renderer próbkuje te tekstury per-piksel z perspektywicznie poprawną
interpolacją UV (nearest-neighbor, żeby zachować pikselowy klimat), a
następnie mnoży wynik przez stałe cieniowanie kierunkowe per ściana
(imitacja ambient occlusion) i mgłę dystansową.

## Podstawowe UI

Doszedł prosty interfejs, rysowany bezpośrednio w buforze pikseli (bez
żadnej biblioteki UI):

- **Celownik** na środku ekranu.
- **Hotbar** na dole z 6 slotami bloków, podświetlonym aktualnie wybranym
  slotem i numerkami 1–6 (rysowanymi wbudowaną, własną czcionką bitmapową
  3×5 z `font.rs`).
- **Panel debugowy** w lewym górnym rogu: FPS, pozycja gracza (X/Y/Z),
  nazwa wybranego bloku, tryb (normalny/lot).
- **Ekran "kliknij aby grać"** wyświetlany zawsze, gdy mysz nie jest
  przechwycona, z informacją czy przechwycenie będzie natywne (X11) czy w
  trybie zapasowym.

## Jak to działa (skrót architektury)

- `math.rs` — własna implementacja `Vec3`, `IVec3` i macierzy 4x4.
- `noise.rs` — deterministyczny szum wartościowy do generowania terenu.
- `world.rs` — chunki 16×64×16, streaming, generacja terenu, raycasting DDA,
  oraz test kolizji AABB (`box_collides`) używany przez fizykę gracza.
- `block.rs` — definicje bloków, kolory-tinty i przypisanie tekstur per
  ściana.
- `texture.rs` — generator proceduralnych tekstur 16×16 dla każdego bloku.
- `font.rs` — wbudowana bitmapowa czcionka 3×5 do rysowania tekstu w UI.
- `mesher.rs` — zamienia siatkę wokseli chunku na trójkąty z UV, z
  cullingiem niewidocznych ścian.
- `renderer.rs` — **serce projektu**: transformacja do przestrzeni clip,
  przycinanie do płaszczyzny bliskiej (Sutherland–Hodgman), rzutowanie
  perspektywiczne, rasteryzacja (edge function) z perspektywicznie poprawną
  interpolacją UV, próbkowanie tekstur, z-bufor, mgła, oraz rysowanie UI
  (crosshair, hotbar, panel debugowy, prompt przechwycenia myszy).
- `mouse_capture.rs` — prawdziwe przechwycenie/ukrycie kursora na
  Linuksie/X11 przez bezpośrednie wywołania Xlib, z bezpiecznym fallbackiem
  na innych platformach.
- `camera.rs` — kamera FPS (yaw/pitch).
- `main.rs` — pętla gry: input, fizyka (grawitacja + kolizja AABB), tryb
  lotu, interakcja z blokami, wywołanie renderera/UI, wysłanie klatki.

## Znane uproszczenia

- Brak oświetlenia dynamicznego/cieni — stałe cieniowanie per ściana + mgła.
- Drzewa mogą być lekko przycinane na granicy chunków.
- Rozmiar okna jest stały (bez obsługi zmiany rozmiaru w locie).

Projekt był kompilowany od zera i uruchamiany (pod Xvfb, z symulowanymi
zdarzeniami myszy/klawiatury przez `xdotool`, potwierdzającymi że
przechwycenie X11 działa bez paniki) w środowisku testowym przed wysłaniem.

