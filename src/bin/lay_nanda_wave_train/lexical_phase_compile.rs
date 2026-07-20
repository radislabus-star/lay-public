#[derive(Clone, Debug, serde::Serialize)]
struct LexicalPhaseCompileReport {
    output: String,
    source_words: usize,
    l1_centers: usize,
    l1_postings: usize,
    l2_word_centers: usize,
    grapheme_nodes: usize,
    grapheme_arcs: usize,
    decoder_states: usize,
    decoder_arcs: usize,
    training_surfaces: usize,
    bytes: usize,
    raw_word_table: bool,
    corpus_hash: String,
    artifact_checksum: String,
    manifest: String,
}

#[derive(Clone, Debug, serde::Serialize)]
struct LexicalPhaseManifest {
    schema: &'static str,
    artifact_format: u32,
    compiler_version: &'static str,
    include_hunspell: bool,
    include_english: bool,
    source_files: Vec<LexicalPhaseSourceDigest>,
    training_surface_files: Vec<LexicalPhaseSourceDigest>,
    corpus_hash: String,
    artifact_checksum: String,
    source_words: usize,
    training_surfaces: usize,
    artifact_bytes: usize,
    raw_word_table: bool,
}

#[derive(Clone, Debug, serde::Serialize)]
struct LexicalPhaseSourceDigest {
    path: String,
    bytes: usize,
    checksum: String,
}

fn compile_lexical_phase_artifact(
    inputs: &[PathBuf],
    training_surface_inputs: &[PathBuf],
    output: &Path,
    include_hunspell: bool,
    include_english: bool,
) -> io::Result<LexicalPhaseCompileReport> {
    let mut source = Vec::new();
    let mut source_files = Vec::new();
    let mut training_surface_files = Vec::new();
    for input in inputs {
        let text = std::fs::read_to_string(input)?;
        source_files.push(source_digest(input, text.as_bytes()));
        source.extend(text.lines().map(str::to_string));
    }
    if include_english {
        let mut english_sources = 0usize;
        for (path, hunspell) in [(lexicon::EN_HUNSPELL, true), (lexicon::EN_WORDS, false)] {
            let path = Path::new(path);
            let Ok(bytes) = std::fs::read(path) else {
                continue;
            };
            let text = String::from_utf8_lossy(&bytes);
            source_files.push(source_digest(path, &bytes));
            source.extend(english_lexical_surfaces(&text, hunspell));
            english_sources = english_sources.saturating_add(1);
        }
        if english_sources == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "--include-english requested but no English word source is installed",
            ));
        }
    }
    // Source words become L2 terminals. Extra training surfaces only shape the
    // compact decoder, so inflected forms can improve reconstruction without
    // becoming a second runtime word table.
    let mut training = source.clone();
    for input in training_surface_inputs {
        let text = std::fs::read_to_string(input)?;
        training_surface_files.push(source_digest(input, text.as_bytes()));
        training.extend(text.lines().map(str::to_string));
    }
    if include_hunspell {
        for path in [lexicon::RU_HUNSPELL, lexicon::RU_HUNSPELL_AFF] {
            let bytes = std::fs::read(path)?;
            source_files.push(source_digest(Path::new(path), &bytes));
        }
        training.extend(russian_lexicon::russian_tiny_dictionary().iter().cloned());
        training.extend(
            russian_lexicon::russian_generated_form_dictionary()
                .iter()
                .cloned(),
        );
    }
    let bytes = lexical_phase_compiler::compile_words_with_training(
        source.iter().map(String::as_str),
        training.iter().map(String::as_str),
    )
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = output.with_extension("bin.tmp");
    std::fs::write(&temporary, &bytes)?;
    std::fs::rename(&temporary, output)?;
    let header = format::read_header(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let manifest_path = output.with_extension("manifest.json");
    let report = LexicalPhaseCompileReport {
        output: output.display().to_string(),
        source_words: header.source_words as usize,
        l1_centers: header.center_count as usize,
        l1_postings: header.posting_count as usize,
        l2_word_centers: header.terminal_count as usize,
        grapheme_nodes: header.node_count as usize,
        grapheme_arcs: header.arc_count as usize,
        decoder_states: header.decoder_state_count as usize,
        decoder_arcs: header.decoder_arc_count as usize,
        training_surfaces: header.training_surfaces as usize,
        bytes: bytes.len(),
        raw_word_table: false,
        corpus_hash: format!("{:016x}", header.corpus_hash),
        artifact_checksum: format!("{:016x}", header.checksum),
        manifest: manifest_path.display().to_string(),
    };
    let manifest = LexicalPhaseManifest {
        schema: "lay.lexical-phase-manifest.v2",
        artifact_format: format::VERSION,
        compiler_version: env!("CARGO_PKG_VERSION"),
        include_hunspell,
        include_english,
        source_files,
        training_surface_files,
        corpus_hash: report.corpus_hash.clone(),
        artifact_checksum: report.artifact_checksum.clone(),
        source_words: report.source_words,
        training_surfaces: report.training_surfaces,
        artifact_bytes: report.bytes,
        raw_word_table: false,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(io::Error::other)?;
    let manifest_temporary = manifest_path.with_extension("json.tmp");
    std::fs::write(&manifest_temporary, manifest_bytes)?;
    std::fs::rename(manifest_temporary, &manifest_path)?;
    Ok(report)
}

fn run_lexical_phase_compile(args: &[String]) -> io::Result<()> {
    let include_hunspell = args.iter().any(|arg| arg == "--include-hunspell");
    let include_english = args.iter().any(|arg| arg == "--include-english");
    let output_index = args.iter().position(|arg| arg == "--out").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: lay-nanda-wave-train --compile-lexical-phase --out ARTIFACT CORPUS...",
        )
    })?;
    let output = args
        .get(output_index + 1)
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing --out value"))?;
    let training_surface_inputs = lexical_phase_training_surface_inputs(args)?;
    let mut excluded_indices = std::collections::BTreeSet::from([output_index, output_index + 1]);
    for (index, value) in args.iter().enumerate() {
        if value == "--training-surfaces" {
            excluded_indices.insert(index);
            excluded_indices.insert(index + 1);
        }
    }
    let inputs = args
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(index, value)| {
            !excluded_indices.contains(index)
                && value.as_str() != "--compile-lexical-phase"
                && value.as_str() != "--include-hunspell"
                && value.as_str() != "--include-english"
        })
        .map(|(_, value)| PathBuf::from(value))
        .collect::<Vec<_>>();
    if inputs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "at least one corpus path is required",
        ));
    }
    let report = compile_lexical_phase_artifact(
        &inputs,
        &training_surface_inputs,
        &output,
        include_hunspell,
        include_english,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(io::Error::other)?
    );
    Ok(())
}

fn lexical_phase_training_surface_inputs(args: &[String]) -> io::Result<Vec<PathBuf>> {
    let mut inputs = Vec::new();
    for (index, argument) in args.iter().enumerate() {
        if argument != "--training-surfaces" {
            continue;
        }
        let path = args.get(index + 1).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--training-surfaces requires a corpus path",
            )
        })?;
        if path.starts_with("--") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--training-surfaces requires a corpus path",
            ));
        }
        inputs.push(PathBuf::from(path));
    }
    Ok(inputs)
}

fn run_export_generated_russian_forms(args: &[String]) -> io::Result<()> {
    let output = arg_path(args, "--out").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: lay-nanda-wave-train --export-generated-russian-forms --out FORMS.txt",
        )
    })?;
    let forms = russian_lexicon::russian_generated_form_dictionary()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if forms.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "generated Russian forms are unavailable; build with --features lexical-compiler and install Hunspell dictionaries",
        ));
    }
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = output.with_extension("txt.tmp");
    std::fs::write(&temporary, format!("{}\n", forms.join("\n")))?;
    std::fs::rename(&temporary, &output)?;
    println!(
        "{}",
        serde_json::json!({
            "output": output,
            "generated_forms": forms.len(),
            "raw_word_table": false,
        })
    );
    Ok(())
}

fn english_lexical_surfaces(text: &str, hunspell: bool) -> impl Iterator<Item = String> + '_ {
    text.lines()
        .skip(usize::from(hunspell))
        .filter_map(move |line| {
            let raw = if hunspell {
                line.trim()
                    .split_once('/')
                    .map_or(line.trim(), |(word, _)| word)
            } else {
                line.trim()
            };
            let word = raw.to_ascii_lowercase();
            ((2..=format::MAX_WORD_CHARS).contains(&word.chars().count())
                && word
                    .chars()
                    .all(|ch| ch.is_ascii_alphabetic() || ch == '-' || ch == '\''))
            .then_some(word)
        })
}

fn source_digest(path: &Path, bytes: &[u8]) -> LexicalPhaseSourceDigest {
    LexicalPhaseSourceDigest {
        path: path.display().to_string(),
        bytes: bytes.len(),
        checksum: format!("{:016x}", format::checksum(bytes)),
    }
}

#[cfg(test)]
mod lexical_phase_compile_tests {
    use super::lexical_phase_training_surface_inputs;
    use std::path::PathBuf;

    #[test]
    fn training_surface_paths_are_not_source_inputs() {
        let args = vec![
            "lay-nanda-wave-train".to_string(),
            "--compile-lexical-phase".to_string(),
            "--training-surfaces".to_string(),
            "forms.txt".to_string(),
            "--out".to_string(),
            "field.bin".to_string(),
            "words.txt".to_string(),
        ];
        assert_eq!(
            lexical_phase_training_surface_inputs(&args).unwrap(),
            vec![PathBuf::from("forms.txt")]
        );
    }

    #[test]
    fn training_surface_flag_requires_a_path() {
        let args = vec![
            "lay-nanda-wave-train".to_string(),
            "--training-surfaces".to_string(),
        ];
        assert!(lexical_phase_training_surface_inputs(&args).is_err());
    }
}
