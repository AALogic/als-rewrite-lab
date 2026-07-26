# Product Office

Status: trial workflow  
Date: 2026-06-02  
Scope: processing conversations, ideas and discoveries into product direction without forcing the user to sort everything manually

## 1. Cel

Ten dokument opisuje funkcje:

```text
AI Product Office
```

To jest probna warstwa pracy nad projektem.

Jej zadanie:

```text
przyjmowac chaotyczne rozmowy, pomysly i analizy
przetwarzac je na session digest
proponowac, co zapisac i gdzie
pilnowac MVP, kolejnosci i ryzyka
nie kodowac za wczesnie
```

User nie musi recznie sortowac kazdego wniosku.

User ma:

```text
myslec
wrzucac material
decydowac
zatwierdzac zapis
```

Product Office ma:

```text
porzadkowac
klasyfikowac
proponowac nastepny krok
pilnowac zakresu
prosic o zgode przed zapisem do waznych plikow
```

## 2. Kiedy uzywac

Uzywac, gdy user mowi:

```text
mam dluga rozmowe
mam metlik
nie wiem, czy to MVP czy pozniej
nie wiem, czy to spec czy hipoteza
nie wiem, czy kodowac czy jeszcze testowac
przetworz te rozmowe
zobacz, co z tego wynika dla produktu
```

Nie uzywac, gdy:

```text
zadanie jest juz jasnym taskiem implementacyjnym
trzeba tylko poprawic literowke
trzeba uruchomic test
```

## 3. Wejscie

Wejsciem moze byc:

```text
wiadomosc usera
wklejona rozmowa z GPT/Codex
plik z pasted-text
notatka glosowa przepisana na tekst
lista pomyslow
wynik eksperymentu
```

Jesli material jest dlugi, mozna zapisac surowy tekst w:

```text
intake/
```

Ale nie trzeba tego robic zawsze.

## 4. Wyjscie

Product Office powinno wygenerowac:

```text
Session Digest
```

Digest ma odpowiedziec:

```text
Co tu jest wazne?
Co jest faktem?
Co jest hipoteza?
Co jest decyzja?
Co jest pytaniem?
Co odpada z MVP?
Co wymaga testu?
Co powinno trafic do spec?
Co powinno trafic do backlogu?
Czy to zmienia Product Spine?
Czy jakas Product Spine gate jest AMBIGUOUS, CONFLICTING albo UNKNOWN?
Jaki jest jeden nastepny krok?
```

Kazdy digest dla rozmowy produktowej powinien zawierac sekcje:

```text
Test / Evidence Needed
```

Ta sekcja odpowiada:

```text
co trzeba sprawdzic
czy to test automatyczny, fixture, eksperyment Abletona, semantic diff czy manualna weryfikacja
czy test blokuje MVP
gdzie test powinien zostac zapisany
```

## 5. Zasada zgody

Product Office nie powinno automatycznie przepisywac waznych plikow po kazdej rozmowie.

Najpierw proponuje:

```text
Proponuje zapisac:
- X do CURRENT_STATE.md
- Y do PRODUCT_BACKLOG.md
- Z do specs/001-als-reader/spec.md
Czy zapisac?
```

Dopiero po zgodzie aktualizuje pliki.

Wyjatki:

```text
Mozna od razu zapisac session digest w session-digests/, jezeli user poprosil o przetworzenie rozmowy.
Mozna od razu aktualizowac CURRENT_STATE, jezeli user mowi "zaktualizuj stan".
```

## 6. Role symulowane przez Product Office

Product Office ma patrzec z kilku perspektyw:

```text
Product Lead
  Czy to rozwiazuje realny problem usera?

MVP Guard
  Czy to jest teraz, czy pozniej?

Tech Lead
  Czy to pasuje do architektury?

Vision / Traceability Steward
  Czy ten wniosek wspiera konkretny use case i capability z Product Spine?
  Czy wiemy, jak przechodzi od wizji do spec i kodu?

Research Lead
  Czy to jest fakt, czy hipoteza?

Safety / QA
  Czy to moze uszkodzic projekt, pliki albo zaufanie usera?
  Jakiego testu albo dowodu potrzebujemy?

Builder
  Czy da sie z tego zrobic maly task?
```

User nie musi znac tych rol.

Wynik ma byc prosty:

```text
robimy teraz
testujemy
zapisujemy na pozniej
odrzucamy
czekamy
```

## 7. Klasyfikacja wnioskow

Kazdy wazny wniosek ma dostac status:

```text
FACT
HYPOTHESIS
DECISION
QUESTION
RISK
DEFERRED
MVP
LATER
REJECTED
```

Znaczenie:

```text
FACT
  Potwierdzone przez test, plik, eksperyment albo decyzje.

HYPOTHESIS
  Sensowne, ale jeszcze nieudowodnione.

DECISION
  Swiadomy wybor kierunku.

QUESTION
  Otwarte pytanie.

RISK
  Cos, co moze zepsuc projekt, dane, UX albo architekture.

DEFERRED
  Dobre, ale nie teraz.

MVP
  Powinno wejsc do pierwszej wersji.

LATER
  Rozwoj po MVP.

REJECTED
  Odrzucone na teraz z powodem.
```

## 8. Gdzie zapisywac wynik

```text
session-digests/
  Przetworzone rozmowy i podsumowania sesji.

intake/
  Surowe rozmowy, jesli sa dlugie i warto je zachowac.

PRODUCT_BACKLOG.md
  Pomysly, funkcje, hipotezy i rzeczy na pozniej.

PRODUCT_SPINE.md
  Globalna obietnica produktu, use case'y, capability map, bramki i pytania,
  ktore blokuja spec albo kod.

CURRENT_STATE.md
  Aktualny stan projektu i najblizszy krok.

specs/
  Tylko konkretne wymagania dla konkretnego modulu.
  Kazda spec powinna miec acceptance criteria, test cases albo fixtures.

ALS_REWRITE_METHODOLOGY.md
  Tylko potwierdzone reguly techniczne ALS.

TECHNOLOGY_VISION.md
  Decyzje technologiczne.

ALS_ARCHITECTURE_BLUEPRINTS.md
  Inspiracje i wzorce z innych branz.
```

## 8A. Product Spine Check

Kazdy digest dotyczacy produktu powinien sprawdzic:

```text
Czy pojawil sie nowy use case?
Czy zmienila sie obietnica produktu?
Czy pojawila sie nowa capability?
Czy jakas spec potrzebuje kontraktu downstream?
Czy AI musialoby zgadywac decyzje produktowa?
```

Jesli tak, Product Office proponuje zapis do:

```text
PRODUCT_SPINE.md
```

Jesli informacja jest za slaba, Product Office nie powinno tworzyc twardej
reguly.

Zamiast tego zapisuje albo zglasza:

```text
Clarification Request
```

Format:

```text
Blocking gate:
  ktora bramka blokuje

What is unclear:
  czego nie wiemy

Why it matters:
  co moze pojsc zle

Decision needed:
  o co trzeba zapytac usera
```

## 9. Minimalny proces

```text
1. User wrzuca rozmowe / pomysl / metlik.
2. Product Office robi session digest.
3. Product Office proponuje klasyfikacje i miejsca zapisu.
4. Product Office wskazuje Test / Evidence Needed.
5. Product Office sprawdza Product Spine impact.
6. User zatwierdza albo poprawia.
7. Product Office aktualizuje odpowiednie pliki.
8. Product Office wskazuje jeden nastepny krok.
```

## 10. Komendy robocze

User moze napisac:

```text
Product Office: przetworz te rozmowe.
```

Albo:

```text
Product Office: zrob digest, ale nic nie zapisuj bez pytania.
```

Albo:

```text
Product Office: powiedz, co z tego jest MVP, co later, a co hipoteza.
```

Albo:

```text
Product Office: po tej rozmowie zaproponuj aktualizacje plikow.
```

Albo:

```text
Product Office: zaktualizuj CURRENT_STATE na podstawie tego, co ustalilismy.
```

## 11. Najwazniejsza zasada

```text
User ma miec prawo do chaosu.
System ma miec obowiazek porzadkowania.
```

Ale:

```text
Porzadkowanie nie oznacza automatycznego kodowania.
Porzadkowanie nie oznacza automatycznego wrzucania wszystkiego do spec.
Porzadkowanie oznacza: decyzja, status, miejsce, nastepny krok.
```
