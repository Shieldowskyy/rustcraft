# RustCraft – Microsoft Xbox 360 Homebrew Edition

Instrukcja instalacji i uruchamiania RustCraft na przerobionej konsoli **Xbox 360** (JTAG / RGH 1.2 / RGH 2.0 / RGH 3.0 / BadUpdate / XeLL).

---

## 🎮 Wymagania

* Skonsolowany exploit: **RGH (Reset Glitch Hack)**, **JTAG** lub **XeLL Reloaded**.
* Zainstalowane środowisko homebrew:
  * **Aurora Dashboard** (zalecane) LUB
  * **XeXMenu 1.1 / 1.2** LUB
  * **Freestyle Dash (FSD3)** LUB
  * **DashLaunch 3.xx**
* Pendrive USB sformatowany w systemie **FAT32** lub wewnętrzny dysk twardy konsoli (**Hdd1:\**).

---

## 📦 Zawartość paczki `RustCraft-Xbox360.zip`

Folder gry (`RustCraft/`) zawiera:
* `default.xex` — główny plik wykonywalny w standardzie XEX2 (niekompresowany, nieszyfrowany, ze zniesioną weryfikacją podpisu RSA dla exploita RGH/JTAG).
* `rustcraft.elf` — binarka ELF dla bezpośredniego bootowania przez **XeLL Reloaded** (przycisk Eject podczas włączania konsoli).
* `icon.png` — ikona gry 128x128 wyświetlana w menu Aurora, FreestyleDash i XeXMenu.
* `nxeart.png` — panoramiczne tło 1280x720 w stylu NXE Dashboard.
* `title.meta` — metadane dla automatycznego skanera gier w Aurora (Title ID: `0x58580864`).
* `saves/` — folder na zapisy światów (slot 1-5).

---

## 🚀 Instrukcja instalacji na konsoli

### Metoda 1: Uruchomienie z pendrive'a USB (najprostsza)
1. Wypakuj archiwum `RustCraft-Xbox360.zip`.
2. Skopiuj cały folder `RustCraft` do katalogu głównego pendrive'a FAT32, aby ścieżka wyglądała następująco:
   ```text
   USB:\RustCraft\default.xex
   USB:\RustCraft\icon.png
   USB:\RustCraft\title.meta
   ```
3. Włóż pendrive do dowolnego portu USB w konsoli Xbox 360.
4. Uruchom **XeXMenu** lub **Aurora Dashboard**.
5. W XeXMenu:
   * Wciśnij `RB`, przejdź do napędu `Usb0:\`.
   * Wejdź do katalogu `RustCraft`.
   * Zaznacz `default.xex` i wciśnij przycisk **[A]**.
6. Gra wystartuje w natywnej rozdzielczości 720p HD (1280x720 60 FPS)!

### Metoda 2: Instalacja na wewnętrznym dysku twardym (Hdd1)
1. Uruchom menedżer plików w **XeXMenu** lub połącz się z konsolą przez **FTP** (np. FileZilla, port 21).
2. Prześlij folder `RustCraft` do lokalizacji:
   ```text
   Hdd1:\Homebrew\RustCraft\
   ```
3. W **Aurora Dashboard**:
   * Wejdź w Ustawienia -> Content -> Manage Paths -> Add.
   * Wybierz `Hdd1:\Homebrew\`.
   * Wciśnij `Scan Content`.
   * RustCraft pojawi się automatycznie na liście gier z dedykowaną ikoną i tłem!

### Metoda 3: Bezpośredni start przez XeLL (przycisk Eject)
1. Skopiuj plik `rustcraft.elf` bezpośrednio na pendrive jako:
   ```text
   USB:\xenon.elf
   # lub
   USB:\rustcraft.elf
   ```
2. Włącz konsolę Xbox 360 przyciskiem **EJECT** (wysunięcie tacki).
3. XeLL Reloaded wystartuje, zainicjalizuje sieć oraz USB, wykryje plik ELF i automatycznie uruchomi grę.

---

## 🕹️ Sterowanie na padzie Xbox 360

| Przycisk / Gałka | Akcja w grze | Akcja w Menu / Ekwipunku |
| :--- | :--- | :--- |
| **Lewa gałka (LS)** | Chodzenie (przód / tył / lewo / prawo) | Ruch wirtualnym kursorem |
| **Prawa gałka (RS)** | Obracanie kamery (płynny rozgląd) | Dodatkowa kontrola kursora |
| **Prawy Trigger (RT)** | Wydobywanie klocka / Atak | Kliknięcie lewym przyciskiem myszy |
| **Lewy Trigger (LT)** | Postawienie klocka / Interakcja | Kliknięcie prawym przyciskiem myszy |
| **Przycisk [A]** | Skok / Pływanie / Wznoszenie (lot) | Wybór / Zatwierdzenie |
| **Przycisk [B]** | Skradanie / Opadanie (lot) | Wstecz / Wyjście z menu |
| **Przycisk [X]** | Wyrzucenie trzymanego przedmiotu | Szybkie rzemiosło |
| **Przycisk [Y]** | Otwarcie ekwipunku | Zamknięcie ekwipunku |
| **Bumpery [LB] / [RB]** | Wybór poprzedniego / następnego slotu paska | Zmiana zakładek |
| **Krzyżak (D-Pad)** | Wybór slotu paska podręcznego | Skokowe przeskakiwanie po polach |
| **Przycisk START** | Menu pauzy / Ustawienia | Powrót do gry |
| **Przycisk BACK** | Przełączenie trybu latania (Fly Mode) | Przełączenie kamery (F5) |
| **Wciśnięcie LS (L3)** | Włączenie sprintu | - |
| **Wciśnięcie RS (R3)** | Włączenie nakładki diagnostycznej F3 (Profiler) | - |

---

## 💾 Zapisy stanów gry

RustCraft na Xbox 360 automatycznie montuje i obsługuje zapisy światów:
* Przy podłączonym dysku USB: `uda0:/RustCraft/saves/`
* Przy zapisie na dysku konsoli: `sda0:/RustCraft/saves/`
Zapisy zawierają ukształtowanie terenu, stan gracza, zawartość skrzyń, pieców oraz pieczęć czasową.

---

## 📋 Diagnostyka i zrzut logów na pendrive (`rustcraft.log`)

Silnik posiada wbudowany moduł aktywnego, asynchronicznego i synchronicznego logowania zdarzeń oraz przechwytywania awarii (Crash Handler):
* **Lokalizacja logu:** Przy każdym uruchomieniu silnik natychmiast tworzy lub nadpisuje plik dziennika bezpośrednio w katalogu z grą na pendrive:
  * `USB:\RustCraft\rustcraft.log`
  * oraz `USB:\rustcraft.log`
* **Zabezpieczenie przed utratą danych:** Każda linia logu jest natychmiast wymuszana do zapisu na pamięć flash USB za pomocą instrukcji `flush()` i `sync_data()`.
* **Przechwytywanie crasha (Panic Hook & Hardware Traps):** 
  * W razie wystąpienia błędu w grze lub wyjątku procesora (DSI, ISI, Alignment, Illegal Instruction), handler natychmiast zrzuca na pendrive pełny raport z informacją o pliku, linii kodu, treści błędu oraz stanie rejestrów CPU PowerPC (r0-r31, PC, MSR, LR, Timebase).
  * Na ekranie telewizora wyświetlana jest awaryjna czerwona plansza diagnostyczna z informacją o zapisaniu logu na pendrive.

---

## 🖥️ Testowanie na komputerze PC (Emulator Xenia)

Jeżeli chcesz przetestować wygenerowany plik `default.xex` na komputerze:
1. Pobierz emulator **Xenia Canary** ze strony: https://github.com/xenia-canary/xenia-canary/releases
2. Uruchom `xenia_canary.exe`.
3. Wybierz `File -> Open` i wskaż plik `default.xex`.
