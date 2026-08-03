mod als_reader;
mod als_reader_impl;
mod als_reader_io;
mod dependency_extractor;
mod dependency_extractor_impl;
mod models;
mod path_observation;
mod path_observation_candidate;
mod path_observation_fs;
mod path_observation_impl;
mod path_observation_path;
mod path_parser;
mod path_text;
mod rewrite_compatibility;

pub use als_reader::{
    analyze_als, ALS_READER_VERSION, ALS_READ_MODEL_VERSION, MAX_COMPRESSED_ALS_BYTES,
    MAX_DECOMPRESSED_XML_BYTES,
};
pub use dependency_extractor::{
    extract_dependencies, DependencyExtractionError, DependencyExtractionMetadata,
    DependencyExtractionResult, DependencyExtractionWarning, DependencyRef, IgnoredInputSummary,
    DEPENDENCY_EXTRACTOR_VERSION, DEPENDENCY_REF_VERSION,
};
pub use models::{
    ALSError, ALSReadError, ALSReadModel, ALSReadWarning, ActiveAudioReference,
    HistoricalReference, NonAudioDependencySignal, SetMetadata,
};
pub use path_observation::{
    observe_dependency_paths, CandidatePathObservation, DependencyPathObservation,
    PathObservationContext, PathObservationError, PathObservationMetadata, PathObservationResult,
    PathObservationWarning, PATH_OBSERVATION_MODEL_VERSION, PATH_OBSERVER_VERSION,
};
pub use path_parser::{parse_als_path, AlsPathKind, ParsedAlsPath, RawAlsPath};
pub use rewrite_compatibility::{
    assess_rewrite_compatibility, is_confirmed_live_11_3_document, reference_is_lab_compatible,
    RewriteCompatibilityAssessment, RewriteReferenceCompatibility,
    REWRITE_COMPATIBILITY_SCHEMA_VERSION,
};
