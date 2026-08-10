# Project Workflow

Status: historical workflow reference; superseded by `AGENTS.md`
Date: 2026-05-31  
Scope: how to move from conversations and experiments to specs and code without chaos

> Use `CURRENT_STATE.md` for the present task, module contracts for executable
> gates, and `ENGINEERING_RULES.md` for implementation quality. This file is
> retained as background and does not define the current build order.

## 1. Najprostsza zasada

Nie probujemy od razu opisac calego produktu.

Pracujemy tak:

```text
rozmowa -> notatka -> decyzja albo hipoteza -> spec -> kontrakt -> test -> kod
```

Kazda rozmowa z GPT/Codexem powinna na koncu trafic do jednego z czterech koszykow:

```text
confirmed rule
hypothesis
known unknown
deferred idea
```

Jesli rozmowa nie trafia do zadnego koszyka, to byla ciekawa, ale nie steruje projektem.

## 2. Co robic teraz, a czego jeszcze nie robic

Na tym etapie nie instalowac ciezkiego workflow, jesli to tylko zwieksza zamieszanie.

Na teraz wystarczy:

```text
Codex
Markdown files
terminal
git diff
real Ableton test projects
```

VS Code jest opcjonalny.

Instalowac VS Code dopiero wtedy, gdy:

1. bedziesz chcial wygodnie czytac wiele plikow naraz,
2. bedziesz chcial przegladac diffy wizualnie,
3. zaczniemy pisac wiecej kodu niz dokumentacji,
4. bedziesz chcial recznie edytowac specyfikacje.

Brak VS Code nie blokuje pracy.

## 3. Trzy tryby pracy

### 3.1 Tryb rozmowy

Uzywac, gdy pytanie brzmi:

```text
czy to ma sens?
jak myslec o problemie?
jakie sa ryzyka?
jak robia to inne branze?
```

Efekt rozmowy:

```text
idea
hypothesis
known unknown
architecture note
```

Rozmowa nie powinna bezposrednio konczyc sie kodem.

### 3.2 Tryb research / eksperyment

Uzywac, gdy pytanie brzmi:

```text
co Ableton faktycznie robi?
czy minimalny rewrite dziala?
jak CAS zmienia ALS?
czy dany przypadek jest bezpieczny?
```

Efekt researchu:

```text
experiment folder
before/after files
semantic diff
observed behavior
rule candidate
```

Research moze potwierdzic regule albo pokazac, ze czegos jeszcze nie wiemy.

### 3.3 Tryb product / kod

Uzywac, gdy pytanie brzmi:

```text
czy mamy potwierdzona regule?
czy mamy spec dla malego modulu?
czy wiemy, jak to przetestowac?
```

Efekt kodowania:

```text
small module
tests
module.contract.json
workflow_guard result
manifest/diff output
updated spec if behavior changed
```

Kodujemy tylko to, co ma wystarczajaco jasne reguly.

### 3.4 Tryb guarded build

Uzywac od modulu 002 wzwyz.

```text
spec.md
-> plan.md
-> tasks.md
-> fixture-contract.md
-> module.contract.json
-> ENGINEERING_RULES.md reference
-> workflow_guard module-ready
-> tests first where practical
-> implementation
-> workflow_guard verify-module
-> closeout
```

Markdown opisuje sens i decyzje. Guard jako kod sprawdza, czy warunki wejscia,
test obligations, engineering quality obligations, zakazane pola, downstream
smoke testy i closeout sa obecne.

Od guard v0.2 `verify-module` sprawdza tez wybrane problemy typowe dla AI
codingu:

```text
obligations obecne, ale nieodhaczone
testy obecne tylko z nazwy
testy ignored / puste / tautologiczne
dodatkowe pola publiczne poza kontraktem
niezgodne typy publicznych pol
scope creep w nazwach funkcji
ryzykowne wzorce w core source
nowe zaleznosci spoza allowlisty
ukryte niewiadome w spec/plan/tasks
```

## 4. Kiedy rozmawiac dalej

Rozmawiac dalej, gdy:

```text
nie wiadomo, jaki problem rozwiazujemy
nie wiadomo, dla kogo jest funkcja
nie wiadomo, czy to v0.1 czy przyszlosc
pojawia sie nowa analogia z innej branzy
decyzja ma duzy wplyw na architekture
```

Nie rozmawiac dalej, gdy rozmowa powtarza te same wnioski.

Wtedy trzeba przeniesc wniosek do pliku i isc dalej.

## 5. Kiedy testowac Abletona

Testowac Abletona, gdy:

```text
nowa regula mialaby zmieniac ALS
nie wiemy, ktore pola Ableton zmienia po CAS
nie wiemy, czy projekt otworzy sie po naszym rewrite
istnieje ryzyko wrong sample match
dotykamy nowego typu zrodla: Downloads, Splice, User Library, MP3, duplicate names
```

Nie trzeba robic wszystkich mozliwych testow przed startem.

Robimy tylko testy potrzebne dla najblizszej wersji.

## 6. Kiedy pisac specyfikacje

Specyfikacje piszemy dopiero wtedy, gdy mamy maly, nazwany modul.

Dobra nazwa modulu:

```text
ALSReader
SampleReferenceExtractor
ProjectAnalyzer
PackagePlanner
ALSRewriter
SemanticDiff
```

Zla nazwa modulu:

```text
Ableton rescue system
everything manager
smart fixer
cleanup tool
```

Spec nie opisuje calego produktu.

Spec opisuje jeden kawalek, ktory da sie przetestowac.

## 7. Kiedy aktualizowac pliki

### Po rozmowie

Jesli powstala tylko inspiracja:

```text
ALS_ARCHITECTURE_BLUEPRINTS.md
```

Jesli powstala zasada pracy:

```text
PROJECT_WORKFLOW.md
```

Jesli powstala potwierdzona regula ALS:

```text
ALS_REWRITE_METHODOLOGY.md
```

### Po eksperymencie

Zapisac:

```text
experiments/YYYY-MM-DD_short_name/
  summary.md
  semantic_diff.json
  notes.md
```

Jesli eksperyment potwierdzil regule, dopiero wtedy przeniesc ja do:

```text
ALS_REWRITE_METHODOLOGY.md
```

### Po zmianie kodu

Aktualizowac:

```text
spec for changed module
tests
decision log, only if decision is architectural
```

Nie aktualizowac wszystkich dokumentow naraz.

## 8. Minimalny zestaw dokumentow na teraz

Na tym etapie wystarcza trzy glowne dokumenty:

```text
ALS_REWRITE_METHODOLOGY.md
  Co wiemy o bezpiecznym rewrite ALS.

ENGINEERING_RULES.md
  Jak wolno projektowac, pisac, testowac i refaktoryzowac kod.

ALS_ARCHITECTURE_BLUEPRINTS.md
  Co pozyczamy z innych branz i co moze wejsc pozniej.

PROJECT_WORKFLOW.md
  Jak prowadzimy projekt, zeby sie nie zgubic.
```

Nie tworzyc jeszcze wielu plikow typu:

```text
AI_CONTRACT.md
PRODUCT.md
TECH.md
STRUCTURE.md
DECISIONS.md
```

Mozna je dodac pozniej, gdy zacznie powstawac prawdziwy kod.

## 9. Nazewnictwo

Uzywac prostych nazw.

Dokumenty glowne:

```text
ALS_REWRITE_METHODOLOGY.md
ALS_ARCHITECTURE_BLUEPRINTS.md
PROJECT_WORKFLOW.md
```

Eksperymenty:

```text
experiments/2026-05-31_cas_downloads_wav/
experiments/2026-05-31_duplicate_filenames/
experiments/2026-05-31_splice_cache/
```

Specyfikacje:

```text
specs/001-als-reader/
specs/002-project-analyzer/
specs/003-package-planner/
specs/004-als-rewriter/
```

Nie zmieniac nazw starych plikow bez powodu.

## 10. Prosty cykl dnia pracy

Kazda sesja pracy powinna miec jeden cel.

Przyklad:

```text
Dzisiejszy cel:
Sprawdzic, czy ALSReader potrafi policzyc aktywne SampleRef w trzech znanych plikach.
```

Cykl:

```text
1. wybierz jeden cel
2. przeczytaj odpowiedni dokument
3. zdecyduj: rozmowa, eksperyment, spec czy kod
4. wykonaj maly krok
5. zapisz wynik
6. nie zaczynaj drugiego duzego watku w tej samej sesji
```

## 11. Prosta tabela decyzyjna

```text
Mam nowy pomysl
-> wpisz jako hypothesis albo blueprint, nie koduj

Mam potwierdzenie z testu Abletona
-> wpisz do methodology jako confirmed rule

Mam modul do zbudowania
-> zrob mala specyfikacje modulu

Mam spec i test fixture
-> mozna kodowac

Mam odkrycie podczas kodowania
-> zatrzymaj sie, zaktualizuj spec albo research notes

Mam niejednoznaczny matching
-> blokuj auto-rewrite

Mam pomysl na usuwanie plikow
-> tylko research, bez delete w produkcie
```

## 12. Najblizszy sensowny krok

Najblizszy krok nie powinien byc duzy.

Nie:

```text
zbuduj cala aplikacje
```

Tak:

```text
Zrob spec 001 dla ALSReader.
Wejscie: sciezka do .als.
Wyjscie: JSON z aktywnymi SampleRef/FileRef.
Zakaz: zadnego kopiowania, zadnego rewrite.
Walidacja: liczby z naszych znanych fixture.
```

To jest pierwszy realny most od dokumentow do kodu.

## 13. Najwazniejsze zdanie

Nie musisz wiedziec, dokad prowadzi caly produkt.

Musisz tylko wiedziec:

```text
co jest potwierdzone,
co jest hipoteza,
co jest nastepnym malym krokiem,
i czego dzisiaj nie dotykamy.
```
