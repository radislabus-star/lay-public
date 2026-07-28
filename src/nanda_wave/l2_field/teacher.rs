use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TeacherForm {
    pub(crate) lemma: String,
    pub(crate) surface: String,
    pub(crate) feature_mask: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TeacherScene {
    pub(crate) lemma: String,
    pub(crate) surface: String,
    pub(crate) feature_mask: u32,
    pub(crate) context: String,
    pub(crate) heldout: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TeacherNeighborScene {
    pub(crate) lemma: String,
    pub(crate) surface: String,
    pub(crate) feature_mask: u32,
    pub(crate) context: String,
    pub(crate) competitors: Vec<String>,
    pub(crate) heldout: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct L2TeacherCorpus {
    pub(crate) forms: Vec<TeacherForm>,
    pub(crate) scenes: Vec<TeacherScene>,
    pub(crate) neighbor_scenes: Vec<TeacherNeighborScene>,
}

impl L2TeacherCorpus {
    pub(crate) fn parse_tsv(text: &str) -> Result<Self, String> {
        let mut forms = Vec::new();
        let mut scenes = Vec::new();
        let mut neighbor_scenes = Vec::new();
        for (line_index, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields = line.split('\t').collect::<Vec<_>>();
            let line_number = line_index + 1;
            match fields.as_slice() {
                ["F", lemma, surface, features] => forms.push(TeacherForm {
                    lemma: normalize(lemma),
                    surface: normalize(surface),
                    feature_mask: parse_features(features, line_number)?,
                }),
                ["T", lemma, surface, features, context]
                | ["H", lemma, surface, features, context] => {
                    validate_context(context, line_number)?;
                    scenes.push(TeacherScene {
                        lemma: normalize(lemma),
                        surface: normalize(surface),
                        feature_mask: parse_features(features, line_number)?,
                        context: normalize(context),
                        heldout: fields[0] == "H",
                    });
                }
                ["T", surface, features, context] | ["H", surface, features, context] => {
                    validate_context(context, line_number)?;
                    scenes.push(TeacherScene {
                        lemma: String::new(),
                        surface: normalize(surface),
                        feature_mask: parse_features(features, line_number)?,
                        context: normalize(context),
                        heldout: fields[0] == "H",
                    });
                }
                ["NT", lemma, surface, features, context, competitors]
                | ["NH", lemma, surface, features, context, competitors] => {
                    validate_context(context, line_number)?;
                    let competitors = competitors
                        .split(',')
                        .map(normalize)
                        .filter(|surface| !surface.is_empty())
                        .collect::<Vec<_>>();
                    if competitors.is_empty() {
                        return Err(format!(
                            "line {line_number}: near-neighbor scene requires competitors"
                        ));
                    }
                    neighbor_scenes.push(TeacherNeighborScene {
                        lemma: normalize(lemma),
                        surface: normalize(surface),
                        feature_mask: parse_features(features, line_number)?,
                        context: normalize(context),
                        competitors,
                        heldout: fields[0] == "NH",
                    });
                }
                _ => {
                    return Err(format!(
                        "line {line_number}: expected F/T/H/NT/NH tab-separated morphology row"
                    ));
                }
            }
        }
        if forms.is_empty() {
            return Err("L2 teacher corpus contains no forms".to_string());
        }

        forms.sort_by(|left, right| {
            (&left.lemma, &left.surface, left.feature_mask).cmp(&(
                &right.lemma,
                &right.surface,
                right.feature_mask,
            ))
        });
        forms.dedup();
        resolve_scene_lemmas(&forms, &mut scenes)?;
        validate_neighbor_scenes(&forms, &neighbor_scenes)?;
        scenes.sort_by(|left, right| {
            (
                left.heldout,
                &left.lemma,
                &left.surface,
                left.feature_mask,
                &left.context,
            )
                .cmp(&(
                    right.heldout,
                    &right.lemma,
                    &right.surface,
                    right.feature_mask,
                    &right.context,
                ))
        });
        scenes.dedup();
        neighbor_scenes.sort_by(|left, right| {
            (
                left.heldout,
                &left.lemma,
                &left.surface,
                left.feature_mask,
                &left.context,
                &left.competitors,
            )
                .cmp(&(
                    right.heldout,
                    &right.lemma,
                    &right.surface,
                    right.feature_mask,
                    &right.context,
                    &right.competitors,
                ))
        });
        neighbor_scenes.dedup();
        if !scenes.iter().any(|scene| !scene.heldout) || !scenes.iter().any(|scene| scene.heldout) {
            return Err("L2 teacher corpus requires train and heldout scenes".to_string());
        }
        Ok(Self {
            forms,
            scenes,
            neighbor_scenes,
        })
    }
}

fn validate_neighbor_scenes(
    forms: &[TeacherForm],
    scenes: &[TeacherNeighborScene],
) -> Result<(), String> {
    let bindings = forms
        .iter()
        .map(|form| (&form.lemma, &form.surface, form.feature_mask))
        .collect::<BTreeSet<_>>();
    let surfaces = forms
        .iter()
        .map(|form| form.surface.as_str())
        .collect::<BTreeSet<_>>();
    for scene in scenes {
        if !bindings.contains(&(&scene.lemma, &scene.surface, scene.feature_mask)) {
            return Err(format!(
                "near-neighbor scene references missing target binding lemma={:?} surface={:?}",
                scene.lemma, scene.surface
            ));
        }
        if let Some(missing) = scene
            .competitors
            .iter()
            .find(|surface| !surfaces.contains(surface.as_str()))
        {
            return Err(format!(
                "near-neighbor scene references missing competitor surface {missing:?}"
            ));
        }
    }
    Ok(())
}

fn resolve_scene_lemmas(forms: &[TeacherForm], scenes: &mut [TeacherScene]) -> Result<(), String> {
    let mut lemmas_by_surface_feature = BTreeMap::<(String, u32), BTreeSet<String>>::new();
    let mut bindings = BTreeSet::<(String, String, u32)>::new();
    for form in forms {
        lemmas_by_surface_feature
            .entry((form.surface.clone(), form.feature_mask))
            .or_default()
            .insert(form.lemma.clone());
        bindings.insert((form.lemma.clone(), form.surface.clone(), form.feature_mask));
    }
    for scene in scenes {
        if !scene.lemma.is_empty() {
            if bindings.contains(&(
                scene.lemma.clone(),
                scene.surface.clone(),
                scene.feature_mask,
            )) {
                continue;
            }
            return Err(format!(
                "scene references missing binding lemma={:?} surface={:?}",
                scene.lemma, scene.surface
            ));
        }
        let matching = lemmas_by_surface_feature
            .get(&(scene.surface.clone(), scene.feature_mask))
            .cloned()
            .unwrap_or_default();
        if matching.len() != 1 {
            return Err(format!(
                "scene {:?} has {} matching lemmas; provide explicit lemma",
                scene.surface,
                matching.len()
            ));
        }
        scene.lemma = matching.first().expect("one matching lemma").clone();
    }
    Ok(())
}

fn parse_features(raw: &str, line_number: usize) -> Result<u32, String> {
    crate::nanda_wave::morphology_phase::parse_features(raw)
        .map_err(|error| format!("line {line_number}: {error}"))
}

fn validate_context(context: &str, line_number: usize) -> Result<(), String> {
    if context
        .split_whitespace()
        .filter(|token| *token == "_")
        .count()
        != 1
    {
        return Err(format!(
            "line {line_number}: L2 context requires exactly one _ placeholder"
        ));
    }
    Ok(())
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teacher_resolves_implicit_lemma_and_preserves_heldout() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             T\tдом\tnoun:nom:sg\t_ стоит\n\
             H\tдома\tnoun:gen:sg\tнет _\n",
        )
        .expect("teacher corpus");
        assert_eq!(corpus.forms.len(), 2);
        assert_eq!(corpus.scenes.len(), 2);
        assert_eq!(corpus.scenes[0].lemma, "дом");
        assert!(!corpus.scenes[0].heldout);
        assert!(corpus.scenes[1].heldout);
    }

    #[test]
    fn teacher_keeps_cross_lemma_neighbor_scenes_separate() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tпосмотреть\tпосмотри\tverb:imp:p2:sg:perf\n\
             F\tпросмотреть\tпросмотри\tverb:imp:p2:sg:perf\n\
             T\tпосмотреть\tпосмотри\tverb:imp:p2:sg:perf\t_ сюда\n\
             H\tпросмотреть\tпросмотри\tverb:imp:p2:sg:perf\t_ документ\n\
             NT\tпосмотреть\tпосмотри\tverb:imp:p2:sg:perf\t_ сюда\tпросмотри\n\
             NH\tпросмотреть\tпросмотри\tverb:imp:p2:sg:perf\t_ документ\tпосмотри\n",
        )
        .expect("teacher corpus");
        assert_eq!(corpus.neighbor_scenes.len(), 2);
        assert!(!corpus.neighbor_scenes[0].heldout);
        assert!(corpus.neighbor_scenes[1].heldout);
    }
}
