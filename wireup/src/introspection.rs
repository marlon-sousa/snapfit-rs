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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeDescriptor {
    pub from: String,
    pub to: String,
}

impl EdgeDescriptor {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GraphDescriptor {
    pub nodes: Vec<NodeDescriptor>,
    pub edges: Vec<EdgeDescriptor>,
}

impl GraphDescriptor {
    pub fn new(nodes: Vec<NodeDescriptor>, edges: Vec<EdgeDescriptor>) -> Self {
        Self { nodes, edges }
    }
}
