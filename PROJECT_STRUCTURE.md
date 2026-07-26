# Project Structure

Status: official working structure  
Date: 2026-06-02  
Scope: repository layout for spec-driven development, Rust core, CLI-first implementation and future Tauri desktop app

## 1. Zasada nadrzedna

Projekt ma byc ulozony tak, zeby:

1. zaczac od malego, testowalnego core,
2. nie blokowac przyszlej aplikacji desktopowej,
3. nie mieszac researchu z kodem produktu,
4. trzymac specyfikacje obok implementacji jako zrodlo prawdy,
5. moc otworzyc caly folder w VS Code bez chaosu.

Najwazniejsza zasada:

```text
Spec first. Core first. UI later.
```

## 2. Root projektu

Rootem projektu jest:

```text
/Users/tru.siak/Documents/New project/als-rewrite-lab
```

Ten folder mozna pozniej otworzyc w VS Code jako:

```text
File -> Open Folder -> als-rewrite-lab
```

Nie otwieramy osobno `crates/`, `apps/desktop/` albo `experiments/`.

Otwieramy caly projekt.

## 3. Docelowy uklad

```text
als-rewrite-lab/
  README.md

  AGENTS.md
  AI_CONTRACT.md
  PROJECT_NAVIGATOR.md
  CURRENT_STATE.md
  PROJECT_WORKFLOW.md
  PROJECT_STRUCTURE.md
  PROJECT_MAP.md
  TECHNOLOGY_VISION.md
  PRODUCT_OFFICE.md
  PRODUCT_SPINE.md
  PRODUCT_BACKLOG.md
  ALS_REWRITE_METHODOLOGY.md
  ALS_ARCHITECTURE_BLUEPRINTS.md

  intake/
    README.md

  session-digests/
    README.md
    _template.md

  specs/
    001-als-reader/
      spec.md
      plan.md
      tasks.md

  crates/
    rescue_core/
      Cargo.toml
      src/
        lib.rs
        als_reader.rs
        models.rs

    rescue_analyzer/
      Cargo.toml
      src/
        lib.rs

    rescue_rewriter/
      Cargo.toml
      src/
        lib.rs

    rescue_index/
      Cargo.toml
      src/
        lib.rs

  cli/
    rescue-cli/
      Cargo.toml
      src/
        main.rs

  apps/
    desktop/
      src-tauri/
      src/
      package.json

    ableton-bridge/
      README.md

  tests/
    fixtures/
      als/
      expected/

  experiments/
    YYYY-MM-DD_short_name/
      notes.md
      summary.md
      semantic_diff.json

  tools/
    experiments/
      python/
```

Nie wszystko tworzymy od razu.

To jest mapa rozwoju, nie lista rzeczy do zakodowania dzisiaj.

## 4. Co tworzymy teraz

Na teraz potrzebne sa tylko:

```text
specs/
  001-als-reader/

crates/
  rescue_core/

cli/
  rescue-cli/

tests/
  fixtures/

experiments/
```

Nie tworzymy jeszcze:

```text
apps/desktop
apps/ableton-bridge
crates/rescue_rewriter
crates/rescue_index
SQLite schema
Tauri config
```

Te rzeczy wejda wtedy, gdy core bedzie mial stabilny kontrakt.

## 5. Rola glownych folderow

### 5.0 Project control files

Te pliki steruja sposobem pracy w repo:

```text
AGENTS.md
  Instrukcje dla Codexa i przyszlych agentow AI.

AI_CONTRACT.md
  Kontrakt bezpieczenstwa: czego AI nie wolno robic i kiedy ma zatrzymac prace.

CURRENT_STATE.md
  Aktualny stan projektu i najblizszy krok.

PROJECT_NAVIGATOR.md
  Jak prowadzimy sesje: rozmowa, research, spec, kod, review.

PROJECT_STRUCTURE.md
  Oficjalna struktura folderow.

PROJECT_MAP.md
  Mapa domen, odpowiedzialnosci i granic architektonicznych.

PRODUCT_SPINE.md
  Globalna obietnica produktu, use case'y, capability map i bramki od wizji do kodu.
```

Zasada:

```text
AGENTS.md i AI_CONTRACT.md trzeba traktowac jako warstwe ochronna przed kodowaniem.
```

### 5.1 Product Office files

Te pliki zarzadzaja rozmowami, pomyslami i priorytetami:

```text
PRODUCT_OFFICE.md
  Jak przetwarzamy rozmowy na session digest, backlog, spec albo state.

PRODUCT_SPINE.md
  Jak pilnujemy, zeby spec i kod wynikaly z produktu, a nie tylko z lokalnego taska.

PRODUCT_BACKLOG.md
  Co jest teraz, co nastepne, co pozniej, jakie hipotezy i ryzyka czekaja.

intake/
  Surowe rozmowy i materialy, jezeli sa dlugie.

session-digests/
  Przetworzone rozmowy i wnioski.
```

Zasada:

```text
Rozmowa nie musi od razu zmieniac spec.
Najpierw moze stac sie session digest.
```

### 5.2 `specs/`

Tu mieszkaja specyfikacje funkcji/modulow.

Kazdy wiekszy klocek ma swoj folder:

```text
specs/001-als-reader/
specs/002-project-analyzer/
specs/003-package-planner/
specs/004-als-rewriter/
```

Kazdy folder spec ma trzy pliki:

```text
spec.md
  Co ma dzialac, czego nie robimy, acceptance criteria.

plan.md
  Jak to technicznie zrobimy.

tasks.md
  Mala lista zadan do wykonania.
```

Zasada:

```text
Kod powstaje dopiero po spec.md.
```

Nie kazda rozmowa tworzy spec.

Spec tworzymy dopiero, gdy mamy maly, nazwany modul.

### 5.3 `crates/`

Tu mieszkaja biblioteki Rust.

To jest serce produktu.

Pierwszy crate:

```text
crates/rescue_core/
```

Bedzie zawieral:

```text
ALSReader
SampleReference model
basic path model
JSON output model
```

Pozniejsze crates:

```text
rescue_analyzer
  klasyfikacja, dependency graph, preflight

rescue_rewriter
  copy, staging, rewrite ALS, semantic diff

rescue_index
  SQLite, asset index, redirect ledger, reachability
```

Zasada:

```text
Core nie zna UI.
```

Core ma dzialac z CLI, testow i pozniej z Tauri.

### 5.4 `cli/`

Tu mieszka narzedzie terminalowe.

Pierwsza komenda:

```text
rescue analyze path/to/project.als --json
```

CLI jest pierwszym klientem core.

Dlaczego CLI przed UI:

1. latwiej testowac,
2. latwiej debugowac,
3. latwiej porownywac output JSON,
4. nie mieszamy layoutu aplikacji z logika `.als`,
5. ten sam core zasili pozniej Tauri.

### 5.5 `apps/`

Tu beda aplikacje koncowe.

Docelowo:

```text
apps/desktop/
  Tauri + TypeScript UI

apps/ableton-bridge/
  ewentualny Max for Live / JUCE / VST bridge
```

Na teraz `apps/` moze nie istniec albo byc puste.

Nie zaczynamy od UI.

### 5.6 `tests/`

Tu trzymamy testy i fixture.

Fixture to kontrolowane dane testowe:

```text
tests/fixtures/als/
  male albo kontrolowane pliki .als

tests/fixtures/expected/
  oczekiwane JSON-y, liczby i raporty
```

Nie wrzucamy tu przypadkowo wielkich katalogow projektow.

Duze before/after ida do:

```text
experiments/
```

### 5.7 `experiments/`

Tu mieszka laboratorium.

Przyklady:

```text
experiments/2026-05-30_blabla_stemiki_2_before_cas/
experiments/2026-05-31_cas_downloads_wav/
experiments/2026-05-31_duplicate_filenames/
```

Tu moga byc:

```text
snapshoty
before/after CAS
raporty semantic diff
notatki
duze pliki testowe
```

Research nie jest kodem produktu.

Jesli eksperyment potwierdza regule, regule przenosimy do:

```text
ALS_REWRITE_METHODOLOGY.md
```

### 5.8 `tools/experiments/python/`

Tu moga byc szybkie skrypty Pythonowe.

Dozwolone:

```text
one-off parser
diff helper
CAS comparison script
report generator
```

Zakaz:

```text
nie robimy z tego glownego produktu
```

Zasada:

```text
Python discovers.
Rust productizes.
```

## 6. Gdzie trafia nowa rzecz

```text
Nowa rozmowa / metlik
-> Product Office, potem session-digests/ albo CURRENT_STATE.md

Nowa instrukcja dla Codexa
-> AGENTS.md

Nowa obietnica produktu / use case / capability / globalna bramka
-> PRODUCT_SPINE.md

Nowa regula bezpieczenstwa AI / danych
-> AI_CONTRACT.md

Dluga surowa rozmowa
-> intake/

Przetworzona rozmowa
-> session-digests/

Pomysl na pozniej / hipoteza / ryzyko
-> PRODUCT_BACKLOG.md

Nowa zasada prowadzenia pracy
-> PROJECT_NAVIGATOR.md albo PROJECT_WORKFLOW.md

Nowa decyzja technologiczna
-> TECHNOLOGY_VISION.md

Nowa struktura folderow
-> PROJECT_STRUCTURE.md

Nowa granica domeny / odpowiedzialnosci
-> PROJECT_MAP.md

Potwierdzona regula ALS
-> ALS_REWRITE_METHODOLOGY.md

Inspiracja z innej branzy
-> ALS_ARCHITECTURE_BLUEPRINTS.md

Nowa funkcja do zbudowania
-> specs/00x-feature-name/

Kod core
-> crates/

Kod CLI
-> cli/

Kod desktop UI
-> apps/desktop/

Eksperyment Abletona
-> experiments/

Szybki skrypt badawczy
-> tools/experiments/python/

Male dane testowe
-> tests/fixtures/
```

## 7. Spec-driven workflow w tej strukturze

Kazda funkcja przechodzi przez etapy:

```text
1. Conversation / research
2. FACT albo HYPOTHESIS
3. Product Spine traceability
4. Spec folder
5. plan.md
6. tasks.md
7. code
8. tests
9. review
10. update CURRENT_STATE
```

Zasada:

```text
Spec musi wiedziec, ktory use case i capability wspiera.
Jesli tego nie wiemy, nie jest jeszcze gotowa do kodu.
```

Przyklad dla pierwszego modulu:

```text
specs/001-als-reader/spec.md
  ALSReader ma czytac .als i zwracac aktywne SampleRef/FileRef.

specs/001-als-reader/plan.md
  Uzywamy Rust gzip + XML parser + modele serde.

specs/001-als-reader/tasks.md
  1. Stworz rescue_core.
  2. Dodaj model SampleReference.
  3. Dodaj parser gzip/XML.
  4. Dodaj CLI analyze.
  5. Dodaj fixture test.
```

Zasada:

```text
Jesli podczas kodu odkrywamy cos nowego, nie kodujemy dalej na slepo.
Najpierw aktualizujemy spec albo research note.
```

## 8. Kolejnosc implementacji

### Etap 0: teraz

```text
PROJECT_STRUCTURE.md
specs/001-als-reader/
```

### Etap 1: pierwszy core

```text
crates/rescue_core
cli/rescue-cli
tests/fixtures
```

Cel:

```text
rescue analyze path/to/file.als --json
```

### Etap 2: analyzer

```text
crates/rescue_analyzer
```

Cel:

```text
source categories
storage state
basic preflight
```

### Etap 3: package/rewrite

```text
crates/rescue_rewriter
```

Cel:

```text
copy package
minimal rewrite copy
semantic diff
validation
```

### Etap 4: desktop

```text
apps/desktop
```

Cel:

```text
Tauri UI pokazujace prawdziwy output z core.
```

### Etap 5: persistent product

```text
crates/rescue_index
SQLite
```

Cel:

```text
asset index
redirect ledger
reachability
verification history
```

## 9. Zasady anty-chaos

```text
Nie przenosimy istniejacych dokumentow bez powodu.

Nie tworzymy Tauri app, zanim CLI nie ma stabilnego outputu.

Nie tworzymy SQLite, zanim nie potrzebujemy pamieci miedzy sesjami.

Nie tworzymy rewriter, zanim reader/analyzer nie sa sprawdzone.

Nie wrzucamy duzych projektow Abletona do tests/fixtures.

Nie mieszamy Python experiments z Rust product code.

Nie kodujemy funkcji bez spec.md.
```

## 10. Oficjalna decyzja robocza

Od teraz projekt idzie tym torem:

```text
Spec-driven
Rust core
CLI first
Tauri desktop later
SQLite when needed
Python only for experiments
Ableton bridge as future adapter
```

Ta decyzja moze zostac zmieniona, ale tylko przez aktualizacje:

```text
TECHNOLOGY_VISION.md
PROJECT_STRUCTURE.md
CURRENT_STATE.md
```
