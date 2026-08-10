# Technology Vision

Status: accepted working direction  
Date: 2026-05-31  
Scope: technology direction for the Ableton dependency safety desktop app

## 1. Kontekst produktu

Budujemy aplikacje desktopowa do pracy lokalnej na plikach Abletona i samplach.

Produkt ma:

1. skanowac foldery i projekty,
2. czytac `.als`,
3. budowac graf zaleznosci,
4. kopiowac pliki w bezpieczny sposob,
5. przepisywac kopie `.als`,
6. zapisywac manifesty operacji,
7. w przyszlosci pomagac w utrzymaniu biblioteki i cleanupie,
8. potencjalnie wspierac Mac -> Windows / Windows -> Mac,
9. potencjalnie laczyc sie z Abletonem przez osobny most, np. Max for Live albo plugin.

Najwazniejsze ryzyko:

```text
Nie wolno uszkodzic projektu ani usunac pliku, ktory jest nadal potrzebny.
```

Dlatego technologia musi wspierac:

```text
filesystem safety
transactional operations
hashing
manifest history
cross-platform paths
local database
desktop UI
CLI/testing workflow
future integration adapters
```

## 2. Rekomendacja

Rekomendowany kierunek:

```text
Rust core + Tauri desktop app + TypeScript frontend + SQLite local database
```

Wersja robocza:

```text
Core engine:
  Rust

Desktop shell:
  Tauri

UI:
  React + TypeScript
  Vite frontend

Database:
  SQLite

Experiment scripts:
  Python allowed only in experiments/

Future Ableton bridge:
  separate adapter, not part of core
```

## 3. Dlaczego Rust w core

Core bedzie robil operacje wysokiego ryzyka:

```text
read ALS
parse gzip/XML
hash audio files
copy files
stage package
rewrite ALS copy
validate semantic diff
move to quarantine
restore from quarantine
```

To przypomina bardziej:

```text
DVC
git-annex
Nix garbage collector
Unreal asset tooling
professional media packaging tools
```

niz klasyczna aplikacje webowa.

Rust pasuje, bo:

1. jest dobry do narzedzi plikowych i systemowych,
2. daje mocny model typow,
3. dobrze nadaje sie do CLI,
4. dobrze nadaje sie do logiki cross-platform,
5. dobrze wspolpracuje z Tauri,
6. pozwala trzymac niebezpieczne operacje poza UI.

Zasada:

```text
UI prosi.
Rust core decyduje i wykonuje.
```

Frontend nie powinien sam przepisywac ALS ani przenosic plikow.

## 4. Dlaczego Tauri na desktop

Tauri daje:

1. desktop app na macOS / Windows / Linux,
2. Rust backend,
3. webowy frontend,
4. male paczki aplikacji,
5. system uprawnien do filesystemu,
6. naturalny most `frontend -> Rust command`,
7. mozliwosc zbudowania CLI i desktop app wokol tego samego core.

To dobrze pasuje do produktu, ktory ma byc:

```text
lokalny
plikowy
bezpieczny
cross-platform-ready
```

Najwazniejsza zaleta architektoniczna:

```text
Tauri wymusza separacje UI od backendu.
```

To jest zdrowe dla naszego produktu.

## 5. Dlaczego TypeScript w UI

UI bedzie potrzebowalo:

```text
dependency graph
project scan results
preflight report
plan/apply flow
review screen
status badges
diff visualization
warnings
manual approvals
```

TypeScript pasuje, bo:

1. jest dobry do UI,
2. dobrze obsluguje JSON contracts,
3. dobrze wspolpracuje z Tauri,
4. latwo budowac tabele, filtry, wykresy, raporty,
5. latwo pozniej iterowac UX.

Zasada:

```text
TypeScript pokazuje i zbiera decyzje.
Rust wykonuje operacje.
```

## 6. Dlaczego SQLite

Produkt bedzie potrzebowal lokalnej pamieci:

```text
asset_index
content_hashes
project_registry
redirect_ledger
operation_manifests
verification_status
quarantine_records
user_pins
```

SQLite pasuje, bo:

1. jest lokalny,
2. nie wymaga serwera,
3. dobrze dziala jako baza desktopowa,
4. pozwala indeksowac wiele projektow i plikow,
5. jest dobry pod safe cleanup i reachability graph.

Na poczatku mozna zapisac raporty jako JSON.

Ale architektura powinna zakladac:

```text
JSON reports for artifacts
SQLite for product state
```

## 7. Rola Pythona

Python zostaje do laboratorium.

Dozwolone:

```text
experiments/
  quick scripts
  one-off diffs
  research reports
  CAS comparison helpers
```

Nie powinien byc glownym core produktu.

Powod:

```text
Nie chcemy zbudowac polowy produktu w technologii, ktora potem trzeba przepisac.
```

Zasada:

```text
Python discovers.
Rust productizes.
```

## 8. Dlaczego nie Electron jako pierwszy wybor

Electron jest sensowna opcja, zwlaszcza gdy:

```text
chcemy najszybciej zbudowac UI
chcemy wszystko trzymac w JS/TS
zespol zna web development, ale nie Rust
```

Ale dla tego produktu Electron ma slabszy profil:

1. core plikowy bylby w Node/TS albo natywnych dodatkach,
2. latwiej przypadkiem wymieszac UI z logika ryzykownych operacji,
3. paczka aplikacji jest ciezsza,
4. bezpieczenstwo operacji plikowych trzeba silniej pilnowac konwencja.

Electron nie jest bledem.

Ale jesli budujemy narzedzie typu:

```text
DVC/git-annex/Nix dla audio projektow
```

to Rust core + Tauri jest bardziej naturalne.

## 9. Dlaczego nie Swift jako glowny kierunek

Swift bylby mocny, gdyby produkt byl:

```text
Mac-only forever
gleboko natywny dla macOS
bez realnej potrzeby Windows
```

Ale mamy hipoteze przyszlej sciezki:

```text
Windows -> Mac project rescue
Mac -> Windows compatibility
cross-platform library management
```

Swift moglby zamknac nas za wczesnie w macOS.

Dlatego:

```text
Swift only for optional macOS-specific helpers later.
```

Nie jako core.

## 10. Dlaczego nie JUCE/VST teraz

JUCE jest bardzo wazne, jesli budujemy:

```text
VST3
AU
standalone audio plugin
audio processing
plugin UI
```

Ale nasz produkt teraz nie jest pluginem audio.

On jest:

```text
desktop dependency manager / rescue engine
```

VST albo AU w przyszlosci moze byc mostem do Abletona, ale nie powinien byc fundamentem.

Zasada:

```text
Desktop app first.
Ableton bridge later.
```

Jesli kiedys powstanie plugin:

```text
apps/ableton-plugin-bridge/
  JUCE/C++
```

Ale plugin ma komunikowac sie z core przez jawny kontrakt, np. local API, files, JSON, albo custom protocol.

Nie powinien zawierac logiki rewrite ALS.

## 11. Max for Live jako przyszly most

Max for Live moze byc lepszym pierwszym mostem z Abletonem niz VST.

Powod:

```text
Max for Live zyje wewnatrz Abletona i ma dostep do Live API / Live Object Model.
```

Ale Max for Live tez nie powinien byc core.

Potencjalne role:

```text
pokaz aktualnie otwarty projekt
wyslij metadane do desktop app
pomoz userowi zweryfikowac projekt
trigger scan from Ableton context
```

Nie powinien:

```text
samodzielnie przepisywac ALS
samodzielnie usuwac plikow
samodzielnie zarzadzac biblioteka
```

## 12. Architektura docelowa

```text
apps/
  desktop/
    Tauri app
    TypeScript UI

crates/
  rescue_core/
    ALS reader
    sample reference model
    path model
    hash engine
    manifest model

  rescue_analyzer/
    project scan
    dependency graph
    storage state
    preflight report

  rescue_rewriter/
    rewrite plan
    copy/staging
    ALS rewrite
    semantic diff
    validation

  rescue_index/
    SQLite schema
    asset index
    redirect ledger
    reachability graph

cli/
  rescue-cli

experiments/
  python scripts allowed
  before/after Ableton tests

specs/
  module specs
```

Na poczatku mozna uproscic:

```text
crates/rescue_core
cli/rescue-cli
specs/001-als-reader
```

Desktop UI moze wejsc dopiero, gdy core ma cos stabilnego do pokazania.

## 13. Pierwszy kamien milowy technologiczny

Nie zaczynac od Tauri UI.

Zaczac od:

```text
Rust CLI + rescue_core
```

Pierwsza komenda:

```text
rescue analyze path/to/project.als --json
```

Wynik:

```text
JSON z aktywnymi SampleRef/FileRef
liczby kontrolne
warnings
zero zmian w pliku
```

Dlaczego CLI first:

1. latwiej testowac,
2. latwiej porownywac fixture,
3. nie miesza UI z logika,
4. ten sam core pozniej zasili desktop app,
5. mozna szybko wykryc bledy w modelu danych.

## 14. Kiedy wejsc w Tauri

Tauri wprowadzic, gdy core potrafi:

```text
analyze ALS
return stable JSON
run tests on fixtures
produce preflight-like summary
```

Wtedy UI ma sens, bo pokazuje prawdziwe dane.

Nie robic UI, zanim nie wiemy, jakie dane sa stabilne.

## 15. Kiedy wejsc w SQLite

SQLite wprowadzic, gdy pojawi sie potrzeba pamieci miedzy sesjami:

```text
user scanned multiple projects
same files appear in many projects
we need asset index
we need redirect ledger
we need verification history
we need safe cleanup planning
```

Nie trzeba SQLite do samego `ALSReader`.

## 16. Kiedy wejsc w Ableton bridge

Nie teraz.

Wrocic do tego dopiero, gdy desktop/core umie:

```text
analyze
package
rewrite copy
validate
store manifest
```

Wtedy mozna rozwazyc:

```text
Max for Live bridge first
JUCE/VST bridge later only if real use case appears
```

## 17. Decyzja robocza

Do czasu jej odwolania, projekt powinien isc tym torem:

```text
1. Rust core
2. Rust CLI first
3. Tauri + React/TypeScript Desktop Alpha after the proven core slice
4. SQLite when persistent index is needed
5. Python only for experiments
6. Max for Live / JUCE as future adapters, not core
```

The core slice reached this gate on 2026-08-02: real Live 11.3 analysis,
explicit candidate selection, packaging, rewrite, static validation, manifest
and manual Ableton-open verification all completed. Desktop Alpha is therefore
the active implementation direction.

## 18. Najwazniejsza zasada technologiczna

```text
Core musi byc niezalezny od UI.
UI moze sie zmienic.
Ableton bridge moze sie zmienic.
Format raportow moze sie zmienic.
Ale logika bezpiecznego czytania, planowania, kopiowania, rewrite i walidacji musi byc jedna.
```

## 19. Zrodla

```text
Tauri:
https://v2.tauri.app/start/
https://v2.tauri.app/concept/architecture/
https://v2.tauri.app/develop/calling-rust/
https://v2.tauri.app/plugin/file-system/
https://v2.tauri.app/plugin/sql/

Electron:
https://www.electronjs.org/docs/latest/

JUCE:
https://juce.com/juce/features/
https://docs.juce.com/

Steinberg VST3 SDK:
https://www.steinberg.net/developers/vstsdk/
https://steinbergmedia.github.io/vst3_dev_portal/

Ableton Max for Live:
https://help.ableton.com/hc/en-us/articles/5402681764242-Controlling-Live-using-Max-for-Live
```
