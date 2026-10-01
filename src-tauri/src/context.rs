use crate::domain::*;
use chrono::Utc;
use std::collections::HashSet;
use uuid::Uuid;

fn words(value: &str) -> HashSet<String> {
    value
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| word.len() > 2)
        .map(str::to_lowercase)
        .collect()
}

fn estimated_tokens(value: &str) -> usize {
    value.chars().count().div_ceil(4)
}

pub fn assemble(
    project: &Project,
    sources: &[Source],
    prompt: &str,
    max_tokens: usize,
) -> WorkingContext {
    let query = words(&format!("{} {}", project.objective, prompt));
    let mut ranked: Vec<(&Source, f32)> = sources
        .iter()
        .filter(|source| {
            source.project_id == project.id
                && !matches!(
                    source.status,
                    SourceStatus::Excluded | SourceStatus::Quarantined
                )
        })
        .map(|source| {
            let body = words(&format!("{} {}", source.name, source.content));
            let overlap = query.intersection(&body).count() as f32;
            let boost = if source.canonical { 8.0 } else { 0.0 }
                + if source.status == SourceStatus::Gold {
                    4.0
                } else {
                    0.0
                };
            (source, overlap + boost)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));

    let reserved = estimated_tokens(prompt)
        + project
            .constraints
            .iter()
            .map(|v| estimated_tokens(v))
            .sum::<usize>();
    let budget = max_tokens.saturating_sub(reserved).max(256);
    let mut used = 0;
    let mut included = vec![];
    let mut not_included = vec![];
    for (source, score) in ranked {
        let excerpt: String = source.content.chars().take(3200).collect();
        let tokens = estimated_tokens(&excerpt);
        if score <= 0.0 && !source.canonical {
            not_included.push(ExcludedContextItem {
                source_id: source.id,
                name: source.name.clone(),
                reason: "low relevance".into(),
            });
        } else if used + tokens > budget {
            not_included.push(ExcludedContextItem {
                source_id: source.id,
                name: source.name.clone(),
                reason: "over budget".into(),
            });
        } else {
            used += tokens;
            included.push(ContextItem {
                source_id: source.id,
                name: source.name.clone(),
                excerpt,
                reason: if source.canonical {
                    "canonical for this project".into()
                } else {
                    "matches the objective and current request".into()
                },
                score,
                tokens,
                canonical: source.canonical,
            });
        }
    }
    for source in sources.iter().filter(|source| {
        source.project_id == project.id
            && matches!(
                source.status,
                SourceStatus::Excluded | SourceStatus::Quarantined
            )
    }) {
        not_included.push(ExcludedContextItem {
            source_id: source.id,
            name: source.name.clone(),
            reason: if source.status == SourceStatus::Quarantined {
                "quarantined".into()
            } else {
                "excluded by you".into()
            },
        });
    }
    WorkingContext {
        id: Uuid::new_v4(),
        project_id: project.id,
        objective: project.objective.clone(),
        constraints: project.constraints.clone(),
        decisions: project
            .memories
            .iter()
            .filter(|m| m.kind == "decision")
            .map(|m| m.text.clone())
            .collect(),
        open_threads: project
            .memories
            .iter()
            .filter(|m| m.kind == "thread")
            .map(|m| m.text.clone())
            .collect(),
        included,
        not_included,
        used_tokens: used + reserved,
        max_tokens,
        created_at: Utc::now(),
    }
}

/// Same request, three levels of steering: nothing, the project's rules, the full Working Context.
pub fn comparison_variants(
    context: &WorkingContext,
) -> Vec<(&'static str, &'static str, WorkingContext)> {
    let mut bare = context.clone();
    bare.objective.clear();
    bare.constraints.clear();
    bare.decisions.clear();
    bare.included.clear();
    let mut rules = context.clone();
    rules.included.clear();
    vec![
        ("bare", "No context", bare),
        ("rules", "Project rules only", rules),
        ("full", "Full Working Context", context.clone()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarantine_never_enters_context() {
        let now = Utc::now();
        let project = Project {
            id: Uuid::new_v4(),
            name: "Test".into(),
            space: Space::Writer,
            objective: "water board".into(),
            constraints: vec![],
            memories: vec![],
            documents: vec![],
            created_at: now,
            updated_at: now,
        };
        let source = Source {
            id: Uuid::new_v4(),
            project_id: project.id,
            space: Space::Writer,
            name: "secret".into(),
            path: "secret.txt".into(),
            content: "water board password".into(),
            status: SourceStatus::Quarantined,
            canonical: true,
            sha256: "x".into(),
            created_at: now,
            updated_at: now,
        };
        let result = assemble(&project, &[source], "water", 1000);
        assert!(result.included.is_empty());
        assert_eq!(result.not_included[0].reason, "quarantined");
    }

    #[test]
    fn comparison_variants_strip_steering_in_order() {
        let source_id = Uuid::new_v4();
        let context = WorkingContext {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            objective: "Explain the budget".into(),
            constraints: vec!["No jargon".into()],
            decisions: vec!["Use 2026 figures".into()],
            included: vec![ContextItem {
                source_id,
                name: "budget.md".into(),
                excerpt: "Spending rose".into(),
                reason: "matches budget".into(),
                score: 1.0,
                tokens: 4,
                canonical: true,
            }],
            not_included: vec![],
            open_threads: vec![],
            used_tokens: 4,
            max_tokens: 1000,
            created_at: Utc::now(),
        };
        let variants = comparison_variants(&context);
        let ids: Vec<_> = variants.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, ["bare", "rules", "full"]);
        let (_, _, bare) = &variants[0];
        assert!(
            bare.objective.is_empty() && bare.constraints.is_empty() && bare.included.is_empty()
        );
        let (_, _, rules) = &variants[1];
        assert_eq!(rules.constraints, context.constraints);
        assert!(rules.included.is_empty());
        assert_eq!(variants[2].2.included.len(), 1);
    }
}
