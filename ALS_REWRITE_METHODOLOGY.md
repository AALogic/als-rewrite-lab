# ALS Rewrite Methodology

Status: working methodology based on local experiments  
Date: 2026-05-30  
Scope: Ableton Live Set `.als` dependency rewrite for audio files, Mac -> Mac, copy-only workflow

## 1. Cel

Celem nie jest stworzenie pelnego edytora formatu `.als`.

Celem jest stworzenie konserwatywnego mechanizmu, ktory:

1. czyta istniejacy, poprawny plik `.als`,
2. znajduje aktywne zaleznosci audio,
3. tworzy bezpieczna kopie projektu lub paczke,
4. przepisuje tylko znane pola aktywnego `SampleRef/FileRef`,
5. waliduje, ze zmienilo sie tylko to, co mialo sie zmienic,
6. zostawia oryginalny projekt nietkniety.

Najwazniejsza zasada:

```text
Rewrite copies, never originals.
```

## 2. Najwazniejsze odkrycia z testow

### 2.1 `.als` jest gzipowanym XML-em

Plik `.als` mozna:

1. rozpakowac jako gzip,
2. odczytac jako XML,
3. zmodyfikowac,
4. ponownie spakowac jako gzip.

Ableton poprawnie otwiera pliki po takiej operacji, jezeli zmiany sa zgodne ze struktura, ktorej oczekuje.

### 2.2 Aktywne zaleznosci audio siedza w `SampleRef/FileRef`

Najwazniejszy fragment:

```text
SampleRef
  FileRef
    RelativePathType
    RelativePath
    Path
    Type
    OriginalFileSize
    OriginalCrc
```

To jest aktywna lokalizacja pliku audio, ktora Ableton wykorzystuje przy otwieraniu projektu.

### 2.3 `SourceContext/OriginalFileRef` to provenance/historia

`SourceContext/OriginalFileRef` moze wskazywac stare zrodlo pliku.

W minimalnym rewrite nie trzeba go zmieniac.

Test `private_fixture_003_rewrite_result` potwierdzil, ze projekt dziala po zmianie tylko aktywnych `Path`, przy nietknietych historycznych `OriginalFileRef`.

### 2.4 CAS robi wiecej niz minimalny rewrite

W eksperymencie `private_fixture_001` Ableton Collect All and Save:

1. skopiowal zewnetrzne audio do `Samples/Imported`,
2. zmienil aktywne `Path`,
3. zmienil aktywne `RelativePath`,
4. zmienil aktywne `RelativePathType` z `1` na `3`,
5. uzupelnil czesc `OriginalFileSize` i `OriginalCrc`, ktore wczesniej byly `0`,
6. zmienil czesc `SourceContext/OriginalFileRef`, glownie relatywna sciezke i brakujace size/CRC.

To jest zachowanie Abletona jako oracle, ale nie jest to minimalny wymagany zakres kazdego rewrite.

### 2.5 Minimalny relocate dziala

Eksperyment:

```text
source:
<private-lab-root>/source-project

target:
<private-lab-root>/relocated-project
```

Operacja:

1. skopiowano caly projekt,
2. w skopiowanym `private_fixture_001.als` zmieniono tylko aktywne `SampleRef/FileRef/Path`,
3. nie zmieniono `RelativePath`,
4. nie zmieniono `RelativePathType`,
5. nie zmieniono `OriginalFileRef`,
6. nie zmieniono `OriginalFileSize`,
7. nie zmieniono `OriginalCrc`.

Walidacja:

```text
SampleRef: 153 -> 153
OriginalFileRef: 47 -> 47
changed active fields: Path only, 33 times
historical changes: 0
missing non-core active paths: 0
Ableton opened the project successfully
```

Wniosek:

```text
For self-contained project relocation, updating active Path only can be sufficient.
```

## 3. Dwa podstawowe tryby

### 3.1 Tryb A: Relocate Self-Contained Project

Uzywac, gdy projekt juz ma audio w lokalnym `Samples/...`, a chcemy przeniesc caly folder projektu w inne miejsce.

Przyklad:

```text
old_root/Samples/Imported/Kick.wav
new_root/Samples/Imported/Kick.wav
```

Zakres rewrite:

```text
change active SampleRef/FileRef/Path
do not change RelativePath
do not change RelativePathType
do not change OriginalFileRef
do not change OriginalFileSize
do not change OriginalCrc
```

Regula:

```text
If RelativePath starts with Samples/ and RelativePathType is 3,
and the same relative file exists under the new project root,
then set Path to new_project_root / RelativePath.
```

### 3.2 Tryb B: Build Rescue / Collect Package

Uzywac, gdy aktywne audio siedzi poza folderem projektu, np.:

```text
Downloads
Desktop
<private-audio-folder>
Splice modified/cache
other Ableton projects
```

Zakres rewrite minimalny:

```text
copy source audio to target_project_root/Samples/Imported or another planned Samples/... folder
set active Path to copied file absolute path
set active RelativePath to target relative Samples/... path
set active RelativePathType to 3
do not change SourceContext/OriginalFileRef in v0.1
do not change OriginalFileSize/OriginalCrc in v0.1 unless explicitly implementing CAS-like metadata update
```

Regula:

```text
External active file -> copied project-local file -> active FileRef points to new project-local file.
```

## 4. Czego nie robic w v0.1

Nie robic:

1. nie generowac `.als` od zera,
2. nie robic globalnego search/replace po calym XML bez ograniczen,
3. nie nadpisywac oryginalnego `.als`,
4. nie usuwac oryginalnych sampli,
5. nie przepisywac pluginow,
6. nie przepisywac presetow,
7. nie przepisywac Core Library,
8. nie przepisywac User Library bez osobnej reguly,
9. nie przepisywac `OriginalFileRef` w podstawowym trybie,
10. nie ufac `OriginalCrc` jako jedynemu fingerprintowi,
11. nie automatyzowac ambiguous matches.

Najgorszy blad:

```text
Wrong sample match, not failed match.
```

## 5. Klasyfikacja `RelativePathType`

Zaobserwowane wzorce:

```text
1 = external path relative to current project/root relationship
3 = local project sample under Samples/...
5 = Ableton Core Library / factory resources
6 = User Library / user presets, often historical
```

To sa reguly robocze, nie pelna specyfikacja Abletona.

Kazda nowa wersja reguly musi byc potwierdzona fixturem before/after.

## 6. Model danych

### 6.1 SampleReference

Minimalny model:

```text
ref_id
path
relative_path
relative_path_type
type
original_file_size
original_crc
default_duration
default_sample_rate
filename
extension
source_category
exists
usage_context
track_name
xml_node_ref
```

### 6.2 UsageContext

`SampleRef` nie zawsze oznacza ten sam typ uzycia.

Zaobserwowane konteksty:

```text
audio_clip
take_lane
session_clip
simpler_multisample
impulse_sample
unknown
```

Parser powinien zapisywac kontekst, bo reguly rewrite moga sie roznic.

### 6.3 RewriteOperation

```text
operation_id
ref_id
old_path
new_path
old_relative_path
new_relative_path
old_relative_path_type
new_relative_path_type
fields_to_change
reason
rule_id
confidence
```

### 6.4 Manifest

Kazda operacja musi zapisac manifest:

```text
operation_id
ruleset_version
mode
source_project_root
target_project_root
source_als
target_als
files_copied
rewrite_operations
validation_result
user_verification_status
```

## 7. Pipeline operacji

### 7.1 Pipeline ogolny

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

### 7.2 Preflight

Sprawdz:

1. czy source `.als` istnieje,
2. czy source `.als` da sie rozpakowac jako gzip,
3. czy XML jest poprawny,
4. czy target root nie jest oryginalnym folderem projektu,
5. czy target root istnieje albo moze byc utworzony,
6. czy target root nie zawiera nieoczekiwanych plikow, chyba ze user zatwierdzi overwrite/merge,
7. czy oryginalny projekt nie bedzie modyfikowany.

### 7.3 Read ALS

Parser ma:

1. rozpakowac gzip,
2. sparsowac XML,
3. znalezc aktywne `SampleRef/FileRef`,
4. znalezc `SourceContext/OriginalFileRef`,
5. policzyc tagi i konteksty,
6. zachowac referencje do konkretnych wezlow XML.

### 7.4 Build Plan

Plan ma byc jawny.

Przyklad:

```text
33 active refs will be updated
6 files will be copied
120 Core Library refs will be left unchanged
47 OriginalFileRef nodes will be left unchanged
```

### 7.5 Rewrite ALS

W trybie relocate:

```text
for each active ref:
  if RelativePathType == 3
  and RelativePath starts with Samples/
  and target_root / RelativePath exists:
      set Path = absolute(target_root / RelativePath)
```

W trybie rescue/package:

```text
for each resolved external active ref:
  copy source file to planned Samples/... destination
  set Path = absolute(target_root / planned_relative_path)
  set RelativePath = planned_relative_path
  set RelativePathType = 3
```

## 8. Walidacja

Walidacja jest obowiazkowa.

### 8.1 Walidacja pliku

Sprawdz:

```text
gzip valid
xml valid
Ableton root attributes preserved except expected creator/version changes if source came from Ableton
```

### 8.2 Walidacja semantyczna

Sprawdz:

```text
SampleRef count unchanged
OriginalFileRef count unchanged
active FileRef count unchanged
no unexpected field changed
all new active non-core paths exist
relative paths resolve under target root where applicable
OriginalFileRef unchanged in minimal mode
OriginalFileSize unchanged in minimal mode
OriginalCrc unchanged in minimal mode
DefaultDuration unchanged
DefaultSampleRate unchanged
```

### 8.3 Walidacja diffu

W trybie relocate minimalny oczekiwany diff:

```text
changed active fields:
  Path only

unchanged:
  RelativePath
  RelativePathType
  OriginalFileRef
  OriginalFileSize
  OriginalCrc
  DefaultDuration
  DefaultSampleRate
```

W trybie rescue/package minimalny oczekiwany diff:

```text
changed active fields:
  Path
  RelativePath
  RelativePathType

unchanged:
  SourceContext/OriginalFileRef
  OriginalFileSize
  OriginalCrc
  DefaultDuration
  DefaultSampleRate
```

## 9. Protokol eksperymentow z Abletonem jako oracle

Kazdy eksperyment powinien miec strukture:

```text
experiments/
  experiment_name/
    before/
      project_copy/
      snapshot_manifest.json
    after/
      project_copy_or_path_reference
    reports/
      semantic_diff.json
      summary.md
```

Minimalne dane:

```text
Ableton version
source project path
target project path
operation performed
CAS options used
selected ALS
did Ableton open result successfully
notes
```

Badane akcje:

```text
Collect All and Save
Save As
manual missing sample relink
project relocation
package relocation
```

Najwazniejszy artefakt:

```text
semantic diff before/after
```

## 10. Reguly decyzyjne

### 10.1 Kiedy auto-rewrite jest dozwolony

Auto-rewrite mozna wykonac, gdy:

1. aktywny ref jest rozpoznany,
2. target file istnieje,
3. match jest exact lub high confidence,
4. brak konfliktu nazw,
5. kontekst jest wspierany,
6. plan przeszedl preflight,
7. wynik przejdzie semantic validation.

### 10.2 Kiedy blokowac rewrite

Blokuj, gdy:

1. nieznany kontekst `SampleRef`,
2. missing target file,
3. ambiguous match,
4. Core Library ref,
5. plugin/preset ref,
6. SourceContext-only ref,
7. mismatch size/hash,
8. target path koliduje z innym plikiem,
9. parser wykryl nierozpoznana strukture.

## 11. Lekcje z eksperymentu `private_fixture_001`

### 11.1 CAS

Before:

```text
33 external audio refs from <private-audio-folder>
120 Core Library refs
RelativePathType: 1 for external, 5 for Core
```

After CAS:

```text
33 refs moved to Samples/Imported
120 Core Library refs unchanged
RelativePathType: 3 for imported, 5 for Core
6 real audio files copied
6 .asd files generated
```

### 11.2 Minimal relocate

Operation:

```text
copy after-CAS project to new folder
change only active Path from old root to new root
```

Result:

```text
Ableton opened project successfully
```

This confirms:

```text
Path-only rewrite is sufficient for relocating a self-contained project copy.
```

## 12. Pierwsze moduly kodu

Kolejnosc implementacji:

1. `als_reader`
2. `sample_ref_extractor`
3. `source_classifier`
4. `semantic_diff`
5. `relocate_plan_builder`
6. `minimal_path_rewriter`
7. `validator`
8. `manifest_writer`
9. `rescue_package_plan_builder`
10. `copy_builder`
11. `active_fileref_rewriter`

Najpierw kodowac tryb:

```text
Relocate Self-Contained Project
```

Bo zostal potwierdzony przez test i ma najmniejszy zakres zmian.

Potem kodowac:

```text
Build Rescue / Collect Package
```

Bo wymaga matchingu, kopiowania i wiekszej liczby regul.

## 13. Definicja sukcesu v0.1

v0.1 jest udane, jezeli:

1. potrafi wczytac `.als`,
2. potrafi wyciagnac aktywne `SampleRef/FileRef`,
3. potrafi sklasyfikowac `Samples/...`, external, Core Library,
4. potrafi zbudowac plan relokacji self-contained projektu,
5. potrafi przepisac tylko aktywne `Path`,
6. potrafi udowodnic semantic diffem, ze nic innego sie nie zmienilo,
7. projekt otwiera sie w Abletonie.

## 14. Zasada nadrzedna

Kazda nowa automatyczna regule trzeba najpierw udowodnic w jednym z dwoch sposobow:

1. eksperyment before/after z Abletonem jako oracle,
2. minimalny rewrite na kopii, potwierdzony otwarciem projektu w Abletonie.

Bez dowodu:

```text
audit only, no rewrite.
```
