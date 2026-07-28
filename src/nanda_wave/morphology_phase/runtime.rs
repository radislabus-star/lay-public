use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::corpus::parse_corpus;
use super::field::MorphologyField;

static SHADOW_FIELD: OnceLock<Option<MorphologyField>> = OnceLock::new();

pub(crate) use super::field::SameLemmaSurfaceReadout;

const DEFAULT_SHADOW_CORPUS: &str = "data/morphology/lay_ru_noun_morph_462k_shadow_v1.tsv";

pub(crate) fn default_shadow_corpus_path() -> PathBuf {
    std::env::var_os("LAY_RU_MORPHOLOGY_CORPUS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_SHADOW_CORPUS))
}

pub(crate) fn shadow_same_lemma_surface_readout(
    context_prefix: &str,
    candidate_surfaces: &[String],
) -> Option<SameLemmaSurfaceReadout> {
    let field = SHADOW_FIELD.get_or_init(load_shadow_field).as_ref()?;
    let context = placeholder_context(context_prefix);
    field.same_lemma_surface_readout(&context, candidate_surfaces)
}

fn load_shadow_field() -> Option<MorphologyField> {
    let path = default_shadow_corpus_path();
    load_shadow_field_from_path(&path).ok()
}

fn load_shadow_field_from_path(path: &Path) -> Result<MorphologyField, String> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read morphology corpus {}: {error}",
            path.display()
        )
    })?;
    let corpus = parse_corpus(&text)?;
    MorphologyField::train(&corpus)
}

fn placeholder_context(context_prefix: &str) -> String {
    let words = crate::correction_core::normalized_correction_words(context_prefix);
    if words.is_empty() {
        "_".to_string()
    } else {
        format!("{} _", words.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MICRO: &str = "F\tдом\tдом\tnoun:nom:sg\n\
                         F\tдом\tдома\tnoun:gen:sg\n\
                         F\tстол\tстол\tnoun:nom:sg\n\
                         T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
                         T\tдом\tдома\tnoun:gen:sg\tнет _\n\
                         H\tдом\tдом\tnoun:nom:sg\t_ открыт\n";

    #[test]
    fn placeholder_context_keeps_left_words_and_one_slot() {
        assert_eq!(placeholder_context("нет   больших "), "нет больших _");
        assert_eq!(placeholder_context(""), "_");
    }

    #[test]
    fn same_lemma_runtime_picks_contextual_form() {
        let path = std::env::temp_dir().join(format!(
            "lay-morph-runtime-{}-{}.tsv",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::fs::write(&path, MICRO).expect("write corpus");
        let field = load_shadow_field_from_path(&path).expect("field");
        let _ = std::fs::remove_file(&path);

        let readout = field.same_lemma_surface_readout(
            &placeholder_context("нет "),
            &["дом".to_string(), "дома".to_string(), "стол".to_string()],
        );
        assert_eq!(
            readout,
            Some(SameLemmaSurfaceReadout::Winner {
                winner_surface: "дома".to_string(),
                cohort_surfaces: vec!["дом".to_string(), "дома".to_string()],
            })
        );
    }
}
