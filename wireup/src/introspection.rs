#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDescriptor {
    pub name: String,
    pub lifecycle: String,
}

impl NodeDescriptor {
    pub fn new(name: impl Into<String>, lifecycle: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            lifecycle: lifecycle.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GraphDescriptor {
    pub nodes: Vec<NodeDescriptor>,
    pub edges: Vec<(String, String)>,
}

impl GraphDescriptor {
    pub fn new(nodes: Vec<NodeDescriptor>, edges: Vec<(String, String)>) -> Self {
        Self { nodes, edges }
    }
}
