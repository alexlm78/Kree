use crate::tree::TreeNode;

pub fn render_tree(root: &TreeNode) {
    println!("{}", root.name);
    let child_count = root.children.len();
    for (i, child) in root.children.iter().enumerate() {
        let is_last = i == child_count - 1;
        render_node(child, 0, is_last, if is_last { 1 } else { 0 });
    }
}

fn render_node(node: &TreeNode, depth: u32, is_last: bool, mask: u64) {
    for i in 0..depth {
        if ((mask >> i) & 1) == 0 {
            print!("│    ");
        } else {
            print!("     ");
        }
    }

    if is_last {
        print!("└── ");
    } else {
        print!("├── ");
    }

    println!("{}", node.name);

    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i == child_count - 1;
        let new_mask = if child_is_last {
            mask | (1u64 << (depth + 1))
        } else {
            mask
        };
        render_node(child, depth + 1, child_is_last, new_mask);
    }
}
