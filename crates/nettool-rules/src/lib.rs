//! Packet predicate rules for the NetTool proxy.
//!
//! Rules are deliberately small, dependency-free and safe to parse from CLI
//! arguments or a future daemon request. They never rewrite traffic; they only
//! allow or drop a captured chunk.

mod rule;

pub use rule::{Action, Direction, Rule, RuleContext, RuleError, RuleSet};
