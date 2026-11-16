use derive_new::new;

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct NodeDescriptor {
    #[new(into)]
    pub name: String,
    #[new(into)]
    pub lifecycle: String,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct EdgeDescriptor {
    #[new(into)]
    pub from: String,
    #[new(into)]
    pub to: String,
}

#[derive(new, Debug, Clone, PartialEq, Eq, Default)]
pub struct GraphDescriptor {
    pub nodes: Vec<NodeDescriptor>,
    pub edges: Vec<EdgeDescriptor>,
}
