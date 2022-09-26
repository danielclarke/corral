use std::error::Error;

use crate::bounding_box::BoundingBox;

#[derive(Clone, Debug)]
pub struct InsertionError {
    msg: String,
}

impl Error for InsertionError {
    fn description(&self) -> &str {
        "error inserting data, not enough space"
    }
}

impl std::fmt::Display for InsertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.msg)
    }
}

type Handle = usize;

struct Arena<T> {
    data: Vec<T>,
}

impl<T> Arena<T> {
    fn get(&self, handle: Handle) -> Option<&T> {
        if self.data.len() < handle {
            None
        } else {
            Some(&self.data[handle])
        }
    }

    fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        if self.data.len() < handle {
            None
        } else {
            Some(&mut self.data[handle])
        }
    }

    fn store(&mut self, data: T) -> Handle {
        self.data.push(data);
        self.data.len() - 1
    }
}

impl<T> Arena<T> {
    fn new() -> Self {
        Arena { data: Vec::new() }
    }
}

pub struct DataSize {
    pub width: u32,
    pub height: u32,
}

pub struct Tree2d<T> {
    root: Handle,
    nodes: Arena<Node<T>>,
}

struct Node<T> {
    bb: BoundingBox,
    parent: Option<Handle>,
    link: Option<Link<T>>,
}

struct Link<T> {
    data: T,
    data_bb: BoundingBox,
    down: Handle,
    right: Handle,
}

impl<T> Node<T> {
    fn leaf(bb: BoundingBox, parent: Option<Handle>) -> Self {
        Self {
            bb,
            parent,
            link: None,
        }
    }

    fn is_leaf(&self) -> bool {
        match self.link {
            None => true,
            Some(..) => false,
        }
    }
}

impl<T> Tree2d<T> {
    pub fn new() -> Self {
        let node: Node<T> = Node::leaf(
            BoundingBox {
                x: 0,
                y: 0,
                width: u32::MAX,
                height: u32::MAX,
            },
            None,
        );

        let mut nodes = Arena::new();
        let root = nodes.store(node);

        Tree2d { root, nodes }
    }

    pub fn get_total_bounding_box(&self) -> BoundingBox {
        let mut result = BoundingBox {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        for node in self.nodes.data.iter() {
            if let Some(link) = &node.link {
                result = result + link.data_bb
            }
        }

        result
    }

    pub fn insert(&mut self, width: u32, height: u32, data: T) -> Result<(), Box<dyn Error>> {
        let handle = self.get_smallest_leaf_handle_for_data(width, height);
        match handle {
            None => Err(Box::new(InsertionError {
                msg: "Error inserting data, no partition large enough".to_owned(),
            })),
            Some(handle) => {
                self.partition(handle, data, width, height);
                Ok(())
            }
        }
    }

    pub fn insert_all(&mut self, data: Vec<(DataSize, T)>) -> Result<(), Box<dyn Error>> {
        for (DataSize { width, height }, data) in data {
            self.insert(width, height, data)?;
        }
        Ok(())
    }

    pub fn flatten(&self) -> Vec<(&T, BoundingBox)> {
        let mut result = vec![];
        for node in self.nodes.data.iter() {
            match &node.link {
                None => (),
                Some(link) => result.push((&link.data, link.data_bb)),
            };
        }
        result
    }

    fn leaves(&self) -> Vec<Handle> {
        let mut result = vec![];
        for (i, node) in self.nodes.data.iter().enumerate() {
            if node.is_leaf() {
                result.push(i);
            }
        }
        result
    }

    fn get_smallest_leaf_handle_for_data(&mut self, width: u32, height: u32) -> Option<Handle> {
        let mut leaves = vec![];
        let total_bb = self.get_total_bounding_box();
        for handle in self.leaves() {
            if let Some(node) = self.nodes.get(handle) {
                if node.bb.can_contain(width, height) {
                    let bb = total_bb
                        + BoundingBox {
                            x: node.bb.x,
                            y: node.bb.y,
                            width,
                            height,
                        };

                    leaves.push((bb, handle));
                }
            }
        }
        if leaves.is_empty() {
            None
        } else {
            leaves.sort_by(|a, b| a.0.cmp(&b.0));
            Some(leaves[0].1)
        }
    }

    fn partition(&mut self, handle: Handle, data: T, width: u32, height: u32) {
        let (right, down) = match self.nodes.get_mut(handle) {
            None => (None, None),
            Some(node) => {
                let width_remainder = node.bb.width - width;
                let height_remainder = node.bb.height - height;

                let (bb_right, bb_down) = if height_remainder > width_remainder {
                    // ---------------
                    // |  data  |    |
                    // ---------------
                    // |             |
                    // |             |
                    // ---------------
                    (
                        BoundingBox {
                            x: node.bb.x + width,
                            y: node.bb.y,
                            width: width_remainder,
                            height,
                        },
                        BoundingBox {
                            x: node.bb.x,
                            y: node.bb.y + height,
                            width: node.bb.width,
                            height: height_remainder,
                        },
                    )
                } else {
                    // ---------------
                    // |     |       |
                    // |data |       |
                    // |     |       |
                    // ------|       |
                    // |     |       |
                    // |     |       |
                    // ---------------
                    (
                        BoundingBox {
                            x: node.bb.x + width,
                            y: node.bb.y,
                            width: width_remainder,
                            height: node.bb.height,
                        },
                        BoundingBox {
                            x: node.bb.x,
                            y: node.bb.y + height,
                            width,
                            height: height_remainder,
                        },
                    )
                };
                (
                    Some(Node::leaf(bb_right, Some(handle))),
                    Some(Node::leaf(bb_down, Some(handle))),
                )
            }
        };

        if let (Some(right), Some(down)) = (right, down) {
            let right_handle = self.nodes.store(right);
            let down_handle = self.nodes.store(down);
            if let Some(node) = self.nodes.get_mut(handle) {
                let data_bb = BoundingBox {
                    x: node.bb.x,
                    y: node.bb.y,
                    width,
                    height,
                };
                *node = Node {
                    bb: node.bb,
                    parent: None,
                    link: Some(Link {
                        data,
                        data_bb,
                        down: down_handle,
                        right: right_handle,
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tree_2d_tests {
    use super::*;

    #[test]
    fn new_empty_tree() -> Result<(), Box<dyn Error>> {
        let tree = Tree2d::<u32>::new();
        let root_node = tree.nodes.get(tree.root);

        if let Some(node) = root_node {
            assert!(node.is_leaf(), "root in empty tree should be leaf");
        } else {
            assert!(false, "root should be Some");
        }

        assert_eq!(
            vec![0],
            tree.leaves(),
            "tree.leaves() should return handle to root for an empty tree"
        );

        let zero_bb = BoundingBox {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        assert_eq!(zero_bb, tree.get_total_bounding_box());

        Ok(())
    }

    #[test]
    fn partition() -> Result<(), Box<dyn Error>> {
        let mut tree = Tree2d::<u32>::new();
        tree.partition(tree.root, 1, 1, 1);

        assert_eq!(tree.nodes.data.len(), 3);

        Ok(())
    }

    #[test]
    fn get_total_bounding_box() -> Result<(), Box<dyn Error>> {
        let mut tree = Tree2d::<u32>::new();
        tree.partition(tree.root, 1, 1, 1);

        assert_eq!(tree.nodes.data.len(), 3);

        let single_bb = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        assert_eq!(single_bb, tree.get_total_bounding_box());

        Ok(())
    }

    #[test]
    fn leaves() -> Result<(), Box<dyn Error>> {
        let mut tree = Tree2d::<u32>::new();

        assert_eq!(
            vec![0],
            tree.leaves(),
            "tree.leaves() should return handle to root for an empty tree"
        );

        tree.partition(tree.root, 1, 1, 1);

        assert_eq!(
            vec![1, 2],
            tree.leaves(),
            "2nd and 3rd nodes should be leaves in single partition tree"
        );

        Ok(())
    }

    #[test]
    fn get_smallest_leaf_for_data() -> Result<(), Box<dyn Error>> {
        let mut tree = Tree2d::<u32>::new();
        let data = 1u32;

        let width = 2;
        let height = 1;

        assert_eq!(
            tree.get_smallest_leaf_handle_for_data(width, height),
            Some(0)
        );

        tree.partition(tree.root, data, width, height);

        assert_eq!(
            tree.get_smallest_leaf_handle_for_data(width, height),
            Some(2)
        );

        Ok(())
    }

    #[test]
    fn insert() -> Result<(), Box<dyn Error>> {
        let mut tree = Tree2d::<u32>::new();
        let data = 1u32;

        let width = 1;
        let height = 1;

        tree.insert(width, height, data)?;

        assert_eq!(tree.nodes.data.len(), 3);

        assert_eq!(1, tree.get_total_bounding_box().area());

        Ok(())
    }

    #[test]
    fn insert_all() -> Result<(), Box<dyn Error>> {
        let mut tree = Tree2d::<u32>::new();

        let data = 1u32;
        let width = 1;
        let height = 1;

        let data = vec![
            (DataSize { width, height }, data),
            (DataSize { width, height }, data),
            (DataSize { width, height }, data),
            (DataSize { width, height }, data),
        ];

        tree.insert_all(data)?;

        assert_eq!(tree.nodes.data.len(), 9);

        assert_eq!(4, tree.get_total_bounding_box().area());

        Ok(())
    }

    // #[test]
    // fn one_million_insertions() -> Result<(), Box<dyn Error>> {
    //     let mut tree = Tree2d::<u32>::new();
    //     for i in 0..1_000_000 {
    //         tree.insert(1, 1, i)?;
    //     }
    //     Ok(())
    // }
}
