use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub struct TempAls {
    path: PathBuf,
}

impl TempAls {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempAls {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn gzip_als(name: &str, xml: &str) -> TempAls {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(xml.as_bytes())
        .expect("synthetic ALS XML should compress");
    let bytes = encoder
        .finish()
        .expect("synthetic ALS compression should finish");
    write_temp_als(name, &bytes)
}

#[allow(dead_code)]
pub fn raw_als(name: &str, bytes: &[u8]) -> TempAls {
    write_temp_als(name, bytes)
}

pub fn synthetic_als(
    name: &str,
    active_relative_path_types: &[&str],
    historical_count: usize,
) -> TempAls {
    let mut body = String::from(
        r#"<Ableton MajorVersion="5" MinorVersion="12" SchemaChangeCount="1" Creator="Ableton Live">
  <LiveSet>
"#,
    );

    for (index, relative_path_type) in active_relative_path_types.iter().enumerate() {
        body.push_str(&format!(
            r#"    <SampleRef>
      <FileRef>
        <Path Value="/Synthetic/Sample_{index}.wav" />
        <RelativePath Value="Samples/Imported/Sample_{index}.wav" />
        <RelativePathType Value="{relative_path_type}" />
        <Type Value="2" />
        <OriginalFileSize Value="{}" />
        <OriginalCrc Value="{}" />
      </FileRef>
      <DefaultDuration Value="44.1" />
      <DefaultSampleRate Value="44100" />
    </SampleRef>
"#,
            1000 + index,
            2000 + index
        ));
    }

    if historical_count > 0 {
        body.push_str("    <SourceContext>\n");
        for index in 0..historical_count {
            body.push_str(&format!(
                r#"      <OriginalFileRef>
        <FileRef>
          <Path Value="/Historical/Sample_{index}.wav" />
          <RelativePath Value="Old/Sample_{index}.wav" />
          <RelativePathType Value="1" />
          <OriginalFileSize Value="{}" />
          <OriginalCrc Value="{}" />
        </FileRef>
      </OriginalFileRef>
"#,
                3000 + index,
                4000 + index
            ));
        }
        body.push_str("    </SourceContext>\n");
    }

    body.push_str("  </LiveSet>\n</Ableton>");
    gzip_als(name, &body)
}

fn write_temp_als(name: &str, bytes: &[u8]) -> TempAls {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "rescue_{name}_{}_{}_{}.als",
        std::process::id(),
        timestamp,
        sequence
    ));
    fs::write(&path, bytes).expect("synthetic ALS should be writable");
    TempAls { path }
}
