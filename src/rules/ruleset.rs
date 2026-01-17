use crate::rules::engine::RuleSet;
use crate::rules::rules::{BreakingChangeRule, CommitQualityRule, ConfigChangeRule, TestsRule};

/// Create the default V1 ruleset
pub fn create_default_ruleset() -> RuleSet {
    RuleSet::new()
        .add_rule(Box::new(TestsRule))
        .add_rule(Box::new(BreakingChangeRule))
        .add_rule(Box::new(ConfigChangeRule))
        .add_rule(Box::new(CommitQualityRule))
}
