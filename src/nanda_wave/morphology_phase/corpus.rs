use std::collections::{BTreeMap, BTreeSet};

use super::{parse_features, MorphBinding16};

#[derive(Clone, Debug)]
pub(super) struct MorphExample {
    pub(super) lemma_id: u32,
    pub(super) surface: String,
    pub(super) features: u32,
    pub(super) context: String,
}

#[derive(Clone, Debug)]
pub(super) struct MorphCorpus {
    pub(super) bindings: Vec<MorphBinding16>,
    pub(super) surfaces: Vec<String>,
    pub(super) lemmas: Vec<String>,
    pub(super) train: Vec<MorphExample>,
    pub(super) heldout: Vec<MorphExample>,
}

#[derive(Clone, Debug)]
struct FormRow {
    lemma: String,
    surface: String,
    features: u32,
}

#[derive(Clone, Debug)]
struct ExampleRow {
    lemma: Option<String>,
    surface: String,
    features: u32,
    context: String,
}

pub(super) fn parse_corpus(text: &str) -> Result<MorphCorpus, String> {
    let mut forms = Vec::new();
    let mut train_rows = Vec::new();
    let mut heldout_rows = Vec::new();
    for (line_index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        let line_number = line_index + 1;
        match fields.as_slice() {
            ["F", lemma, surface, features] => forms.push(FormRow {
                lemma: normalize(lemma),
                surface: normalize(surface),
                features: parse_features(features)
                    .map_err(|error| format!("line {line_number}: {error}"))?,
            }),
            ["T", surface, features, context] => train_rows.push(ExampleRow {
                lemma: None,
                surface: normalize(surface),
                features: parse_features(features)
                    .map_err(|error| format!("line {line_number}: {error}"))?,
                context: normalize_context(context, line_number)?,
            }),
            ["H", surface, features, context] => heldout_rows.push(ExampleRow {
                lemma: None,
                surface: normalize(surface),
                features: parse_features(features)
                    .map_err(|error| format!("line {line_number}: {error}"))?,
                context: normalize_context(context, line_number)?,
            }),
            ["T", lemma, surface, features, context] => train_rows.push(ExampleRow {
                lemma: Some(normalize(lemma)),
                surface: normalize(surface),
                features: parse_features(features)
                    .map_err(|error| format!("line {line_number}: {error}"))?,
                context: normalize_context(context, line_number)?,
            }),
            ["H", lemma, surface, features, context] => heldout_rows.push(ExampleRow {
                lemma: Some(normalize(lemma)),
                surface: normalize(surface),
                features: parse_features(features)
                    .map_err(|error| format!("line {line_number}: {error}"))?,
                context: normalize_context(context, line_number)?,
            }),
            _ => {
                return Err(format!(
                    "line {line_number}: expected F/T/H tab-separated morphology row"
                ));
            }
        }
    }
    if forms.is_empty() || train_rows.is_empty() || heldout_rows.is_empty() {
        return Err("morphology corpus requires forms, train and heldout rows".to_string());
    }

    let lemma_names = forms
        .iter()
        .map(|row| row.lemma.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let lemma_ids = lemma_names
        .iter()
        .enumerate()
        .map(|(index, lemma)| (lemma.clone(), index as u32))
        .collect::<BTreeMap<_, _>>();
    let surface_names = forms
        .iter()
        .map(|row| row.surface.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let surface_ids = surface_names
        .iter()
        .enumerate()
        .map(|(index, surface)| (surface.clone(), index as u32))
        .collect::<BTreeMap<_, _>>();
    let surfaces = surface_names;

    let mut bindings = forms
        .iter()
        .map(|row| MorphBinding16 {
            form_center_id: surface_ids[&row.surface],
            lemma_center_id: lemma_ids[&row.lemma],
            features: row.features,
            support: 0,
            phase: 0,
            flags: 0,
        })
        .collect::<Vec<_>>();
    bindings.sort_unstable_by_key(|binding| {
        (
            binding.lemma_center_id,
            binding.form_center_id,
            binding.features,
        )
    });
    bindings.dedup_by_key(|binding| {
        (
            binding.lemma_center_id,
            binding.form_center_id,
            binding.features,
        )
    });
    let mut binding_lemmas_by_form_slot = BTreeMap::<(u32, u32), Vec<u32>>::new();
    for binding in &bindings {
        binding_lemmas_by_form_slot
            .entry((binding.form_center_id, binding.features))
            .or_default()
            .push(binding.lemma_center_id);
    }
    for matching_lemmas in binding_lemmas_by_form_slot.values_mut() {
        matching_lemmas.sort_unstable();
        matching_lemmas.dedup();
    }

    let train = resolve_examples(
        train_rows,
        &lemma_ids,
        &surface_ids,
        &binding_lemmas_by_form_slot,
        "train",
    )?;
    let heldout = resolve_examples(
        heldout_rows,
        &lemma_ids,
        &surface_ids,
        &binding_lemmas_by_form_slot,
        "heldout",
    )?;

    Ok(MorphCorpus {
        bindings,
        surfaces,
        lemmas: lemma_names,
        train,
        heldout,
    })
}

fn resolve_examples(
    examples: Vec<ExampleRow>,
    lemma_ids: &BTreeMap<String, u32>,
    surface_ids: &BTreeMap<String, u32>,
    binding_lemmas_by_form_slot: &BTreeMap<(u32, u32), Vec<u32>>,
    split: &str,
) -> Result<Vec<MorphExample>, String> {
    examples
        .into_iter()
        .map(|example| {
            let explicit_lemma = example
                .lemma
                .as_ref()
                .map(|lemma| {
                    lemma_ids
                        .get(lemma)
                        .copied()
                        .ok_or_else(|| format!("{split} example references unknown lemma {lemma:?}"))
                })
                .transpose()?;
            let form_center_id = surface_ids.get(&example.surface).copied().ok_or_else(|| {
                format!(
                    "{split} example references unknown surface {:?}",
                    example.surface
                )
            })?;
            let matching_lemmas = binding_lemmas_by_form_slot
                .get(&(form_center_id, example.features))
                .map(Vec::as_slice)
                .unwrap_or_default();
            let lemma_id = match explicit_lemma {
                Some(lemma_id) if matching_lemmas.binary_search(&lemma_id).is_ok() => lemma_id,
                Some(_) => {
                    return Err(format!(
                        "{split} example {:?} has no binding for its explicit lemma",
                        example.surface
                    ));
                }
                None if matching_lemmas.len() == 1 => matching_lemmas[0],
                None => {
                    return Err(format!(
                        "{split} example {:?} has {} matching lemma bindings; provide an explicit lemma",
                        example.surface,
                        matching_lemmas.len()
                    ));
                }
            };
            Ok(MorphExample {
                lemma_id,
                surface: example.surface,
                features: example.features,
                context: example.context,
            })
        })
        .collect()
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn normalize_context(context: &str, line_number: usize) -> Result<String, String> {
    let normalized = normalize(context);
    if normalized
        .split_whitespace()
        .filter(|token| *token == "_")
        .count()
        != 1
    {
        return Err(format!(
            "line {line_number}: morphology context requires exactly one _ placeholder"
        ));
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_preserves_syncretic_surface_bindings() {
        let corpus = parse_corpus(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдом\tnoun:acc:sg\n\
             T\tдом\tnoun:nom:sg\t_ стоит\n\
             H\tдом\tnoun:acc:sg\tвижу _\n",
        )
        .expect("valid corpus");
        assert_eq!(corpus.surfaces, vec!["дом"]);
        assert_eq!(corpus.bindings.len(), 2);
    }

    #[test]
    fn parser_preserves_multiple_surfaces_for_one_slot() {
        let corpus = parse_corpus(
            "F\tучитель\tучителя\tnoun:nom:pl\n\
             F\tучитель\tучители\tnoun:nom:pl\n\
             T\tучитель\tучителя\tnoun:nom:pl\t_ появились\n\
             H\tучитель\tучители\tnoun:nom:pl\t_ находятся здесь\n",
        )
        .expect("valid multi-surface corpus");
        assert_eq!(corpus.surfaces, vec!["учители", "учителя"]);
        assert_eq!(corpus.bindings.len(), 2);
    }
}
