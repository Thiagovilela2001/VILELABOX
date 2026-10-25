use crate::{parse_ppth, sections, track_from_anlz, AnlzError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "aif", "aiff", "m4a"];

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub anlz_dir: PathBuf,
    pub music_dir: PathBuf,
    pub output_dir: PathBuf,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportIndexTrack {
    pub sha256: String,
    pub bpm: Option<f64>,
    pub first_beat_time_s: Option<f64>,
    pub cues_count: usize,
    pub phrases_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportIndex {
    pub count: usize,
    pub with_phrases: usize,
    pub with_cues: usize,
    pub missing_audio: usize,
    pub tracks: Vec<ExportIndexTrack>,
}

/// Calcula o SHA-256 de um arquivo por streaming em blocos de 64 KiB.
pub fn compute_file_sha256(path: &Path) -> Result<String, AnlzError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    Ok(hex)
}

fn canonical_or_projected(path: &Path) -> Result<PathBuf, AnlzError> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let mut normalized = PathBuf::new();
    for comp in abs.components() {
        match comp {
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(AnlzError::CaminhoInvalido(format!(
                        "caminho escapa da raiz: {}",
                        path.display()
                    )));
                }
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }

    let mut existing = normalized.as_path();
    let mut tail = Vec::new();
    while !existing.exists() {
        let Some(name) = existing.file_name() else {
            break;
        };
        tail.push(name.to_os_string());
        let Some(parent) = existing.parent() else {
            break;
        };
        existing = parent;
    }

    let mut resolved = if existing.exists() {
        fs::canonicalize(existing)?
    } else {
        normalized
    };
    for part in tail.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}

fn paths_overlap(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

fn is_within_root(root: &Path, candidate: &Path) -> bool {
    fs::canonicalize(candidate).is_ok_and(|canon| canon.starts_with(root))
}

fn relative_slash(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    Some(rel.to_string_lossy().replace('\\', "/"))
}

fn collect_dat_files(
    root: &Path,
    dir: &Path,
    out: &mut Vec<(String, PathBuf)>,
) -> Result<(), AnlzError> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if !is_within_root(root, &path) {
            continue;
        }
        if file_type.is_dir() {
            collect_dat_files(root, &path, out)?;
        } else if file_type.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("dat"))
        {
            if let Some(rel) = relative_slash(root, &path) {
                out.push((rel, path));
            }
        }
    }
    Ok(())
}

fn find_sibling_ext(root: &Path, dat_path: &Path) -> Option<PathBuf> {
    for ext in ["EXT", "ext"] {
        let candidate = dat_path.with_extension(ext);
        if candidate.is_file()
            && !fs::symlink_metadata(&candidate).is_ok_and(|m| m.file_type().is_symlink())
            && is_within_root(root, &candidate)
        {
            return Some(candidate);
        }
    }
    None
}

struct MusicIndex {
    by_rel_lower: HashMap<String, PathBuf>,
    // `None` representa uma chave ambígua. Nunca escolhemos silenciosamente
    // uma faixa quando nomes iguais aparecem em diretórios diferentes.
    by_name_lower: HashMap<String, Option<PathBuf>>,
    by_stem_lower: HashMap<String, Option<PathBuf>>,
}

impl MusicIndex {
    fn build(music_root: &Path) -> Result<Self, AnlzError> {
        let mut files = Vec::new();
        Self::walk(music_root, music_root, &mut files)?;
        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut by_rel_lower = HashMap::new();
        let mut by_name_lower = HashMap::new();
        let mut by_stem_lower = HashMap::new();

        for (rel, path) in files {
            by_rel_lower.insert(rel.to_lowercase(), path.clone());
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                Self::insert_unique(&mut by_name_lower, name.to_lowercase(), path.clone());
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                Self::insert_unique(&mut by_stem_lower, stem.to_lowercase(), path);
            }
        }

        Ok(Self {
            by_rel_lower,
            by_name_lower,
            by_stem_lower,
        })
    }

    fn insert_unique(index: &mut HashMap<String, Option<PathBuf>>, key: String, path: PathBuf) {
        use std::collections::hash_map::Entry;
        match index.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(Some(path));
            }
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = None;
            }
        }
    }

    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<(), AnlzError> {
        let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if !is_within_root(root, &path) {
                continue;
            }
            if file_type.is_dir() {
                Self::walk(root, &path, out)?;
            } else if file_type.is_file() {
                let is_audio = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|ext| {
                        AUDIO_EXTENSIONS
                            .iter()
                            .any(|allowed| ext.eq_ignore_ascii_case(allowed))
                    });
                if is_audio {
                    if let Some(rel) = relative_slash(root, &path) {
                        out.push((rel, path));
                    }
                }
            }
        }
        Ok(())
    }

    fn resolve(&self, raw_ppth: Option<&str>) -> Option<&Path> {
        let raw = raw_ppth?.trim();
        if raw.is_empty() {
            return None;
        }
        let clean = raw.replace("?/", "").replace('\\', "/");
        let rel_key = clean.trim_start_matches('/').to_lowercase();
        if let Some(found) = self.by_rel_lower.get(&rel_key) {
            return Some(found.as_path());
        }

        let file_name = clean.rsplit('/').next()?.trim();
        if file_name.is_empty() {
            return None;
        }
        let name_key = file_name.to_lowercase();
        if let Some(found) = self.by_name_lower.get(&name_key) {
            return found.as_deref();
        }

        let stem = file_name
            .rsplit_once('.')
            .map_or(file_name, |(prefix, _)| prefix);
        if stem.is_empty() {
            return None;
        }
        let stem_key = stem.to_lowercase();
        self.by_stem_lower.get(&stem_key).and_then(Option::as_deref)
    }
}

fn write_atomic(output_dir: &Path, file_name: &str, content: &str) -> Result<PathBuf, AnlzError> {
    let final_path = output_dir.join(file_name);
    let tmp_path = output_dir.join(format!(".tmp-{file_name}"));
    let write_res = (|| -> Result<(), std::io::Error> {
        let mut file = File::create(&tmp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&tmp_path, &final_path)?;
        Ok(())
    })();

    if let Err(err) = write_res {
        let _ = fs::remove_file(&tmp_path);
        return Err(AnlzError::Io(err));
    }
    Ok(final_path)
}

pub fn export_anlz(options: &ExportOptions) -> Result<ExportIndex, AnlzError> {
    if !options.anlz_dir.is_dir() {
        return Err(AnlzError::CaminhoInvalido(format!(
            "diretório ANLZ inexistente ou inválido: {}",
            options.anlz_dir.display()
        )));
    }
    if !options.music_dir.is_dir() {
        return Err(AnlzError::CaminhoInvalido(format!(
            "diretório de música inexistente ou inválido: {}",
            options.music_dir.display()
        )));
    }

    let anlz_root = fs::canonicalize(&options.anlz_dir)?;
    let music_root = fs::canonicalize(&options.music_dir)?;
    let output_root = canonical_or_projected(&options.output_dir)?;

    if paths_overlap(&anlz_root, &output_root) {
        return Err(AnlzError::CaminhoInvalido(
            "diretórios --anlz-dir e --output não podem se sobrepor".into(),
        ));
    }
    if paths_overlap(&music_root, &output_root) {
        return Err(AnlzError::CaminhoInvalido(
            "diretórios --music-dir e --output não podem se sobrepor".into(),
        ));
    }

    fs::create_dir_all(&options.output_dir)?;
    let output_root = fs::canonicalize(&options.output_dir)?;

    let music_index = MusicIndex::build(&music_root)?;
    let mut dat_files = Vec::new();
    collect_dat_files(&anlz_root, &anlz_root, &mut dat_files)?;
    dat_files.sort_by(|a, b| a.0.cmp(&b.0));
    if let Some(limit) = options.limit {
        dat_files.truncate(limit);
    }

    let mut staged_files = Vec::new();
    let result = (|| -> Result<ExportIndex, AnlzError> {
        let mut tracks = Vec::new();
        let mut missing_audio = 0usize;
        let mut with_phrases = 0usize;
        let mut with_cues = 0usize;

        for (source_ref, dat_path) in dat_files {
            let dat_bytes = fs::read(&dat_path)?;
            let ext_bytes = find_sibling_ext(&anlz_root, &dat_path)
                .map(fs::read)
                .transpose()?;

            let dat_secs = sections(&dat_bytes);
            let ext_secs = ext_bytes.as_deref().map(sections).unwrap_or_default();
            let raw_ppth = ext_secs
                .get(b"PPTH")
                .or_else(|| dat_secs.get(b"PPTH"))
                .and_then(|payload| parse_ppth(payload));

            let Some(audio_path) = music_index.resolve(raw_ppth.as_deref()) else {
                missing_audio += 1;
                continue;
            };

            let sha256 = compute_file_sha256(audio_path)?;
            let track = track_from_anlz(&dat_bytes, ext_bytes.as_deref(), &sha256, &source_ref)?;

            if !track.phrases.is_empty() {
                with_phrases += 1;
            }
            if !track.cues.is_empty() {
                with_cues += 1;
            }

            let track_json = track.to_json_str()?;
            let file_name = format!("{sha256}.json");
            let staged_name = format!("pending-{file_name}");
            let staged = write_atomic(&output_root, &staged_name, &track_json)?;
            staged_files.push((staged, output_root.join(file_name)));

            tracks.push(ExportIndexTrack {
                sha256,
                bpm: track.bpm,
                first_beat_time_s: track.first_beat_time_s,
                cues_count: track.cues.len(),
                phrases_count: track.phrases.len(),
            });
        }

        tracks.sort_by(|a, b| a.sha256.cmp(&b.sha256));
        let index = ExportIndex {
            count: tracks.len(),
            with_phrases,
            with_cues,
            missing_audio,
            tracks,
        };
        // Publica os conteúdos primeiro e o índice por último. Até esse ponto,
        // uma falha preserva integralmente a exportação válida anterior.
        for (staged, final_path) in &staged_files {
            fs::rename(staged, final_path)?;
        }
        let index_json = serde_json::to_string_pretty(&index)?;
        write_atomic(&output_root, "index.json", &index_json)?;
        Ok(index)
    })();

    if result.is_err() {
        for (staged, _) in staged_files {
            let _ = fs::remove_file(staged);
        }
    }

    result
}
