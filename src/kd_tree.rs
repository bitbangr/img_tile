use ego_tree::{NodeMut, NodeRef, Tree};

static mut DIST_CALLS: u64 = 0;

pub fn dist_sq(a: &[u8], b: &[u8]) -> u32 {
    unsafe {
        DIST_CALLS += 1;
    }
    let mut sum: u32 = 0;
    for (aa, bb) in a.iter().zip(b.iter()) {
        let val: i32 = i32::from(*aa) - i32::from(*bb);
        sum += (val * val) as u32;
    }
    sum
}

pub fn construct_kd_tree(v: &mut [Vec<u8>], dimension: usize) -> Tree<Vec<u8>> {
    v.sort_unstable_by_key(|k| k[0]);
    let middle: usize = v.len() / 2;
    let mut tree = Tree::with_capacity(v[middle].clone(), v.len());
    {
        let mut root = tree.root_mut();
        construct_kd_tree_recursive(&mut v[..middle], &mut root, 1, dimension);
        construct_kd_tree_recursive(&mut v[middle + 1..], &mut root, 1, dimension);
    }
    tree
}

pub fn construct_kd_tree_recursive(
    v: &mut [Vec<u8>],
    node: &mut NodeMut<Vec<u8>>,
    current_dim: usize,
    max_dimension: usize,
) {
    if v.len() == 2 {
        v.sort_unstable_by_key(|k| k[current_dim]);
        let mut child = node.append(v[1].clone());
        child.append(v[0].clone());
        return;
    } else if v.len() == 1 {
        node.append(v[0].clone());
        return;
    }
    v.sort_unstable_by_key(|k| k[current_dim]);
    let middle: usize = v.len() / 2;
    let mut child = node.append(v[middle].clone());
    let mut next_dim = current_dim + 1;
    if next_dim == max_dimension {
        next_dim = 0;
    }
    construct_kd_tree_recursive(&mut v[..middle], &mut child, next_dim, max_dimension);
    construct_kd_tree_recursive(&mut v[middle + 1..], &mut child, next_dim, max_dimension);
}

pub fn query_nearest_neighbor<'a>(
    q: &[u8],
    kd_tree: &'a Tree<Vec<u8>>,
    max_dimension: usize,
    root_node_ref: NodeRef<'a, Vec<u8>>,
) -> NodeRef<'a, Vec<u8>> {
    let mut current_node = root_node_ref;
    let mut current_dim = 0;
    loop {
        // We have reached a leaf node.
        if !current_node.has_children() {
            break;
        }
        let left_child = current_node.first_child().unwrap();
        if !left_child.has_siblings() {
            current_node = left_child;
            current_dim += 1;
            if current_dim == max_dimension {
                current_dim = 0;
            }
        } else {
            if q[current_dim] < current_node.value()[current_dim] {
                current_node = left_child;
            } else {
                current_node = current_node.last_child().unwrap();
            }
            current_dim += 1;
            if current_dim == max_dimension {
                current_dim = 0;
            }
        }
    }

    let mut best_guess_node = current_node;
    let mut best_guess_dist = dist_sq(q, current_node.value());

    loop {
        // We have reached the root.
        if current_node == root_node_ref {
            break;
        }
        let parent_node = current_node.parent().unwrap();
        let parent_val = parent_node.value();
        let parent_dist = dist_sq(q, parent_val);
        if parent_dist < best_guess_dist {
            best_guess_dist = parent_dist;
            best_guess_node = parent_node;
        }

        let plane_dist: u32 = (i32::from(parent_val[current_dim])
            - i32::from(best_guess_node.value()[current_dim]))
            .abs() as u32;
        if plane_dist * plane_dist < best_guess_dist {
            let mut node_id_option = current_node.next_sibling();
            if node_id_option.is_none() {
                node_id_option = current_node.prev_sibling();
            }
            if !node_id_option.is_none() {
                let second_best_guess_node =
                    query_nearest_neighbor(q, kd_tree, max_dimension, node_id_option.unwrap());
                let second_best_guess_dist = dist_sq(q, second_best_guess_node.value());
                if second_best_guess_dist < best_guess_dist {
                    best_guess_dist = second_best_guess_dist;
                    best_guess_node = second_best_guess_node;
                }
            }
        }

        if current_dim == 0 {
            current_dim = max_dimension - 1;
        } else {
            current_dim -= 1;
        }

        current_node = parent_node;
    }

    best_guess_node
}
