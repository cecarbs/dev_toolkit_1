use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the collections tree (can be a folder or template)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    /// Name of this node
    pub name: String,

    /// Full path from root (e.g., "Customer/Add")
    pub path: String,

    /// Type of node
    pub node_type: NodeType,

    /// Child nodes (for folders)
    pub children: Vec<TreeNode>,

    /// Whether this node is expanded in the UI
    pub is_expanded: bool,

    /// Whether this node is currently selected
    pub is_selected: bool,

    /// Depth in the tree (0 = root level)
    pub depth: usize,
}

/// Type of tree node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// A folder that can contain other folders or templates
    Folder,
    /// A template file that can be loaded
    Template,
}

impl TreeNode {
    /// Create a new folder node
    pub fn new_folder(name: String, path: String, depth: usize) -> Self {
        Self {
            name,
            path,
            node_type: NodeType::Folder,
            children: Vec::new(),
            is_expanded: depth == 0, // Expand root level by default
            is_selected: false,
            depth,
        }
    }

    /// Create a new template node
    pub fn new_template(name: String, path: String, depth: usize) -> Self {
        Self {
            name,
            path: path.clone(),
            node_type: NodeType::Template,
            children: Vec::new(),
            is_expanded: false, // Templates can't be expanded
            is_selected: false,
            depth,
        }
    }

    /// Toggle the expanded state of this node
    pub fn toggle_expanded(&mut self) {
        if self.node_type == NodeType::Folder {
            self.is_expanded = !self.is_expanded;
        }
    }

    /// Set the selected state
    pub fn set_selected(&mut self, selected: bool) {
        self.is_selected = selected;
    }

    /// Add a child node
    pub fn add_child(&mut self, child: TreeNode) {
        if self.node_type == NodeType::Folder {
            self.children.push(child);
        }
    }

    /// Sort children alphabetically (folders first, then templates)
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| {
            // Folders come before templates
            match (&a.node_type, &b.node_type) {
                (NodeType::Folder, NodeType::Template) => std::cmp::Ordering::Less,
                (NodeType::Template, NodeType::Folder) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name), // Same type, sort by name
            }
        });

        // Recursively sort children
        for child in &mut self.children {
            child.sort_children();
        }
    }

    /// Find a node by path
    pub fn find_by_path(&self, target_path: &str) -> Option<&TreeNode> {
        if self.path == target_path {
            return Some(self);
        }

        for child in &self.children {
            if let Some(found) = child.find_by_path(target_path) {
                return Some(found);
            }
        }

        None
    }

    /// Find a node by path (mutable)
    pub fn find_by_path_mut(&mut self, target_path: &str) -> Option<&mut TreeNode> {
        if self.path == target_path {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(found) = child.find_by_path_mut(target_path) {
                return Some(found);
            }
        }

        None
    }

    /// Get all visible nodes (taking into account expanded state)
    pub fn get_visible_nodes(&self) -> Vec<&TreeNode> {
        let mut visible = Vec::new();
        self.collect_visible_nodes(&mut visible);
        visible
    }

    /// Recursively collect visible nodes
    fn collect_visible_nodes<'a>(&'a self, visible: &mut Vec<&'a TreeNode>) {
        visible.push(self);

        if self.is_expanded {
            for child in &self.children {
                child.collect_visible_nodes(visible);
            }
        }
    }

    /// Get display icon for this node
    pub fn get_icon(&self) -> &'static str {
        match self.node_type {
            NodeType::Folder => {
                if self.is_expanded {
                    "📂" // Open folder
                } else {
                    "📁" // Closed folder
                }
            }
            NodeType::Template => "📄", // File
        }
    }

    /// Get indentation string for display
    pub fn get_indent(&self) -> String {
        "  ".repeat(self.depth)
    }

    /// Check if this node can be expanded
    pub fn can_expand(&self) -> bool {
        self.node_type == NodeType::Folder && !self.children.is_empty()
    }
}

/// Tree state manager for the collections panel
#[derive(Debug, Clone)]
pub struct TreeState {
    /// Root nodes of the tree
    pub roots: Vec<TreeNode>,

    /// Currently selected node path
    pub selected_path: Option<String>,

    /// Currently focused node index (for keyboard navigation)
    pub focused_index: usize,
}

impl TreeState {
    /// Create a new empty tree state
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            selected_path: None,
            focused_index: 0,
        }
    }

    /// Build tree from folder and template lists
    pub fn build_from_storage(
        folders: Vec<String>,
        templates_by_folder: HashMap<String, Vec<String>>,
    ) -> Self {
        let mut state = Self::new();

        // Build the tree structure
        for folder_path in folders {
            state.add_folder_path(&folder_path);
        }

        // Add templates to folders
        for (folder_path, templates) in templates_by_folder {
            for template_name in templates {
                state.add_template(&folder_path, &template_name);
            }
        }

        // Sort all nodes
        for root in &mut state.roots {
            root.sort_children();
        }

        state
    }

    /// Add a folder path to the tree
    fn add_folder_path(&mut self, path: &str) {
        let parts: Vec<&str> = path.split('/').collect();
        let mut current_path = String::new();
        let mut current_roots = &mut self.roots;

        for (depth, part) in parts.iter().enumerate() {
            if depth > 0 {
                current_path.push('/');
            }
            current_path.push_str(part);

            // Find or create this node
            let existing_index = current_roots.iter().position(|node| node.name == *part);

            if existing_index.is_none() {
                // Create new folder node
                let new_node = TreeNode::new_folder(part.to_string(), current_path.clone(), depth);
                current_roots.push(new_node);
            }

            // Move to the children of this node
            let index = current_roots
                .iter()
                .position(|node| node.name == *part)
                .unwrap();
            current_roots = &mut current_roots[index].children;
        }
    }

    /// Add a template to a folder
    fn add_template(&mut self, folder_path: &str, template_name: &str) {
        if let Some(folder_node) = self.find_folder_mut(folder_path) {
            let template_path = if folder_path.is_empty() {
                template_name.to_string()
            } else {
                format!("{}/{}", folder_path, template_name)
            };

            let template_node = TreeNode::new_template(
                template_name.to_string(),
                template_path,
                folder_node.depth + 1,
            );

            folder_node.add_child(template_node);
        }
    }

    /// Find a folder node by path
    fn find_folder_mut(&mut self, path: &str) -> Option<&mut TreeNode> {
        for root in &mut self.roots {
            if let Some(node) = root.find_by_path_mut(path) {
                return Some(node);
            }
        }
        None
    }

    /// Get all visible nodes for display
    pub fn get_visible_nodes(&self) -> Vec<&TreeNode> {
        let mut visible = Vec::new();
        for root in &self.roots {
            root.collect_visible_nodes(&mut visible);
        }
        visible
    }

    /// Toggle expansion of a node
    pub fn toggle_expansion(&mut self, path: &str) {
        for root in &mut self.roots {
            if let Some(node) = root.find_by_path_mut(path) {
                node.toggle_expanded();
                break;
            }
        }
    }

    /// Select a node
    pub fn select_node(&mut self, path: &str) {
        // Deselect previous
        if let Some(old_path) = &self.selected_path {
            for root in &mut self.roots {
                if let Some(node) = root.find_by_path_mut(old_path) {
                    node.set_selected(false);
                    break;
                }
            }
        }

        // Select new
        for root in &mut self.roots {
            if let Some(node) = root.find_by_path_mut(path) {
                node.set_selected(true);
                self.selected_path = Some(path.to_string());
                break;
            }
        }
    }

    /// Move focus up/down for keyboard navigation
    pub fn move_focus(&mut self, direction: FocusDirection) {
        let visible_nodes = self.get_visible_nodes();
        if visible_nodes.is_empty() {
            return;
        }

        match direction {
            FocusDirection::Up => {
                if self.focused_index > 0 {
                    self.focused_index -= 1;
                }
            }
            FocusDirection::Down => {
                if self.focused_index < visible_nodes.len() - 1 {
                    self.focused_index += 1;
                }
            }
        }
    }

    /// Get the currently focused node
    pub fn get_focused_node(&self) -> Option<&TreeNode> {
        let visible_nodes = self.get_visible_nodes();
        visible_nodes.get(self.focused_index).copied()
    }
}

/// Direction for focus movement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusDirection {
    Up,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_creation() {
        let folder = TreeNode::new_folder("Test".to_string(), "Test".to_string(), 0);
        assert_eq!(folder.node_type, NodeType::Folder);
        assert_eq!(folder.name, "Test");
        assert!(folder.is_expanded); // Root level expanded by default

        let template = TreeNode::new_template("File".to_string(), "Test/File".to_string(), 1);
        assert_eq!(template.node_type, NodeType::Template);
        assert!(!template.is_expanded); // Templates can't be expanded
    }

    #[test]
    fn test_tree_building() {
        let folders = vec!["Customer".to_string(), "Customer/Add".to_string()];
        let mut templates = HashMap::new();
        templates.insert("Customer/Add".to_string(), vec!["Email".to_string()]);

        let state = TreeState::build_from_storage(folders, templates);

        assert_eq!(state.roots.len(), 1);
        assert_eq!(state.roots[0].name, "Customer");
        assert_eq!(state.roots[0].children.len(), 1);
        assert_eq!(state.roots[0].children[0].name, "Add");
        assert_eq!(state.roots[0].children[0].children.len(), 1);
        assert_eq!(state.roots[0].children[0].children[0].name, "Email");
    }
}
