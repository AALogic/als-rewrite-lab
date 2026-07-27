# Current State

Status: live project state  
Date: 2026-06-09

## 1. Aktualny etap

```text
DependencyExtractor v0.1 implemented and verified
```

Nie jestesmy juz tylko w rozmowie o pomysle.

Jestesmy juz w kodowaniu MVP core, ale nadal modul po module.

Jestesmy na etapie:

```text
budowac pipeline po ALSReaderze: ALSReadModel -> DependencyExtractionResult
```

## 2. Aktualny tryb

```text
Navigator / guarded module implementation
```

Czyli:

```text
porzadkujemy fakty
oddzielamy hipotezy
wybieramy pierwszy zakres
piszemy kod tylko wtedy, gdy modul ma spec, kontrakt, taski i testy
```

## 2A. Ostatni zamkniety krok

```text
001-als-reader zostal retroaktywnie objety tym samym guarded workflow co
002-dependency-extractor.

Dodano:
  specs/001-als-reader/module.contract.json
  crates/rescue_core/src/als_reader_impl.rs

Refaktor:
  crates/rescue_core/src/als_reader.rs jest teraz cienkim publicznym API
  prywatna implementacja readera zostala przeniesiona do als_reader_impl.rs
  zachowanie modulu pozostalo bez zmian, potwierdzone testami

Weryfikacja 001 po retroficie:
  python3 tools/workflow_guard.py module-ready 001-als-reader: PASS
  python3 tools/workflow_guard.py verify-module 001-als-reader: PASS
  cargo fmt --check: PASS
  cargo check --workspace: PASS
  cargo test --workspace: PASS, 24 tests passed
  cargo clippy --workspace --all-targets -- -D warnings: PASS

002-dependency-extractor v0.1 zostal zaimplementowany jako czysty modul core.

Nowe pliki kodu:
  crates/rescue_core/src/dependency_extractor.rs
  crates/rescue_core/src/dependency_extractor_impl.rs
  crates/rescue_core/tests/dependency_extractor.rs

Zmiany w eksporcie:
  crates/rescue_core/src/lib.rs eksportuje extract_dependencies oraz modele
  DependencyExtractionResult, DependencyExtractionMetadata, DependencyRef,
  IgnoredInputSummary, DependencyExtractionWarning, DependencyExtractionError.

Granica modulu:
  bierze ALSReadModel v0.2 z ALSReadera
  zwraca DependencyExtractionResult v0.1
  nie czyta sampli z dysku
  nie sprawdza istnienia sciezek
  nie klasyfikuje zrodel
  nie matchuje brakujacych sampli
  nie planuje kopiowania
  nie przepisuje ALS

Weryfikacja:
  cargo fmt --check: PASS
  cargo check --workspace: PASS
  cargo test --workspace: PASS, 24 tests passed
  python3 tools/workflow_guard.py verify-module 002-dependency-extractor: PASS
  cargo clippy --workspace --all-targets -- -D warnings: PASS
```

## 2B. Ostatni hardening przed 003

Date:

```text
2026-06-30
```

Zakres:

```text
Utwardzenie 001/002 po review kodu, bez przechodzenia do ALSRewriter.
```

Wykonano:

```text
Dodano crates/rescue_core/src/path_text.rs.
ALSReader nie uzywa juz std::path::Path do wyciagania filename/extension
z raw sciezek ALS, dzieki czemu Windows backslashes i UNC paths nie sa
interpretowane przez host OS.

ALSReader wybiera first non-empty candidate przy filename/extension:
  raw_path jesli ma tresc
  raw_relative_path jesli raw_path jest pusty/brakujacy

ALSReader ma limity:
  MAX_COMPRESSED_ALS_BYTES
  MAX_DECOMPRESSED_XML_BYTES

Dodano strukturalne bledy:
  ALS_COMPRESSED_TOO_LARGE
  ALS_DECOMPRESSED_XML_TOO_LARGE

ALS_READER_VERSION podniesiono do 0.2.1.

CLI ma nowa komende:
  rescue extract path/to/file.als --json

Komenda extract uruchamia:
  analyze_als
  -> extract_dependencies
  -> DependencyExtractionResult JSON
```

Odlozono swiadomie:

```text
Pelna migracja stringowych statusow do enumow.
Zmiana nazwy active_audio_references albo dodanie activity_status.
Stabilne dependency_id do manifestow/cross-run comparison.
Parsowanie liczbowych raw ALS fields w ALSReader.
Projekt storage/SQLite schema.
```

Powod:

```text
Te tematy sa zasadne, ale powinny wejsc przy 003 PathVerifier,
ManifestWriter albo rewrite-readiness, a nie jako szeroki refactor przed 003.
```

## 2C. Architecture Spine And Path Semantics

Date:

```text
2026-06-30
```

Zakres:

```text
Utrwalenie minimalnej warstwy architektonicznej oraz przygotowanie 003
PathVerifier bez pisania jeszcze kodu weryfikatora.
```

Dodano:

```text
docs/architecture/README.md
docs/architecture/diagrams/001-core-pipeline.mmd
docs/architecture/adr/ADR-001-no-original-rewrite.md
docs/architecture/adr/ADR-002-path-semantics.md
docs/architecture/traceability.md
specs/003-path-verifier/spec.md
specs/003-path-verifier/plan.md
specs/003-path-verifier/tasks.md
specs/003-path-verifier/fixture-contract.md
specs/003-path-verifier/module.contract.json
crates/rescue_core/src/path_parser.rs
crates/rescue_core/tests/path_parser.rs
```

Decyzje:

```text
Raw ALS paths sa zachowywane jako tekst.
Interpretacja ksztaltu sciezki nalezy do PathParser / PathVerifier.
Sprawdzenie istnienia pliku na dysku zaczyna sie dopiero w 003 PathVerifier.
PathVerifier nie szuka sampli, nie matchuje, nie klasyfikuje, nie kopiuje i
nie przepisuje ALS.
```

Nowy core helper:

```text
RawAlsPath
ParsedAlsPath
AlsPathKind
parse_als_path
```

Cel:

```text
Nie blokowac przyszlego Windows/macOS supportu przez przedwczesne uzycie
host OS path semantics.
```

## 3. Potwierdzone fakty

```text
.als jest gzipowanym XML-em.

Aktywne audio refs sa w SampleRef/FileRef.

SourceContext/OriginalFileRef jest historia/provenance i nie musi byc zmieniany w minimalnym rewrite.

Dla self-contained projektu minimalny Path-only rewrite zadzialal na kopii private_fixture_001 -> private_fixture_003_rewrite_result.

Oryginalne projekty maja pozostac nietkniete.

Rewrite musi dzialac na kopii.

Ambiguous match ma blokowac automatyczny rewrite.
```

## 4. Hipotezy

```text
ALSReader powinien byc pierwszym modulem.

Source classification moze byc osobnym modulem, a nie czescia ALSReader.

Preflight report powinien wejsc przed pierwszym realnym rewrite/package.

Redirect ledger i reachability sa wazne, ale pozniej niz v0.1.

Kierunek technologiczny zostal przyjety roboczo: Rust core + CLI first + Tauri/TypeScript desktop later + SQLite for persistent index.

Platform stance zostal doprecyzowany:
  macOS first
  platform-portable core
  Windows migration nie jest w MVP
  core modules nie moga bez potrzeby hardcodowac macOS-only zalozen
  raw ALS paths maja byc zachowane jako dane, a interpretacja platformowa
  nalezy do PathVerifier / filesystem adapters

Oficjalna struktura projektu jest opisana w PROJECT_STRUCTURE.md.

Tryb Product Office zostal dodany jako probna funkcja do przetwarzania rozmow, pomyslow i metliku w session digest, backlog, spec albo aktualizacje state.

AGENTS.md zostal dodany jako instrukcja pracy Codexa w repo.

AI_CONTRACT.md zostal dodany jako kontrakt bezpieczenstwa AI i danych.

specs/001-als-reader/spec.md zostal zreviewowany pod start implementacji.

specs/001-als-reader/plan.md zostal dodany jako plan pierwszego modulu.

specs/001-als-reader/tasks.md zostal dodany jako lista malych zadan.

specs/001-als-reader/fixture-contract.md zostal dodany jako kontrakt oczekiwanych wynikow dla pierwszych ALS fixture.

Minimalne fixture .als zostaly skopiowane do tests/fixtures/als.

Sztuczny invalid fixture zostal dodany do tests/fixtures/invalid/not_gzip.als.

Rust workspace zostal utworzony:
  Cargo.toml
  crates/rescue_core
  cli/rescue-cli

Pierwsze fixture tests zostaly dodane dla ALSReader.

Minimalna implementacja ALSReader zostala napisana jako read-only parser gzip XML.

Rust toolchain zostal zainstalowany lokalnie przez rustup.

cargo 1.96.0, rustc 1.96.0 i rustup 1.29.0 sa dostepne po zaladowaniu ~/.cargo/env.

Historical v0.1 cargo fmt --check: PASS.

Historical v0.1 cargo test: PASS, 5 tests passed.

Historical v0.1 manual CLI smoke test:
  cargo run -q -p rescue-cli -- analyze tests/fixtures/als/private_fixture_002_external_refs.als --json
  PASS, JSON zwrocil sample_ref_count 11 i active_file_ref_count 11.

PROJECT_MAP.md zostal dodany jako mapa domen i granic odpowiedzialnosci.

Workflow Smoke Test 001 zostal wykonany: 9 PASS, 1 PASS WITH WATCH ITEM, 0 FAIL.

Raport zapisano w session-digests/2026-06-01_workflow-smoke-test-001.md.

PRODUCT_SPINE.md zostal dodany jako globalny kregoslup produktu:
  product promise
  primary use cases
  capability map
  bramki od wizji do kodu
  clarification request
  definition of ready/done

AGENTS.md, PROJECT_NAVIGATOR.md, PRODUCT_OFFICE.md, PROJECT_STRUCTURE.md,
PROJECT_MAP.md i specs/001-als-reader/spec.md zostaly zaktualizowane tak,
zeby spec i kod musialy przechodzic przez Product Spine gates.

ALSReader v0.1 jest obecnie uznany za dzialajacy read-only diagnostic slice,
ale nie za pelny kontrakt danych dla analyzer/package/rewrite.

ALSReader v0.2 ma zaakceptowana granice odpowiedzialnosci:
  czyta ALS
  tworzy ALSReadModel v0.2
  nie sprawdza istnienia sampli na dysku
  nie klasyfikuje zrodel
  nie matchuje
  nie planuje kopiowania
  nie przepisuje ALS

CLI JSON jest raportem diagnostycznym pochodzacym z modelu readera, a nie
pelna umowa danych dla kolejnych modulow.

ALSReadModel v0.2 jest wewnetrznym kontraktem downstream.

PathVerifier jest przyszlym wlascicielem sprawdzania, czy sciezki z ALS
istnieja na dysku.

ALSReader v0.2 zostal zaimplementowany jako pierwszy pass:
  analyze_als zwraca ALSReadModel v0.2
  set_metadata zawiera hash SHA-256 pliku ALS, rozmiar, wersje readera i liczniki
  active_audio_references zastapilo stare active_refs
  historical_refs jest osobna lista, nie miesza sie z aktywnymi zaleznosciami
  non_audio_dependency_signals zbiera ostrozne sygnaly FileRef poza aktywnymi SampleRef
  exists_on_disk zostalo usuniete z ALSReader output
  active path missing warnings zostaly usuniete z ALSReader
  CLI drukuje diagnostyczny JSON wyprowadzony z ALSReadModel

cargo fmt --check po ALSReadModel v0.2: PASS.

cargo test po ALSReadModel v0.2: PASS, 7 tests passed.

Manual CLI smoke test po ALSReadModel v0.2:
  cargo run -q -p rescue-cli -- analyze tests/fixtures/als/private_fixture_002_external_refs.als --json
  PASS, JSON zwrocil set_metadata, active_audio_references, historical_refs
  i non_audio_dependency_signals.

ALSReader v0.2 hardening po review GPT:
  error codes ujednolicone do spec-style taxonomy:
    ALS_NOT_FOUND
    ALS_NOT_READABLE
    ALS_NOT_GZIP
    ALS_XML_INVALID
    ALS_UNSUPPORTED_ROOT
    ALS_INTERNAL_ERROR
  dodano testy:
    missing file
    gzip invalid XML
    gzip without Ableton root
    active RelativePathType 0 z kopii korpusu
  cargo fmt --check: PASS
  cargo test: PASS, 11 tests passed

Eksperyment ALS Structure Corpus 20 zostal wykonany i zapisany:
  experiments/2026-06-02_als_structure_corpus_20/
  session-digests/2026-06-02_als-structure-corpus-20.md

Eksperyment OriginalCrc sample subset zostal wykonany i zapisany:
  experiments/2026-06-02_als_structure_corpus_20/crc_probe_sample_subset/
  session-digests/2026-06-09_original-crc-probe-sample-subset.md

Wynik:
  z 5 skopiowanych ALS skopiowano 21 realnych sampli
  lacznie okolo 244 MB na dysku
  ALS OriginalFileSize zgadza sie z faktycznym rozmiarem dla wszystkich 21
  ALS OriginalCrc nie zgadza sie z prostym CRC32 low16 ani Adler32 low16
  dla tych 21 probek

Eksperyment OriginalCrc hypothesis probe zostal wykonany i zapisany:
  experiments/2026-06-02_als_structure_corpus_20/crc_probe_sample_subset/hypothesis_probe/
  experiments/2026-06-02_als_structure_corpus_20/crc_probe_sample_subset/fast_full_payload_probe/
  session-digests/2026-06-09_original-crc-hypothesis-probe.md

Wynik:
  Python fragment/context sweep: 0 trafien na 21 sampli
  Fast C full/payload sweep: 1 pojedyncze trafienie na 21 sampli
  brak kandydata trafiajacego w wiecej niz 1 sampel
  popularne CRC16/CRC32/sum/Fletcher hipotezy nie sa potwierdzone
  OriginalCrc pozostaje UNKNOWN i moze byc tylko weak evidence

Plan nastepnego testu OriginalCrc zostal zapisany:
  PRODUCT_SPEC.md, sekcja 11.6 OriginalCrc Controlled Ableton Test
  experiments/2026-06-02_als_structure_corpus_20/crc_probe_sample_subset/controlled_ableton_original_crc_test_plan.md

Cel planu:
  sprawdzic w kontrolowanym projekcie Abletona, czy OriginalCrc reaguje na
  zmiane lokalizacji, nazwy, bajtow pliku, zdekodowanego PCM, metadanych,
  Collect All and Save, Save As oraz regeneracje .asd

Eksperyment OriginalCrc static direction dataset zostal wykonany i zapisany:
  tools/experiments/python/original_crc_static_direction_dataset.py
  experiments/2026-06-02_als_structure_corpus_20/original_crc_static_direction_dataset/
  experiments/2026-06-02_als_structure_corpus_20/original_crc_static_direction_dataset_balanced/
  session-digests/2026-06-09_original-crc-static-direction-dataset.md

Rekomendowany dataset do kolejnych statycznych testow:
  experiments/2026-06-02_als_structure_corpus_20/original_crc_static_direction_dataset_balanced/

Wynik balanced:
  copied ALS files: 8
  copied sample files: 60
  total copied sample bytes: 144109271
  all source stat checks passed: true
  pokrycie: .aif/.wav/.mp3, RelativePathType 0/1/3/5, sidecary .asd,
  same_source_path_repeated, same_source_path_different_original_crc,
  same_original_crc_and_size

Bezpieczenstwo:
  oryginalne sample byly tylko kopiowane
  oryginalne ALS nie byly modyfikowane
  wszystkie source stat checks po kopiowaniu przeszly

Eksperyment OriginalCrc static analysis probe zostal wykonany:
  tools/experiments/python/original_crc_static_analysis_probe.py
  experiments/2026-06-02_als_structure_corpus_20/original_crc_static_direction_dataset_balanced/static_analysis_probe/
  session-digests/2026-06-09_original-crc-static-analysis-probe.md

Testy wykonane:
  same SHA-256 vs OriginalCrc
  kolizje OriginalCrc
  decoded PCM przez afconvert
  .asd hash/checksum/embedded-value probe
  korelacja z metadanymi audio z afinfo i polami ALS

Wynik:
  sample count: 60
  decode errors: 0
  unique SHA-256: 52
  SHA groups with multiple nonzero OriginalCrc: 0
  unique decoded PCM hashes: 52
  PCM hash groups with multiple nonzero OriginalCrc: 0
  unique nonzero OriginalCrc: 37
  OriginalCrc values used by >1 sample: 9
  OriginalCrc values used by >1 SHA-256: 6
  decoded PCM checksum match rows: 1
  .asd checksum match rows: 0
  metadata candidate match rows: 0

Interpretacja:
  OriginalCrc jest stabilny dla identycznych plikow/PCM/.asd w tym datasetcie,
  ale koliduje pomiedzy roznymi plikami i roznym PCM.
  Nie jest wyjasniony przez proste checksumy, .asd checksum ani oczywiste
  metadane audio.
  Nadal: OriginalCrc = UNKNOWN / weak evidence only.

Dodano zasade minimalnego przekazania danych miedzy modulami:
  PRODUCT_SPEC.md, sekcja 5C Minimal handoff rule
  specs/001-als-reader/spec.md, sekcja 6A ALSReader Handoff Field Policy

Cel:
  odroznic pola konieczne dla nastepnego modulu od pol zebranych diagnostycznie,
  dowodowo lub "na przyszlosc".

Aktualne etykiety dojrzalosci pol:
  core_handoff
  supporting_evidence
  diagnostic_only
  preserve_for_future
  unknown_semantics

Zasada:
  DependencyExtractor v0.1 moze polegac tylko na core_handoff.
  Uzycie pozostalych pol przez downstream wymaga jawnej zmiany kontraktu,
  aktualizacji specyfikacji i testow kontraktowych.

ALSReader v0.2 closeout zostal wykonany:
  session-digests/2026-06-09_alsreader-v02-closeout.md

Weryfikacja:
  cargo fmt --check: PASS
  cargo test --workspace: PASS, 11 ALSReader tests passed
  cargo check --workspace: PASS
  CLI smoke test: PASS
  cargo clippy: NOT RUN, cargo-clippy nie jest zainstalowany w toolchainie

Decyzja:
  ALSReader v0.2 jest zaakceptowany jako read-only foundation module.
  Nie blokuje przejscia do DependencyExtractor v0.1.

Nieblokujacy backlog:
  automatyczny CLI smoke test jesli praktyczne
  clippy po instalacji komponentu
  stabilna strategia xml_locator przed ALSRewriter
  lepsze usage_context po testach domenowych
  osobna specyfikacja PathVerifier
  specyfikacja DependencyExtractor v0.1

DependencyExtractor v0.1 concept decisions zostaly spisane jako modul 002:
  specs/002-dependency-extractor/spec.md
  specs/002-dependency-extractor/fixture-contract.md
  specs/002-dependency-extractor/tasks.md

Decyzje:
  tylko aktywne audio sample
  input: ALSReadModel v0.2 core_handoff subset
  output: DependencyExtractionResult v0.1
  1 active_audio_reference = 1 DependencyRef
  brak deduplikacji w extractorze
  niepelne refy sa zachowywane z warningami
  statusy: extraction_status, path_basis
  OriginalCrc = weak evidence only
  0 dependencies = valid result
  wynik deterministyczny
  brak filesystem checks, source classification, matching, copy, rewrite

DependencyExtractor v0.1 spec review corrections:
  doprecyzowano, ze resolved_path/existence_status nie istnieja w DependencyRef
  v0.1 i naleza do PathVerifier
  doprecyzowano, ze source_category/risk_flags naleza do DependencyClassifier
  usunieto dwuznaczne "dependencies empty or result untrusted" dla bledow inputu
  warning dependency_id / als_ref_id sa opcjonalne

DependencyExtractor v0.1 ALSReader alignment corrections:
  wersja wejscia jest sprawdzana w set_metadata.als_read_model_version
  DependencyRef v0.1 zachowuje source_kind z ActiveAudioReference
  OriginalFileSize, OriginalCrc, DefaultDuration i DefaultSampleRate pozostaja
  raw string values, bez parsowania/liczbowej koercji w module 002
  evidence_status w spec 001 zostal ujednolicony do lowercase, zgodnie z kodem

Eksperyment skopiowal 20 plikow .als i analizowal tylko kopie.
Wyniki wskazuja m.in.:
  RelativePathType 0 wystepuje licznie w aktywnych refs.
  FileRef istnieje poza aktywnymi SampleRef/FileRef.
  OriginalFileRef wystepuje takze w device/source contexts.
  ALSReader prawdopodobnie potrzebuje xml_context / usage_context dla downstream.
```

## 5. Otwarte pytania

```text
Czy dodac automatyczny CLI smoke test teraz, czy wystarczy manualny smoke test dla spec 001?

Czy tryb Product Office pomaga wystarczajaco, czy trzeba go uproscic po pierwszym uzyciu?

Ktore otwarte granice z PROJECT_MAP.md trzeba rozstrzygnac przed implementacja ALSReader?

Czy PROJECT_WORKFLOW.md zostaje aktywnym dokumentem, czy pozniej stanie sie dokumentem archiwalnym?

Jakie dodatkowe warstwy poza danymi musza byc zachowane dla downstream:
zachowanie, decyzje usera, walidacja, manifest, rewersyjnosc, batch, UX?

Jak dokladnie rozumiec RelativePathType 0?

Jakie xml_context wartosci musza byc zachowane dla analyzer/package/rewrite?
```

## 6. Nie teraz

```text
UI
Windows
cross-platform migration
delete / cleanup
safe garbage collection
plugin rewriting
batch rewrite wielu projektow
pelny sample manager
Tauri UI przed stabilnym core
```

## 7. Nastepny maly krok

```text
Review ALSReadModel v0.2 implementation output on 1-2 fixtures.
Then decide whether the next coding step is:
  automated CLI smoke test
  stable xml_locator strategy
  usage_context improvement
  or specs/002-path-verifier
```

Spec 001 obecnie zaklada:

```text
Wejscie:
  path do pliku .als

Wyjscie:
  ALSReadModel v0.2 jako internal downstream contract
  CLI JSON jako uproszczony raport diagnostyczny

Robi:
  gzip read
  XML parse
  extract set metadata
  extract active audio references
  separate historical/provenance references
  capture raw path fields and raw RelativePathType
  capture non-audio dependency signals as report-only evidence
  capture structure/context info where possible

Nie robi:
  no path existence check
  no source classification
  no matching
  no copy
  no rewrite
```

Fixture contract ma pierwsze oczekiwane liczby:

```text
private_fixture_001_after_collect:
  SampleRef 153
  active FileRef 153
  RelativePathType: 3 -> 33, 5 -> 120

private_fixture_001_before_collect:
  SampleRef 153
  active FileRef 153
  RelativePathType: 1 -> 33, 5 -> 120

synthetic_fixture_zero_active:
  SampleRef 0
  active FileRef 0

private_fixture_002_external_refs:
  SampleRef 11
  active FileRef 11
  unique active paths 4
```

Spec 001 nie robi:

```text
Nie robi:
  no copy
  no rewrite
  no matching
  no delete
  no UI

Walidacja:
  Python sanity check dla skopiowanych fixture: PASS
  cargo fmt --check: PASS after ALSReadModel v0.2
  cargo test: PASS, 11 tests passed after ALSReader v0.2 hardening
  manual CLI smoke test: PASS after ALSReadModel v0.2
```

Product Spine review status:

```text
Vision Gate: PASS for ALSReader v0.2
Use Case Gate: PASS for read-only dependency extraction foundation
Flow Gate: PASS for reader boundary; downstream flow remains future work
Data Gate: PASS for first ALSReadModel v0.2 implementation pass
Behavior Gate: PASS for read-only behavior
Decision Gate: N/A for reader, because no user decision is made here
Safety Gate: PASS
Audit/Manifest Gate: N/A for reader; future manifest consumes reader facts
Reversibility Gate: N/A for reader
Batch Gate: N/A for reader MVP
Evidence Gate: PASS with existing fixture corpus, missing extra fixtures noted
Module Gate: PASS for first ALSReader v0.2 implementation pass
```

Nowa zasada implementacji:

```text
contract/spec
-> plan
-> tasks
-> fixture/test contract
-> failing tests when practical
-> implementation
-> passing tests
```

Workflow Guard Simulation wykonany:

```text
experiments/2026-06-09_workflow_guard_simulation/

Cel:
  sprawdzic, czy code-enforced workflow guard lapie bledy procesu/kontraktu,
  ktore markdown-only review moze przepuscic.

Wynik:
  bad_scenario_count: 5
  baseline_markdown_review caught: 0/5
  code_guard_module_ready caught: 2/5
  code_guard_verify_module caught: 5/5

Wniosek:
  markdown nadal jest dobry do kontekstu i sensu produktu,
  ale acceptance/build guard powinien byc kodem albo testem.
  Najbardziej wartosciowy nastepny krok to minimalny module.contract + verify-module
  dla 002-dependency-extractor, zanim zaakceptujemy implementacje modulu 002.
```

ALSReader Guard Simulation wykonany:

```text
experiments/2026-06-09_alsreader_guard_simulation/

Cel:
  retrospektywnie sprawdzic, czy code-enforced guard lapie regresje jakosci
  ALSReadera, ktore markdown-only review moglby przepuscic.

Wynik:
  bad_scenario_count: 6
  baseline_markdown_review caught: 0/6
  process_guard_ready caught: 1/6
  behavioral_guard_verify caught: 5/6
  actual_current_alsreader_behavioral_guard_passed: true
  actual_current_alsreader_process_guard_passed: false

Wniosek:
  aktualny kod ALSReadera jest zdrowy behawioralnie wedlug guarda,
  ale modul 001 nie ma jeszcze machine-readable module.contract, bo powstal
  przed tym workflow. Nie przepisywac ALSReadera tylko dla procesu.
  Zastosowac module.contract + verify-module przede wszystkim do modulu 002.
```

Clean Code Guard Simulation wykonany:

```text
experiments/2026-06-09_clean_code_guard_simulation/

Cel:
  sprawdzic, czy podejscie "agent as code" poprawia nie tylko kontrakty
  funkcjonalne, ale tez czystosc kodu, refactor safety i utrzymywalnosc.

Wynik:
  bad_scenario_count: 9
  baseline_markdown_review caught: 0/9
  quality_guard_verify caught: 9/9
  quality_guard_false_rejections_on_good: 0

Zlapane klasy bledow:
  missing machine-readable quality contract
  forbidden dependency/import
  public contract drift
  responsibility leak field
  refactor mode changing tests
  oversized function
  god module growth
  surviving critical mutation
  missing required quality test

Probe aktualnego ALSReadera:
  cargo test --workspace: PASS, 11 tests
  crates/rescue_core/src/lib.rs: okolo 494 lines
  analyze_als: okolo 238 lines

Wniosek:
  quality guard daje mierzalna poprawe nad markdown-only review.
  Aktualny ALSReader jest poprawny behawioralnie, ale przed modulem 002 warto
  nie powiekszac monolitycznego lib.rs; preferowac rozdzielenie na moduly/pliki
  przy zachowaniu testow i publicznych kontraktow.
```

Workflow Guard Full Matrix wykonany:

```text
experiments/2026-06-09_workflow_guard_full_matrix/

Cel:
  przetestowac 1:1 dziesiec zaproponowanych testow workflow:
  Spec-To-Test Coverage, Contract Drift, Forbidden Responsibility Leak,
  Missing Test, Ambiguity Gate, Refactor Discipline, Downstream Consumer,
  Mutation, A/B Workflow, Closeout Completeness.

Wynik:
  bad_scenario_count: 10
  good_scenario_count: 1
  baseline_markdown_review caught: 0/10
  code_workflow_guard caught: 10/10
  code_guard_false_rejections_on_good: 0

Wniosek:
  poprzednie eksperymenty pokrywaly czesc listy, ale nie pelna macierz.
  Pelna macierz potwierdza, ze kodowy guard lapie klasy bledow, ktore
  markdown-only review przepuszcza.
  Przy module 002 przyjac module.contract + required obligations + required
  tests + downstream smoke + closeout jako acceptance model.
```

ALSReader structural refactor wykonany:

```text
Cel:
  przygotowac rescue_core pod modul 002 bez powiekszania monolitycznego lib.rs.

Zmiany:
  crates/rescue_core/src/lib.rs
    cienki publiczny eksport API

  crates/rescue_core/src/models.rs
    publiczne modele ALSReadModel / ActiveAudioReference / errors / warnings

  crates/rescue_core/src/als_reader.rs
    analyze_als i prywatne helpery ALSReadera

  analyze_als zostal dodatkowo rozbity na prywatne kroki:
    extract_active_audio_references
    extract_historical_refs
    extract_non_audio_dependency_signals

Publiczny kontrakt:
  rescue_core::{analyze_als, ALSError, ALSReadModel} zachowany.
  Testy nie byly zmieniane.

Weryfikacja:
  cargo fmt --check: PASS
  cargo check --workspace: PASS
  cargo test --workspace: PASS, 11 tests
  ALSReader behavioral guard: PASS dla aktualnego ALSReadera
  cargo clippy: NOT RUN, cargo-clippy nie jest zainstalowany w toolchainie

Metryka:
  przed: crates/rescue_core/src/lib.rs okolo 494 lines, analyze_als okolo 238 lines
  po:    lib.rs 8 lines, models.rs 165 lines, als_reader.rs okolo 358 lines,
         analyze_als okolo 90 lines

Wniosek:
  Modul 002 moze dostac osobny plik dependency_extractor.rs zamiast rosnac
  w lib.rs. Dalsze rozbijanie als_reader.rs jest opcjonalne i powinno byc
  robione tylko jesli pojawi sie konkretna potrzeba utrzymaniowa.
```

Guarded Spec-Driven Development wdrozone jako globalny workflow:

```text
Status:
  globalny standard dla nowych modulow od 002 wzwyz
  002-dependency-extractor jest pilotem

Nowe elementy:
  tools/workflow_guard.py
  specs/002-dependency-extractor/module.contract.json
  specs/002-dependency-extractor/plan.md
  specs/002-dependency-extractor/tasks.md -> sekcja Test Obligations

Zaktualizowane dokumenty workflow:
  AGENTS.md
  PROJECT_WORKFLOW.md
  PRODUCT_SPINE.md
  PRODUCT_SPEC.md

Nowy flow:
  spec.md
  -> plan.md
  -> tasks.md
  -> fixture-contract.md
  -> module.contract.json
  -> workflow_guard module-ready
  -> tests first where practical
  -> implementation
  -> workflow_guard verify-module
  -> closeout / CURRENT_STATE

Weryfikacja:
  python3 -m py_compile tools/workflow_guard.py: PASS
  python3 tools/workflow_guard.py module-ready 002-dependency-extractor: PASS
  python3 tools/workflow_guard.py verify-module 002-dependency-extractor: EXPECTED FAIL
    powod: brak jeszcze dependency_extractor.rs, wymaganych testow i publicznych typow
  cargo fmt --check: PASS
  cargo check --workspace: PASS
  cargo test --workspace: PASS, 11 tests

Decyzja:
  module-ready PASS oznacza, ze mozna zaczac pisac testy/kod modulu 002.
  verify-module FAIL jest poprawne przed implementacja i bedzie bramka akceptacji
  po implementacji.
```

Eksperyment Code Quality Standard Simulation wykonany:

```text
Cel:
  sprawdzic, czy warto dodawac centralny CODE_QUALITY_STANDARD.md, zanim
  faktycznie dolozymy kolejny staly plik do workflow.

Lokalizacja:
  experiments/2026-06-09_code_quality_standard_simulation/

Porownanie:
  baseline distributed docs review
  vs
  central quality standard + guard-enforced anchors

Testowane zle scenariusze:
  missing standard
  missing standard section
  module contract not referencing standard
  missing quality obligation
  destructive operation
  panic-based error handling
  long function
  missing quality/safety test
  missing code review record

Wynik:
  bad_scenario_count: 9
  good_scenario_count: 1
  baseline_caught_bad_scenarios: 0/9
  quality_standard_guard_caught_bad_scenarios: 9/9
  quality_standard_guard_false_rejections_on_good: 0

Weryfikacja:
  python3 -m py_compile code_quality_standard_sim.py: PASS
  python3 code_quality_standard_sim.py: PASS
  cargo test --workspace: PASS, 11 tests

Wniosek:
  centralny standard jakosci jest wart dodania, ale tylko jako dokument
  podlaczony do module.contract.json, tasks.md, testow i workflow_guard.py.
  Sam markdown bez egzekwowalnych kotwic bylby za slaby.
```

ENGINEERING_RULES.md wdrozony jako praktyczny standard inzynierski:

```text
Cel:
  zamienic ogolne zasady clean code / modularnosci / bezpieczenstwa danych
  w staly standard, ktory Codex musi brac pod uwage przy projektowaniu,
  implementacji i refaktoryzacji kodu.

Nowy plik:
  ENGINEERING_RULES.md

Wersja:
  Engineering Rules Version: 0.1

Zakres:
  wszystkie nowe moduly implementacyjne od 002-dependency-extractor wzwyz.

Charakter:
  nie jest to luzny esej markdown.
  Dokument zawiera poziomy egzekwowania:
    ENFORCED_BY_GUARD
    ENFORCED_BY_TEST
    ENFORCED_BY_TYPE
    REVIEW_ONLY
    DOCUMENTED_ONLY

Podlaczenie do workflow:
  AGENTS.md
  PROJECT_WORKFLOW.md
  PRODUCT_SPINE.md
  PRODUCT_SPEC.md
  specs/002-dependency-extractor/module.contract.json
  specs/002-dependency-extractor/tasks.md
  specs/002-dependency-extractor/plan.md
  tools/workflow_guard.py

Guard:
  workflow_guard.py sprawdza teraz:
    czy module.contract.json zawiera engineering_rules
    czy ENGINEERING_RULES.md istnieje
    czy wersja dokumentu zgadza sie z kontraktem modulu
    czy wymagane sekcje dokumentu istnieja
    czy tasks.md zawiera wymagane quality obligations
    czy product source nie zawiera zakazanych wzorcow z kontraktu

002-dependency-extractor:
  module.contract.json wskazuje ENGINEERING_RULES.md version 0.1.
  tasks.md zawiera osobna sekcje Engineering Quality Obligations.
  plan.md dodaje krok potwierdzenia zasad inzynierskich przed kodem.

Weryfikacja:
  python3 -m json.tool specs/002-dependency-extractor/module.contract.json: PASS
  python3 -m py_compile tools/workflow_guard.py: PASS
  python3 tools/workflow_guard.py module-ready 002-dependency-extractor: PASS
  python3 tools/workflow_guard.py verify-module 002-dependency-extractor: EXPECTED FAIL
    powod: modul 002 nie ma jeszcze kodu, testow i publicznych struktur
  cargo fmt --check: PASS
  cargo check --workspace: PASS
  cargo test --workspace: PASS, 11 tests

Wniosek:
  standard jakosci zostal dodany w wersji praktycznej i polaczonej z guardem.
  Nie implementuje jeszcze modulu 002, ale wzmacnia bramke przed jego budowa.
```

Eksperyment AI Workflow Bypass Adversarial wykonany:

```text
Cel:
  sprawdzic kreatywne sposoby, w jakie AI moze zgubic sie, skrocic droge albo
  ominac workflow, jednoczesnie sprawiajac wrazenie, ze artefakty sa zielone.

Lokalizacja:
  experiments/2026-06-09_ai_workflow_bypass_adversarial/

Metoda:
  tymczasowe toy module repos
  prawdziwa logika tools/workflow_guard.py uruchamiana przez monkey-patched root
  scenariusze red-team przeciwko obecnemu workflow

Wynik:
  scenario_count: 12
  bad_scenario_count: 11
  bad_scenarios_caught_by_current_guard: 2
  bad_scenarios_missed_by_current_guard: 9
  false_rejections_on_good: 0

Guard zlapal:
  direct panic in expected source
  missing ENGINEERING_RULES.md

Guard przepuscil:
  unchecked_quality_obligations
  empty_tests_by_name_only
  hidden_forbidden_code_in_unlisted_file
  weakened_contract_removes_required_test
  spec_contract_test_coverage_gap
  missing_review_record
  destructive_shell_escape
  public_contract_extra_field
  empty_fixture_contract

Weryfikacja:
  python3 -m py_compile ai_workflow_bypass_adversarial.py: PASS
  python3 ai_workflow_bypass_adversarial.py: PASS
  python3 tools/workflow_guard.py module-ready 002-dependency-extractor: PASS
  cargo test --workspace: PASS, 11 tests

Wniosek:
  obecny guard v0.1 jest lepszy niz markdown-only, ale nadal lapie glownie
  proste naruszenia. Nie chroni jeszcze dobrze przed subtelnym "satisfy the
  shape, skip the substance" typowym dla pracy z AI.

Rekomendowany hardening:
  1. require checked obligations on verify-module
  2. require non-empty docs plus required literals/sections
  3. exact public contract mode
  4. scan all relevant Rust source for high-risk forbidden patterns
  5. spec-to-obligation coverage markers for MUST rules
  6. machine-readable closeout/review record
  7. minimal empty-test detection
```

Eksperyment AI Coding Pain Points Research wykonany:

```text
Cel:
  na podstawie zewnetrznych zrodel o typowych problemach AI codingu stworzyc
  testy, ktore sprawdzaja, czy nasze workflow chroni przed realnymi bolaczkami.

Zrodla:
  Stack Overflow Developer Survey 2025
  METR early-2025 AI developer productivity study
  Do Users Write More Insecure Code with AI Assistants?
  USENIX Security 2025 package hallucination paper
  ITPro / Tricentis 2026 quality report coverage

Lokalizacja:
  experiments/2026-06-09_ai_coding_pain_points_research/

Wynik:
  scenario_count: 14
  bad_scenario_count: 13
  bad_scenarios_caught_by_current_guard: 2
  bad_scenarios_missed_by_current_guard: 11
  false_rejections_on_good: 0

Guard zlapal:
  direct panic in expected source
  missing ENGINEERING_RULES.md

Guard przepuscil:
  ignored_required_tests
  tautological_tests
  unreviewed_external_dependency
  wrong_field_type_by_same_name
  nondeterministic_output
  platform_hardcoded_path
  network_privacy_leak
  todo_placeholder
  unsafe_block
  swallowed_error
  ui_scope_creep_in_core

Weryfikacja:
  python3 -m py_compile ai_coding_pain_points_research.py: PASS
  python3 ai_coding_pain_points_research.py: PASS
  python3 tools/workflow_guard.py module-ready 002-dependency-extractor: PASS
  cargo test --workspace: PASS, 11 tests

Wniosek:
  typowe bolaczki AI codingu nie sa jeszcze wystarczajaco pokryte przez guard.
  Potrzebujemy kolejnego malego hardeningu przed albo w trakcie 002:
    ignored required test detection
    placeholder pattern detection
    dependency change declaration
    exact public field/type mode
    forbidden std::net/std::process/unsafe/SystemTime/rand in pure core modules
    platform portability guard
    core-layer purity check
```

Eksperyment AI Workflow Deep Risk Matrix wykonany:

```text
Cel:
  sprawdzic glebsze ryzyka AI codingu, ktore nie sa tylko brakiem pliku,
  brakiem testu albo prostym zakazanym patternem.

Lokalizacja:
  experiments/2026-06-09_ai_workflow_deep_risk_matrix/

Testowane klasy ryzyka:
  hypothesis-to-code drift
  MVP scope creep
  refactor behavior change
  documentation-code drift
  cross-module regression
  overengineering
  long-session context conflict
  hidden uncertainty

Wynik:
  scenario_count: 12
  bad_scenario_count: 11
  bad_scenarios_caught_by_current_guard: 2
  bad_scenarios_missed_by_current_guard: 9
  false_rejections_on_good: 0

Guard zlapal:
  direct panic in expected source
  missing ENGINEERING_RULES.md

Guard przepuscil:
  hypothesis_to_code_drift
  mvp_scope_creep
  refactor_changes_behavior_and_tests
  documentation_code_drift
  cross_module_regression
  overengineering
  long_session_context_conflict
  hidden_uncertainty
  code_changes_without_doc_update

Weryfikacja:
  python3 -m py_compile ai_workflow_deep_risk_matrix.py: PASS
  python3 ai_workflow_deep_risk_matrix.py: PASS
  python3 tools/workflow_guard.py module-ready 002-dependency-extractor: PASS
  cargo test --workspace: PASS, 11 tests

Wniosek:
  obecny guard nie jest odporny na glebsze semantyczne ryzyka AI workflow.
  Chroni przed prostymi naruszeniami, ale nie wykrywa jeszcze, czy kod nadal
  respektuje sens decyzji produktowych i domenowych.

Najbardziej wartosciowe kolejne zabezpieczenia:
  scope ownership guard
  hidden uncertainty guard
  exact public contract mode
  refactor mode guard
  cross-module fake consumer test
  hypothesis trace-id guard pozniej, gdy ustalimy format faktow
```

Guard Hardening v0.2 wdrozony:

```text
Cel:
  zamienic wnioski z trzech adversarial AI workflow experiments w realne
  blokady przed budowa 002-dependency-extractor.

Zmiany w tools/workflow_guard.py:
  verify-module moze teraz wymagac odhaczonych obligations
  required tests moga byc sprawdzane pod katem #[ignore]
  required tests moga byc sprawdzane pod katem pustych/tautologicznych body
  public_contract_mode exact blokuje dodatkowe publiczne structy/pola
  public_contract_field_types sprawdza typy publicznych pol
  forbidden_scope_verbs / forbidden_scope_terms lapia scope creep
  forbidden_guarded_source_patterns skanuje szersze core source high-risk patterns
  allowed_dependency_names blokuje niezatwierdzone zaleznosci Cargo
  required_doc_min_chars / required_doc_literals sprawdzaja substance dokumentow
  hidden_uncertainty_gate blokuje frazy typu TBD / Question left open / unclear

Zmiany w specs/002-dependency-extractor/module.contract.json:
  guard_version: 0.2
  required_checked_obligations: true
  required_tests_must_not_be_ignored: true
  required_tests_must_have_substance: true
  public_contract_mode: exact
  public_contract_field_types dodane dla wszystkich publicznych modeli v0.1
  forbidden_scope_verbs / forbidden_scope_terms dodane
  forbidden_source_patterns rozszerzone
  guarded_source_globs / forbidden_guarded_source_patterns dodane
  allowed_dependency_names dodane
  hidden_uncertainty_gate dodany
  required_doc_min_chars / required_doc_literals dodane

Zmiany w docs/workflow:
  ENGINEERING_RULES.md
  AGENTS.md
  PROJECT_WORKFLOW.md
  PRODUCT_SPEC.md
  specs/002-dependency-extractor/plan.md
  specs/002-dependency-extractor/tasks.md

Regresja:
  experiments/2026-06-09_guard_hardening_v0_2_regression/

Wynik regresji:
  scenario_count: 11
  bad_scenario_count: 10
  bad_scenarios_caught_by_v0_2_guard: 10
  bad_scenarios_missed_by_v0_2_guard: 0
  false_rejections_on_good: 0

Zlapane scenariusze:
  unchecked_obligations
  ignored_required_test
  tautological_required_test
  extra_public_field
  wrong_public_type
  scope_creep
  forbidden_core_pattern
  unreviewed_dependency
  hidden_uncertainty
  empty_required_doc

Status 002:
  module-ready nadal PASS, czyli mozna zaczac implementacje.
  verify-module nadal EXPECTED FAIL, bo kod/testy 002 jeszcze nie istnieja
  i obligations nie sa odhaczone.

Wniosek:
  workflow przeszedl z artifact-presence guard v0.1 do substance/quality guard
  v0.2 dla najwazniejszych ryzyk AI codingu przed modulem 002.
```

## 8. Zdanie startowe na kolejna sesje

```text
Nawigator: przeczytaj CURRENT_STATE.md, PRODUCT_SPINE.md, AGENTS.md, AI_CONTRACT.md, PROJECT_NAVIGATOR.md, PROJECT_MAP.md oraz specs/001-als-reader/spec.md, plan.md, tasks.md i fixture-contract.md. ALSReader Contract Review zakonczyl sie decyzja: ALSReadModel v0.2 jest internal downstream contract, CLI JSON jest diagnostic-only, a path existence nalezy do przyszlego PathVerifier. Pierwszy pass implementacji ALSReadModel v0.2 jest zrobiony i testy sa zielone: cargo fmt --check PASS, cargo test PASS 7 tests. Chce teraz zreviewowac output v0.2 na fixture albo wybrac nastepny maly krok.
```

Jesli wracamy do eksperymentu ALS corpus:

```text
Nawigator: przeczytaj experiments/2026-06-02_als_structure_corpus_20/README.md,
findings.md oraz session-digests/2026-06-02_als-structure-corpus-20.md.
Chce uzyc tych wynikow jako dowodu w ALSReader Contract Review.
```
