#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

pub mod adoption;
pub mod catalog;
pub mod error;
pub mod overlap;
pub mod prompts;
pub mod rules;
pub mod validation;

pub use adoption::{load_adoption_paths, RustUseAdoptionPath, RustUseAdoptionPaths};
pub use catalog::{
    load_catalog, CatalogSearchResult, ChildrenResult, CrateLookupResult, NameCollisionKind,
    NameCollisionReport, RustUseCatalog, RustUseCrate, RustUseSet, SearchMatch,
};
pub use error::CoreError;
pub use overlap::{find_overlap, OverlapFinding, OverlapReport};
pub use prompts::{
    generate_copilot_prompt, list_prompt_templates, render_prompt, GeneratedCopilotPrompt,
    PromptArgument, PromptRenderRequest, PromptRenderResult, PromptTemplate,
};
pub use rules::{load_rules, RustUseRule, RustUseRuleSet};
pub use validation::{validate_set_plan, RuleCheck, ValidationFinding, ValidationReport};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_json_loads_successfully() -> Result<(), CoreError> {
        let catalog = load_catalog()?;
        assert!(catalog.set("use-math").is_some());
        assert!(catalog.set("use-geometry").is_some());
        Ok(())
    }

    #[test]
    fn rules_json_loads_successfully() -> Result<(), CoreError> {
        let rules = load_rules()?;
        assert!(rules
            .rules
            .iter()
            .any(|rule| rule.id == "rust-edition-2024"));
        Ok(())
    }

    #[test]
    fn adoption_paths_json_loads_successfully() -> Result<(), CoreError> {
        let paths = load_adoption_paths()?;
        assert!(paths.paths.iter().any(|path| path.name == "copy_and_own"));
        Ok(())
    }

    #[test]
    fn catalog_search_returns_expected_matches() -> Result<(), CoreError> {
        let catalog = load_catalog()?;
        let result = catalog.search("geometry", 10);
        assert!(result.total > 0);
        assert!(result
            .matches
            .iter()
            .any(|item| item.name == "use-geometry"));
        Ok(())
    }

    #[test]
    fn get_set_works_for_known_set() -> Result<(), CoreError> {
        let catalog = load_catalog()?;
        let set = catalog.set("use-math");
        assert!(set.is_some());
        assert_eq!(set.map(|item| item.children.len()), Some(28));
        Ok(())
    }

    #[test]
    fn get_set_handles_unknown_set() -> Result<(), CoreError> {
        let catalog = load_catalog()?;
        assert!(catalog.set("use-unknown").is_none());
        Ok(())
    }

    #[test]
    fn name_collision_detects_existing_names() -> Result<(), CoreError> {
        let catalog = load_catalog()?;
        let report = catalog.check_name_collision("use_math", NameCollisionKind::Any);
        assert!(report.collision);
        assert!(report.exact_matches.iter().any(|item| item == "use-math"));
        Ok(())
    }

    #[test]
    fn validate_set_plan_produces_findings() -> Result<(), CoreError> {
        let catalog = load_catalog()?;
        let rules = load_rules()?;
        let report = validate_set_plan(
            &catalog,
            &rules,
            "use-math",
            "Math utilities",
            &[String::from("use-number"), String::from("use-number")],
        );
        assert!(!report.valid);
        assert!(!report.findings.is_empty());
        Ok(())
    }

    #[test]
    fn prompt_rendering_works() -> Result<(), CoreError> {
        let request = PromptRenderRequest {
            prompt_name: "rustuse_brainstorm_set".to_owned(),
            set_name: Some("use-example".to_owned()),
            description: Some("Example primitives".to_owned()),
            minimum_children: Some(10),
            ..PromptRenderRequest::default()
        };
        let result = render_prompt(&request)?;
        assert!(result.text.contains("use-example"));
        assert!(result.text.contains("Example primitives"));
        Ok(())
    }
}
