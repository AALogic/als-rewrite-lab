# ALS Architecture Blueprints

Status: architectural comparison based on external product patterns  
Date: 2026-05-30  
Scope: patterns to borrow from mature dependency-management tools

## 1. Cel dokumentu

Ten dokument odpowiada na pytanie:

```text
Co z dojrzalych narzedzi z innych branz powinnismy przeniesc do naszego systemu Ableton dependency management?
```

Nie chodzi o kopiowanie funkcji 1:1.

Chodzi o wybranie wzorcow architektonicznych, ktore:

1. zmniejszaja ryzyko uszkodzenia projektow,
2. pomagaja userowi podejmowac decyzje,
3. robia z narzedzia cos wiecej niz jednorazowy rescue,
4. tworza podwaliny pod bezpieczne czyszczenie i utrzymanie biblioteki.

## 2. Obecna architektura ALS

Obecna metodologia ma juz mocny rdzen:

```text
Preflight
-> Read ALS
-> Extract active SampleRefs
-> Classify refs
-> Build plan
-> Copy files if needed
-> Rewrite ALS copy
-> Validate semantic diff
-> User opens in Ableton
-> Mark verified
```

To odpowiada klasycznemu schematowi:

```text
scan -> plan -> apply -> validate -> manifest
```

Najwieksza luka nie jest w samym rewrite.

Najwieksza luka jest w warstwie dlugoterminowej:

```text
Jak system pamieta decyzje usera?
Jak wie, ktore pliki sa dalej potrzebne?
Jak pomaga utrzymac porzadek miesiac pozniej?
Jak usuwa pliki bez ryzyka?
Jak radzi sobie z tym, ze wiele projektow uzywa tych samych sampli?
```

## 3. Blueprint 1: Unreal Redirectors

### Co robi dojrzaly system

Unreal Engine po przeniesieniu albo zmianie nazwy assetu zostawia redirector w starej lokalizacji. Stare referencje moga wtedy odnalezc asset w nowym miejscu. Pozniej `Fix Up Redirectors` przepisuje referencje i usuwa redirector, jezeli wszystkie zalezne pakiety zostaly poprawnie zapisane.

### Co pozyczyc

W naszym systemie powinien powstac:

```text
Path Redirect Ledger
```

To nie musi byc prawdziwy plik w starym miejscu. Na poczatku wystarczy baza/mapa:

```text
old_path -> asset_id -> new_path
```

Przyklad:

```text
/Users/me/Downloads/kick.wav
  -> sha256:abc123
  -> /Users/me/Audio Library/Drums/Kicks/kick.wav
```

### Dlaczego to wazne

Bez redirect ledger user musi od razu naprawic wszystkie projekty.

Z redirect ledger moze:

1. uporzadkowac biblioteke dzisiaj,
2. zachowac wiedze o starych sciezkach,
3. naprawiac projekty partiami,
4. widziec, ktore stare sciezki sa juz bezpiecznie przepiete,
5. dopiero pozniej sprzatac stare lokalizacje.

### Wplyw na architekture

Dodac modul:

```text
redirect_ledger
```

Minimalny model:

```text
redirect_id
old_path
old_project_root
asset_id
new_path
created_at
created_by_operation
verified_project_ids
status: proposed | active | fixed_up | obsolete
```

## 4. Blueprint 2: Premiere / Final Cut Consolidate

### Co robi dojrzaly system

Narzedzia video rozrozniaja media linkowane i media zarzadzane. Premiere Project Manager potrafi zebrac media uzyte w wybranych sekwencjach do jednej lokalizacji i utworzyc nowy projekt linkujacy do skopiowanych plikow. Final Cut Pro pozwala konsolidowac media z wielu lokalizacji do aktualnej lokalizacji biblioteki; dokumentacja podkresla reguly copy/move, ktore maja zapobiegac zerwaniu linkow w innych bibliotekach.

### Co pozyczyc

Potrzebujemy jawnego modelu:

```text
Media Storage State
```

Kategorie:

```text
external_linked
project_managed
library_managed
factory_managed
user_library_managed
missing
ambiguous
unsupported
```

Przyklad:

```text
Downloads/kick.wav                  -> external_linked
Project/Samples/Imported/kick.wav   -> project_managed
Audio Library/Drums/Kicks/kick.wav  -> library_managed
Ableton Core Library/...            -> factory_managed
```

### Dlaczego to wazne

User nie pyta tylko:

```text
Gdzie sa pliki?
```

User potrzebuje odpowiedzi:

```text
Czy ten plik jest bezpiecznie zarzadzany?
Czy jest tylko przypadkowo podlinkowany z Downloads?
Czy moge go przeniesc?
Czy moge go usunac?
Czy powinien zostac w projekcie, czy w centralnej bibliotece?
```

### Wplyw na architekture

Dodac do `SampleReference`:

```text
storage_state
management_owner
safe_to_move
safe_to_delete_candidate
```

Dodac do planu operacji:

```text
scope
media_classes
destination_policy
copy_or_link_policy
```

## 5. Blueprint 3: Blender External Data

### Co robi dojrzaly system

Blender ma osobne akcje:

```text
Pack Resources
Unpack Resources
Make Paths Relative
Make Paths Absolute
Report Missing Files
Find Missing Files
```

To rozdziela kilka problemow, ktore latwo pomylic:

1. czy plik jest wbudowany/spakowany,
2. czy sciezka jest absolutna czy relatywna,
3. czy plik istnieje,
4. czy system ma znalezc brakujace pliki,
5. czy po znalezieniu ma przepisac sciezki.

### Co pozyczyc

Nie robic jednej funkcji "napraw projekt".

Zrobic osobne akcje:

```text
Audit External Files
Find Missing Samples
Make Project Self-Contained
Relocate Self-Contained Project
Convert External Paths To Project Paths
Convert Project Paths To Library Paths
```

### Dlaczego to wazne

Kazda akcja ma inny poziom ryzyka.

`Audit` jest bezpieczny.

`Find Missing Samples` moze byc bezpieczny, ale matching moze byc niejednoznaczny.

`Make Project Self-Contained` kopiuje pliki.

`Rewrite ALS` zmienia projekt.

`Delete Originals` jest najgrozniejsze i musi byc osobna operacja.

### Wplyw na architekture

Dodac `OperationType`:

```text
audit
find_missing
collect_package
relocate
rewrite_paths
fix_redirects
quarantine
delete_unreachable
```

Kazdy typ operacji ma miec osobne reguly preflight i walidacji.

## 6. Blueprint 4: InDesign Preflight + Package

### Co robi dojrzaly system

InDesign przed przekazaniem projektu robi preflight: sprawdza fonty, linki, obrazy, problemy i dopiero potem tworzy paczke z dokumentem, linkami, fontami i raportem.

Wazny wzorzec:

```text
Package contains both files and report.
```

Drugi wazny wzorzec:

```text
If packaging fails, rollback should leave no half-created package.
```

### Co pozyczyc

Nasze narzedzie powinno generowac:

```text
Ableton Project Preflight Report
```

Przyklad sekcji:

```text
Missing audio
External audio
Project-managed audio
Library-managed audio
Core Library refs
Plugin refs
Ambiguous matches
Duplicate filenames
Duplicate content
Unsupported refs
Will copy
Will rewrite
Will leave unchanged
Cannot fix automatically
```

### Dlaczego to wazne

To odpowiada na pytanie usera:

```text
Co mam teraz zrobic?
```

Nie tylko pokazujemy graf zaleznosci.

Pokazujemy decyzje:

```text
Copy these
Rewrite these
Keep these
Review these
Do not touch these
```

### Wplyw na architekture

Dodac modul:

```text
preflight_reporter
```

Dodac severity:

```text
info
warning
needs_review
blocked
danger
```

Dodac transakcje:

```text
staging_dir -> validate -> commit_dir
rollback_on_error
```

## 7. Blueprint 5: SolidWorks Pack and Go

### Co robi dojrzaly system

Pack and Go zbiera model oraz zalezne pliki do folderu albo zipa. Wazne jest to, ze nie wszystkie zaleznosci sa takie same: czesci, rysunki, tabele, materialy, sceny, wyniki symulacji. Narzedzie daje opcje wlaczania i wylaczania klas zaleznosci.

### Co pozyczyc

Potrzebujemy `Dependency Class`.

Kategorie dla Abletona:

```text
active_audio
historical_source_audio
ableton_core_library
ableton_user_library
plugin_binary
plugin_preset
asd_sidecar
backup_als
frozen_audio
recorded_audio
processed_audio
unknown
```

### Dlaczego to wazne

Nie kazda zaleznosc powinna byc traktowana tak samo.

Na przyklad:

```text
active_audio          -> mozna kopiowac i przepisywac
ableton_core_library  -> zostawic
plugin_binary         -> raportowac, nie kopiowac
asd_sidecar           -> kopiowac opcjonalnie
backup_als            -> decyzja usera
historical_source     -> zachowac jako provenance, nie przepisywac w v0.1
```

### Wplyw na architekture

Dodac do `SampleReference` i manifestu:

```text
dependency_class
included_in_package: true | false
include_reason
exclude_reason
```

Dodac polityke kolizji nazw:

```text
preserve_relative_tree
flatten_with_hash_suffix
dedupe_identical_hash
block_different_content_same_name
```

## 8. Blueprint 6: DVC / git-annex / Nix GC

### Co robi dojrzaly system

DVC i Nix nie usuwaja plikow na podstawie "wydaje sie niepotrzebne".

One definiuja:

```text
roots
reachable objects
unreachable objects
scope
dry run
```

git-annex dodaje jeszcze jeden bardzo wazny wzorzec:

```text
Do not drop content unless enough other copies can be verified.
```

### Co pozyczyc

Nasze narzedzie potrzebuje:

```text
Reachability Model
```

Rooty:

```text
protected ALS files
protected Ableton project folders
verified migrated projects
manual keep pins
recent operation manifests
quarantine items within retention period
```

Plik moze byc kandydatem do usuniecia tylko wtedy, gdy:

```text
not reachable from any protected root
and not manually pinned
and not referenced by redirect ledger
and has at least one verified managed copy if content should be preserved
and survived quarantine period
```

### Dlaczego to wazne

To jest serce bezpiecznego sprzatania.

Bez tego narzedzie moze usunac sample z Downloads, ktory nadal jest uzywany przez stary projekt, ktorego akurat nie przeskanowalismy.

### Wplyw na architekture

Dodac moduly:

```text
asset_index
content_hasher
reachability_analyzer
safe_gc_planner
quarantine_manager
pin_manager
```

Dodac komendy/akcje:

```text
scan_roots
mark_project_protected
mark_file_keep
show_unreachable
dry_run_cleanup
move_to_quarantine
delete_after_retention
restore_from_quarantine
```

## 9. Porownanie z obecna metodologia

### Co juz mamy dobrze

```text
copy, never mutate original
semantic diff
manifest
explicit plan
minimal rewrite
oracle experiments with Ableton
block ambiguous matches
do not trust Ableton CRC as identity
```

To jest bardzo dobry fundament.

### Czego jeszcze brakuje

Najwazniejsze braki:

```text
1. Redirect ledger
2. Storage state model
3. Dependency class model
4. Scope model
5. Preflight report as user decision tool
6. Transactional staging/rollback
7. Content-addressed asset index
8. Reachability graph across many projects
9. Safe cleanup / quarantine / restore
10. Long-term verification status
```

## 10. Nowa architektura docelowa

Docelowy system powinien miec trzy warstwy.

### 10.1 Warstwa 1: Project Rewrite Core

To jest aktualny v0.1.

```text
als_reader
sample_ref_extractor
source_classifier
relocate_plan_builder
copy_builder
active_fileref_rewriter
semantic_diff
validator
manifest_writer
```

Cel:

```text
bezpiecznie stworzyc dzialajaca kopie projektu
```

### 10.2 Warstwa 2: Library Management

To jest produkt powracajacy.

```text
asset_index
content_hasher
library_importer
inbox_watcher
dedupe_engine
redirect_ledger
preflight_reporter
```

Cel:

```text
opanowac obecne projekty i nowe sample, zanim zrobi sie chaos
```

### 10.3 Warstwa 3: Safe Cleanup

To jest najwieksza obietnica, ale musi wejsc pozniej.

```text
project_root_registry
reachability_analyzer
safe_gc_planner
quarantine_manager
restore_manager
delete_executor
```

Cel:

```text
powiedziec userowi, co moze usunac, i umiec to cofnac
```

## 11. Rekomendowana kolejnosc wdrazania

### v0.1

```text
Relocate Self-Contained Project
ALS scan
active SampleRef extraction
semantic diff
manifest
```

### v0.2

```text
Build Rescue / Collect Package
copy external active audio
rewrite Path + RelativePath + RelativePathType
preflight report
transactional staging
```

### v0.3

```text
asset index
content hashing
dedupe
library import
redirect ledger
```

### v0.4

```text
multi-project scan
project root registry
reachability graph
safe cleanup dry-run
quarantine
```

### v0.5

```text
fix redirects across many ALS files
batch rewrite with user-approved plan
verification dashboard
restore workflow
```

## 12. Najwazniejszy product insight

Jednorazowy rescue odpowiada na pytanie:

```text
Jak otworzyc ten stary projekt?
```

Produkt powracajacy odpowiada na pytanie:

```text
Jak miec pewnosc, ze moje projekty nie rozpadna sie za pol roku?
```

Dlatego najwazniejsza zmiana architektoniczna to przejscie od:

```text
ALS rewrite tool
```

do:

```text
Audio dependency safety system
```

## 13. Zasada nadrzedna

Do kodu powinna wejsc zasada:

```text
No delete without reachability.
No rewrite without semantic diff.
No cleanup without quarantine.
No confidence without content hash.
No long-term product without manifest history.
```

## 14. Zrodla

```text
Unreal Engine Redirectors:
https://dev.epicgames.com/documentation/unreal-engine/asset-redirectors-in-unreal-engine

Adobe Premiere Project Manager:
https://helpx.adobe.com/ca/premiere/desktop/organize-media/create-projects/copy-project.html

Apple Final Cut Pro Consolidate:
https://support.apple.com/en-euro/guide/final-cut-pro/ver9c7660349/mac

Blender External Data / Paths:
https://docs.blender.org/manual/en/3.0/interface/window_system/topbar.html
https://docs.blender.org/manual/en/latest/files/blend/packed_data.html

Adobe InDesign Preflight and Package:
https://helpx.adobe.com/uk/indesign/using/preflighting-files-handoff.html

SolidWorks Pack and Go:
https://help.solidworks.com/2023/english/SolidWorks/sldworks/c_pack_go_ovw_wpdm.htm

DVC Garbage Collection:
https://doc.dvc.org/command-reference/gc

Nix Garbage Collector Roots:
https://nix.dev/manual/nix/2.31/package-management/garbage-collector-roots

git-annex unused / copies:
https://git-annex.branchable.com/git-annex-unused/
https://git-annex.branchable.com/copies/
```
