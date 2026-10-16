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

## 🚀 Instrukcja instalacji i uruchamiania na konsoli

### Metoda 1: Uruchomienie przez XeLL Reloaded (Zalecane – pełna grafika TV 720p)
Silnik RustCraft dla Xbox 360 korzysta z bezpośredniego bufora liniowego VRAM układu graficznego Xenos. W środowisku **XeLL Reloaded** obraz jest wyświetlany natychmiast na telewizorze wraz z ekranem diagnostycznym:
1. Skopiuj plik `xenon.elf` (lub `rustcraft.elf`) bezpośrednio do katalogu głównego pendrive'a sformatowanego w systemie **FAT32** (`USB:\xenon.elf`).
2. Wyłącz konsolę Xbox 360.
3. Włącz konsolę **przyciskiem EJECT** (wysunięcie tacki napędu).
4. XeLL Reloaded wystartuje, zainicjalizuje urządzenia USB i sieć, po czym automatycznie uruchomi grę z natywnym obrazem 720p HD i konsolą diagnostyczną na ekranie TV!

### Metoda 2: Uruchomienie z Dashboardu (Aurora / XeXMenu) przez `xelllaunch.xex`
Jeśli chcesz uruchamiać grę wygodnie z menu Aurory lub XeXMenu bez wyłączania konsoli:
1. Skopiuj folder `RustCraft` na pendrive (`USB:\RustCraft\`).
2. Upewnij się, że plik `xenon.elf` znajduje się w katalogu głównym pendrive'a (`USB:\xenon.elf`).
3. W Aurora lub XeXMenu wejdź do folderu `RustCraft` i uruchom plik **`xelllaunch.xex`**.
4. Konsola zrestartuje się bezpiecznie do XeLL i natychmiast uruchomi RustCraft.

### Metoda 3: Diagnostyka przez `default.xex` (Aurora / XeXMenu)
* Bezpośrednie uruchomienie `default.xex` z poziomu menedżera plików przeprowadza pełen pakiet testów sprzętowych (CPU, 64 MB alokator pamięci GDDR3, system plików USB, kontrolery) oraz zapisuje pełny dziennik zdarzeń do pliku:
  * `USB:\RustCraft\rustcraft.log`
  * `USB:\rustcraft.log`
* Po zakończeniu testów aplikacja bezpiecznie i czysto kończy działanie bez zawieszania konsoli!

---

## 🕹️ Sterowanie na padzie Xbox 360

| Przycisk / Gałka | Akcja w grze | Ekran Diagnostyczny |
| :--- | :--- | :--- |
| **Przycisk [A]** | Skok / Pływanie / Wznoszenie (lot) | Zamknięcie konsoli / Start gry |
| **Przycisk START** | Menu pauzy / Ustawienia | Zamknięcie konsoli / Start gry |
| **Wciśnięcie RS (R3)** | Przełączenie konsoli diagnostycznej | Schowanie / Pokazanie diagnostyki |
| **Lewa gałka (LS)** | Chodzenie (przód / tył / lewo / prawo) | Ruch wirtualnym kursorem |
| **Prawa gałka (RS)** | Obracanie kamery (płynny rozgląd) | Dodatkowa kontrola kursora |
| **Prawy Trigger (RT)** | Wydobywanie klocka / Atak | Kliknięcie lewym przyciskiem myszy |
| **Lewy Trigger (LT)** | Postawienie klocka / Interakcja | Kliknięcie prawym przyciskiem myszy |
| **Przycisk [B]** | Skradanie / Opadanie (lot) | Powrót / Wyjście |
| **Przycisk [X]** | Wyrzucenie trzymanego przedmiotu | Szybkie rzemiosło |
| **Przycisk [Y]** | Otwarcie ekwipunku | Zamknięcie ekwipunku |
| **Bumpery [LB] / [RB]** | Wybór slotu paska podręcznego | Zmiana zakładek |
| **Przycisk BACK** | Przełączenie trybu latania (Fly Mode) | Przełączenie kamery (F5) |
| **Wciśnięcie LS (L3)** | Włączenie sprintu | - |

---

## 💾 Zapisy stanów gry

RustCraft na Xbox 360 automatycznie montuje i obsługuje zapisy światów:
* Przy podłączonym dysku USB: `uda0:/RustCraft/saves/`
* Przy zapisie na dysku konsoli: `sda0:/RustCraft/saves/`
Zapisy zawierają ukształtowanie terenu, stan gracza, zawartość skrzyń, pieców oraz pieczęć czasową.

---

## 📋 Diagnostyka i zrzut logów na pendrive (`rustcraft.log`)

Silnik posiada wbudowany moduł aktywnego logowania zdarzeń oraz ekran diagnostyczny (**On-Screen Diagnostic Console**):
* **Na ekranie TV:** Przy starcie gry wyświetla się panel diagnostyczny pokazujący stan wszystkich 6 etapów startowych (taktowanie procesora, test alokatora 64 MB, wykrywanie pamięci masowej, bufor wideo, generowanie świata i pad).
* **Lokalizacja logu:** Przy każdym uruchomieniu silnik natychmiast tworzy lub nadpisuje plik dziennika bezpośrednio na pendrive:
  * `USB:\RustCraft\rustcraft.log`
  * oraz `USB:\rustcraft.log`
* **Zabezpieczenie przed utratą danych:** Każda linia logu jest natychmiast wymuszana do zapisu na pamięć flash USB (`xbox_flush_log_to_disk`), dzięki czemu w razie problemu cała historia zdarzeń znajduje się w pliku na pendrive.

---

## 🖥️ Testowanie na komputerze PC (Emulator Xenia)

Jeżeli chcesz przetestować wygenerowany plik `default.xex` na komputerze:
1. Pobierz emulator **Xenia Canary** ze strony: https://github.com/xenia-canary/xenia-canary/releases
2. Uruchom `xenia_canary.exe`.
3. Wybierz `File -> Open` i wskaż plik `default.xex`.
