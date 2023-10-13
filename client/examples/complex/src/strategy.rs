use crate::condition::Condition;
use tokyo::{
    analyzer::Analyzer,
    behavior::{Behavior, Noop},
};

#[derive(Debug)]
pub struct Strategy {
    tree: StrategyNode,
}

impl Strategy {
    pub fn new(branches: Vec<(Box<dyn Condition>, Box<StrategyNode>)>) -> Self {
        Self { tree: StrategyNode::Branch(branches) }
    }

    pub fn next_behavior(&mut self, analyzer: &Analyzer) -> Option<PrioritizedBehavior> {
        self.tree.next_behavior(analyzer)
    }
}

#[derive(Debug)]
pub enum StrategyNode {
    Branch(Vec<(Box<dyn Condition>, Box<StrategyNode>)>),
    Leaf(PrioritizedBehavior),
}

impl StrategyNode {
    pub fn next_behavior(&mut self, analyzer: &Analyzer) -> Option<PrioritizedBehavior> {
        match self {
            StrategyNode::Branch(nodes) => {
                for (condition, node) in nodes.iter_mut() {
                    if condition.evaluate(analyzer) {
                        return node.next_behavior(analyzer);
                    }
                }
                None
            },
            StrategyNode::Leaf(leaf) => Some(leaf.clone()),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Priority {
    Empty = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

// TODO: Replace with a pair.
#[derive(Clone, Debug)]
pub struct PrioritizedBehavior {
    pub priority: Priority,
    pub behavior: Box<dyn Behavior>,
}

impl PrioritizedBehavior {
    pub fn new() -> Self {
        Self { priority: Priority::Empty, behavior: Box::new(Noop {}) }
    }

    pub fn with_low<T: Behavior>(behavior: T) -> Self {
        Self { priority: Priority::Low, behavior: behavior.box_clone() }
    }

    pub fn with_medium<T: Behavior>(behavior: T) -> Self {
        Self { priority: Priority::Medium, behavior: behavior.box_clone() }
    }

    pub fn with_high<T: Behavior>(behavior: T) -> Self {
        Self { priority: Priority::High, behavior: behavior.box_clone() }
    }
}
