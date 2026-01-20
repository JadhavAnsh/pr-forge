use crate::pr::model::{BranchAnalysis, RuleFinding};

/// Core rule trait - all rules implement this
pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    #[allow(dead_code)]
    fn description(&self) -> &'static str;

    /// Evaluate the rule against branch analysis
    /// Returns None if rule doesn't apply, Some(finding) if it does
    fn evaluate(&self, analysis: &BranchAnalysis) -> Option<RuleFinding>;
}

/// Container for all rules
pub struct RuleSet {
    pub rules: Vec<Box<dyn Rule>>,
}

impl RuleSet {
    /// Create a new empty ruleset
    pub fn new() -> Self {
        RuleSet { rules: Vec::new() }
    }

    /// Add a rule to the set
    pub fn add_rule(mut self, rule: Box<dyn Rule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Evaluate all rules against analysis
    pub fn evaluate(&self, analysis: &BranchAnalysis) -> Vec<RuleFinding> {
        self.rules
            .iter()
            .filter_map(|rule| rule.evaluate(analysis))
            .collect()
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}
