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
    source_files: Vec<LexicalPhaseSourceDigest>,
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
    output: &Path,
    include_hunspell: bool,
) -> io::Result<LexicalPhaseCompileReport> {
    let mut source = Vec::new();
    let mut source_files = Vec::new();
    for input in inputs {
        let text = std::fs::read_to_string(input)?;
        source_files.push(source_digest(input, text.as_bytes()));
        source.extend(text.lines().map(str::to_string));
    }
    let mut training = source.clone();
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
        source_files,
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
    let inputs = args
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(index, value)| {
            *index != output_index
                && *index != output_index + 1
                && value.as_str() != "--compile-lexical-phase"
                && value.as_str() != "--include-hunspell"
        })
        .map(|(_, value)| PathBuf::from(value))
        .collect::<Vec<_>>();
    if inputs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "at least one corpus path is required",
        ));
    }
    let report = compile_lexical_phase_artifact(&inputs, &output, include_hunspell)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(io::Error::other)?
    );
    Ok(())
}

fn source_digest(path: &Path, bytes: &[u8]) -> LexicalPhaseSourceDigest {
    LexicalPhaseSourceDigest {
        path: path.display().to_string(),
        bytes: bytes.len(),
        checksum: format!("{:016x}", format::checksum(bytes)),
    }
}
