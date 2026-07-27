# Project Navigator

Status: working guide  
Date: 2026-06-02  
Scope: a guided workflow that turns conversations, discoveries and tests into a real product

## 1. Po co istnieje ten dokument

Ten dokument jest prowadnica pracy.

Ma odpowiadac na pytania:

```text
Czy teraz rozmawiamy?
Czy teraz testujemy?
Czy teraz piszemy spec?
Czy teraz piszemy kod?
Czy to odkrycie zmienia architekture?
Czy to jest potwierdzona regula, czy tylko hipoteza?
```

To nie jest kolejna specyfikacja produktu.

To jest workflow, ktory pilnuje przechodzenia od chaosu do produktu.

## 2. Rola Nawigatora

W kazdej sesji Codex ma zachowywac sie jak:

```text
Product Navigator + technical partner
```

Nie tylko odpowiadac na pytanie.

Ma tez pilnowac:

1. w jakim trybie teraz jestesmy,
2. czy rozmowa prowadzi do produktu,
3. czy powstal nowy fakt, hipoteza albo decyzja,
4. czy mamy wystarczajaco danych, zeby pisac spec,
5. czy mamy wystarczajaco jasna spec, zeby pisac kod,
6. czy trzeba zatrzymac kodowanie i wrocic do testu.

Jesli rozmowa jest dluga, chaotyczna albo strategiczna, Codex ma przejsc w tryb:

```text
Product Office
```

Tryb Product Office jest opisany w:

```text
PRODUCT_OFFICE.md
PRODUCT_SPINE.md
```

Jego zadanie:

```text
zrobic session digest
oddzielic fakty, hipotezy, pytania, ryzyka i decyzje
zaproponowac miejsca zapisu
sprawdzic, czy wniosek zmienia Product Spine
zapytac przed aktualizacja waznych plikow
```

## 3. Ograniczenie

Codex nie widzi automatycznie rozmow z innych chatow.

Jesli rozmowa dzieje sie w ChatGPT albo innym narzedziu, trzeba ja tu wkleic albo strescic.

Po wklejeniu rozmowy zadanie Nawigatora brzmi:

```text
przeczytaj rozmowe
oddziel szum od decyzji
zapisz wnioski we wlasciwych koszykach
powiedz, jaki jest nastepny krok
```

## 4. Jeden prosty model

Kazdy wniosek trafia do jednego z pieciu statusow:

```text
FACT
  Potwierdzone przez eksperyment, plik, test albo jasna decyzje.

HYPOTHESIS
  Brzmi sensownie, ale nie jest jeszcze udowodnione.

QUESTION
  Tego nie wiemy i trzeba to kiedys sprawdzic.

DECISION
  Wybieramy takie podejscie, nawet jesli istnieja alternatywy.

DEFERRED
  Ciekawy pomysl, ale nie teraz.
```

To jest najwazniejszy mechanizm.

Nie kazda dobra mysl jest wymaganiem.

Nie kazda hipoteza moze sterowac kodem.

Nie kazde pytanie musi byc rozwiazane przed v0.1.

## 5. Tryby pracy

### 5.1 Conversation Mode

Uzywamy, gdy:

```text
czujesz metlik
nie wiesz, ktory kierunek jest wazny
rozmawiamy o produkcie, uzytkowniku, konkurencji albo analogiach
```

Wynik:

```text
FACT / HYPOTHESIS / QUESTION / DEFERRED
```

Zakaz:

```text
nie piszemy kodu z samej rozmowy
```

### 5.2 Research Mode

Uzywamy, gdy:

```text
trzeba sprawdzic, co naprawde robi Ableton
trzeba porownac before/after CAS
trzeba sprawdzic strukture ALS
trzeba potwierdzic hipoteze
```

Wynik:

```text
experiment note
semantic diff
confirmed fact albo rejected hypothesis
```

Zakaz:

```text
nie zamieniamy jednego testu w ogolna regule bez ostroznosci
```

### 5.3 Spec Mode

Uzywamy, gdy:

```text
mamy maly modul
wiemy, co ma robic
wiemy, czego nie ma robic
wiemy, jak sprawdzic wynik
```

Wynik:

```text
requirements
design notes
acceptance criteria
test fixtures
Product Spine traceability
gate status summary
```

Kazda spec musi odpowiedziec:

```text
ktory use case i capability wspiera?
gdzie lezy w przeplywie produktu?
kto jest downstream consumer?
jak to sprawdzimy?
jakie fixture albo eksperymenty sa potrzebne?
jakie decyzje uzytkownika beda potrzebne?
jakie dane, zachowania, ograniczenia i dowody musza przejsc dalej?
co blokuje implementacje?
```

Zakaz:

```text
nie opisujemy calego produktu w jednej specyfikacji
```

### 5.4 Build Mode

Uzywamy, gdy:

```text
spec jest wystarczajaco jasna
zakres jest maly
test albo fixture istnieje
wiadomo, co znaczy "dziala"
```

Wynik:

```text
kod
testy
raport walidacji
diff
```

Zakaz:

```text
nie dodajemy nowych funkcji przy okazji
```

### 5.5 Review Mode

Uzywamy po kodzie albo po duzym eksperymencie.

Wynik:

```text
co sie zmienilo
czy zalozenia sie potwierdzily
czy trzeba zaktualizowac spec
czy mozna isc dalej
```

## 6. Bramka przejscia miedzy trybami

### Conversation -> Research

Przechodzimy, gdy rozmowa zawiera zdanie typu:

```text
nie wiemy, czy Ableton robi X
nie wiemy, czy rewrite X jest bezpieczny
trzeba to sprawdzic na prawdziwym projekcie
```

### Research -> FACT

Przechodzimy, gdy:

```text
mamy before/after
mamy diff
mamy konkretna obserwacje
wiemy, na jakim projekcie i wersji Abletona to sprawdzono
```

### FACT -> Spec

Przechodzimy, gdy:

```text
fakt dotyczy modulu, ktory chcemy budowac teraz
da sie go opisac jako wymaganie albo acceptance criterion
da sie wskazac rodzica w PRODUCT_SPINE.md
da sie wskazac miejsce w flow produktu
```

### Spec -> Build

Przechodzimy, gdy:

```text
wejscie jest jasne
wyjscie jest jasne
zakres jest maly
wiadomo, czego nie wolno robic
jest sposob walidacji
Product Spine gates nie blokuja pracy
```

### Build -> Research

Wracamy, gdy:

```text
test obalil zalozenie
parser znalazl nierozpoznana strukture
rewrite wymaga pola, ktorego nie rozumiemy
matching jest niejednoznaczny
```

## 7. Minimalny rytual kazdej sesji

Na poczatku sesji Nawigator powinien ustalic:

```text
1. Co jest celem tej sesji?
2. W jakim trybie pracujemy?
3. Jakiego pliku/dokumentu dotykamy?
4. Czego dzisiaj nie robimy?
5. Czy praca dotyka Product Spine, spec, czy tylko rozmowy?
```

Na koncu sesji Nawigator powinien powiedziec:

```text
1. Co ustalilismy?
2. Co jest FACT?
3. Co jest HYPOTHESIS?
4. Co zostaje QUESTION?
5. Jakiego testu albo dowodu potrzebujemy?
6. Jaki jest nastepny maly krok?
7. Czy trzeba zaktualizowac Product Spine, backlog albo spec?
```

## 7A. Product Spine Gates

Przed stworzeniem spec, akceptacja spec albo kodowaniem modulu Codex musi
sprawdzic bramki z:

```text
PRODUCT_SPINE.md
```

Minimalny zestaw odpowiedzi:

```text
Parent Product Capability:
  Jaka zdolnosc produktu budujemy?

Supported Use Case:
  Dla ktorego przypadku uzycia jest ten modul?

Operation Flow Position:
  W ktorym miejscu przeplywu produktowego modul dziala?

Downstream Consumers:
  Kto uzyje danych albo zachowania tego modulu pozniej?

Gate Status Summary:
  Vision / Use Case / Flow / Data / Behavior / Decision /
  Safety / Audit / Reversibility / Batch / Evidence / Module
```

Statusy:

```text
CLEAR
PARTIAL
AMBIGUOUS
CONFLICTING
UNKNOWN
```

Regula:

```text
AMBIGUOUS, CONFLICTING albo wazne UNKNOWN blokuje spec/build.
```

Wyjatek:

```text
Mozna isc dalej tylko, jesli zakres zostaje jawnie zwezony,
a ograniczenie trafia do spec.
```

## 7B. Clarification Request

Jesli bramka blokuje prace, Codex nie powinien dopisywac lokalnego warunku
na slepo.

Powinien wrocic do usera z krotkim zapytaniem:

```text
Clarification Request

Blocking gate:
  nazwa bramki

What is unclear:
  konkretna niejasnosc

Why it matters:
  co moze sie popsuc w produkcie, jezeli zgadniemy

Decision needed:
  jedna decyzja albo pytanie do usera

Temporary safe scope:
  co mozemy zrobic bez tej decyzji, jesli cokolwiek
```

## 7C. Misalignment Review

Jesli user mowi, ze modul albo spec moze nie pasowac do produktu, Codex ma
zatrzymac implementacje i zrobic review przeplywu:

```text
1. Jaka wiedza produktowa byla w rozmowie?
2. Gdzie powinna byla zostac zapisana?
3. Ktora bramka powinna byla zatrzymac prace?
4. Czy problem dotyczy danych, zachowania, decyzji, bezpieczenstwa,
   walidacji, manifestu, rewersyjnosci, batcha czy UX?
5. Jaka globalna zasada naprawia klase bledu, a nie tylko ten przypadek?
```

To review ma prowadzic do aktualizacji:

```text
PRODUCT_SPINE.md
PROJECT_NAVIGATOR.md
AGENTS.md
spec
```

Nie do szybkiego dopisania lokalnego if-a w kodzie.

## 8. Jak uzywac tego z Codexem

Mozesz zaczynac wiadomosc tak:

```text
Nawigator: przeczytaj to i powiedz, w jakim trybie jestesmy.
```

Albo:

```text
Nawigator: mam metlik, uporzadkuj to bez kodowania.
```

Albo:

```text
Nawigator: czy to jest juz material na spec, czy jeszcze hipoteza?
```

Albo:

```text
Nawigator: zamien ta rozmowe w fakty, hipotezy, pytania i nastepny krok.
```

Albo:

```text
Nawigator: pilnuj mnie, zebym nie zaczal kodowac za wczesnie.
```

Albo:

```text
Product Office: przetworz te rozmowe. Zrob digest i zaproponuj, co zapisac gdzie.
```

Albo:

```text
Product Office: powiedz, co jest MVP, co pozniej, co jest hipoteza, a co wymaga testu.
```

## 9. Jak uzywac tego z VS Code

VS Code nie jest wymagany, ale moze pomoc jako mapa projektu.

Najprostszy setup:

```text
Open Folder:
$WORKSPACE
```

W VS Code korzystasz glownie z:

```text
Explorer
  zeby widziec pliki projektu

Search
  zeby znalezc, gdzie zapisano dana decyzje

Markdown Preview
  zeby czytac dokumenty wygodniej

Source Control
  zeby zobaczyc diff zmian

Terminal
  zeby uruchamiac testy i komendy
```

Nie trzeba znac calego VS Code.

Na poczatku wystarczy:

```text
otworz folder
czytaj markdown
patrz na diff
```

## 10. Gdzie zapisywac rzeczy

Na razie nie mnozymy plikow.

Uzywamy:

```text
AGENTS.md
  Jak Codex ma pracowac w tym repo.

AI_CONTRACT.md
  Czego AI nie wolno robic i jakie sa warunki bezpieczenstwa.

PRODUCT_OFFICE.md
  Jak przetwarzamy dlugie rozmowy na digest, backlog, spec albo state.

PRODUCT_SPINE.md
  Globalna obietnica produktu, use case'y, capability map i bramki od wizji do kodu.

PRODUCT_BACKLOG.md
  Co jest teraz, co nastepne, co pozniej, jakie hipotezy i ryzyka czekaja.

PROJECT_NAVIGATOR.md
  Jak prowadzimy sesje.

CURRENT_STATE.md
  Gdzie teraz jestesmy i jaki jest nastepny krok.

PROJECT_WORKFLOW.md
  Ogolny workflow projektu.

PROJECT_STRUCTURE.md
  Oficjalna struktura folderow.

PROJECT_MAP.md
  Mapa domen, odpowiedzialnosci i miejsc, gdzie logika moze mieszkac.

TECHNOLOGY_VISION.md
  Decyzje technologiczne.

ALS_REWRITE_METHODOLOGY.md
  Potwierdzone zasady rewrite ALS.

ALS_ARCHITECTURE_BLUEPRINTS.md
  Inspiracje i architektura przyszlosci.

session-digests/
  Przetworzone rozmowy.

intake/
  Surowe dlugie materialy do przetworzenia.
```

Jesli cos jest tylko rozmowa:

```text
session-digests/ albo CURRENT_STATE.md, zaleznie od wagi
```

Jesli cos jest potwierdzona regula techniczna:

```text
ALS_REWRITE_METHODOLOGY.md
```

Jesli cos jest inspiracja z innych branz:

```text
ALS_ARCHITECTURE_BLUEPRINTS.md
```

Jesli cos jest instrukcja prowadzenia pracy:

```text
PROJECT_NAVIGATOR.md
```

## 11. Jak Nawigator decyduje, czy cos idzie w produkt

Pytania kontrolne:

```text
Czy to rozwiazuje realny problem usera?
Czy dotyczy najblizszego zakresu?
Czy mamy dowod albo test?
Czy da sie to opisac jako mala funkcje?
Czy umiemy powiedziec, kiedy dziala?
Czy umiemy powiedziec, kiedy ma odmowic?
```

Jesli odpowiedz brzmi "nie" przy kilku pytaniach:

```text
to nie jest jeszcze produkt
```

Wtedy zostaje jako:

```text
HYPOTHESIS
QUESTION
DEFERRED
```

## 12. Tutorial przejscia od metliku do kodu

### Krok 1: Zrzut mysli

User moze pisac chaotycznie.

To jest ok.

Przyklad:

```text
Nie wiem, czy ALSReader powinien od razu klasyfikowac Splice i User Library.
Nie wiem, czy to juz spec.
Boje sie, ze zaczniemy kod za szybko.
```

### Krok 2: Nawigator klasyfikuje

```text
FACT:
- ALSReader ma czytac aktywne SampleRef/FileRef.

HYPOTHESIS:
- ALSReader moze od razu nadawac source_category.

QUESTION:
- Czy source_category nalezy do ALSReader, czy do ProjectAnalyzer?

NEXT:
- Nie kodujemy. Najpierw robimy spec 001 ALSReader z granicami odpowiedzialnosci.
```

### Krok 3: Spec tylko dla malego klocka

```text
ALSReader robi:
- gzip read
- XML parse
- extract active SampleRef/FileRef

ALSReader nie robi:
- matching
- copying
- rewriting
- delete
```

### Krok 4: Test/fixture

```text
Wejscie:
cziki.als

Oczekiwane:
153 SampleRef
33 external active refs
120 Core refs
0 changes to file
```

### Krok 5: Kod

Dopiero teraz:

```text
implement ALSReader
run tests
compare output
```

### Krok 6: Review

```text
Czy kod zrobil tylko to, co bylo w spec?
Czy odkryl cos nowego?
Czy spec trzeba poprawic?
Czy mozemy przejsc do nastepnego klocka?
```

## 13. Aktualny najblizszy tor

Najblizszy tor projektu:

```text
1. uporzadkowac stan projektu
2. stworzyc spec 001 ALSReader
3. wybrac 2-3 znane ALS jako fixture
4. napisac minimalny czytnik
5. wygenerowac JSON z aktywnymi SampleRef/FileRef
6. porownac liczby z naszymi dotychczasowymi analizami
```

Nie robimy jeszcze:

```text
UI
delete
safe cleanup
Windows
cross-platform rewrite
plugin rewrite
batch rewrite wielu projektow
```

Ale:

```text
macOS first nie oznacza macOS-only core.
Nie implementujemy teraz Windows migration, ale nie wolno bez potrzeby blokowac
tej drogi w kontraktach i module core.
```

## 14. Najwazniejsza obietnica

Nawigator ma chronic projekt przed dwoma skrajnosciami:

```text
wieczne gadanie bez produktu
```

i

```text
kodowanie bez pewnosci, co budujemy
```

Dlatego przy kazdej wiekszej rozmowie powinien umiec powiedziec:

```text
Teraz jestesmy w Conversation Mode.
To sa fakty.
To sa hipotezy.
To sa pytania.
To nie wchodzi do v0.1.
Nastepny krok to test/spec/kod.
```

## 15. Zdanie sterujace

Jesli czujesz metlik, uzyj tego zdania:

```text
Nawigator: zatrzymajmy sie. Powiedz, co jest faktem, co hipoteza, co pytaniem, i jaki jest jeden nastepny krok.
```
