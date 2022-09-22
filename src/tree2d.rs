use std::error::Error;
use std::rc::Rc;

use crate::bounding_box::BoundingBox;

pub struct DataSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct PlacementError {}

impl Error for PlacementError {
    fn description(&self) -> &str {
        "error placing data"
    }
}

impl std::fmt::Display for PlacementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", 1)
    }
}

struct Node<T> {
    right: Tree2d<T>,
    down: Tree2d<T>,
    data: Rc<T>,
    bb: BoundingBox,
}
pub struct Tree2d<T> {
    bb: BoundingBox,
    node: Option<Box<Node<T>>>,
}

impl<T> Tree2d<T> {
    pub fn new() -> Self {
        Self {
            bb: BoundingBox {
                x: 0,
                y: 0,
                width: u32::MAX,
                height: u32::MAX,
            },
            node: None,
        }
    }

    pub fn insert(&mut self, width: u32, height: u32, data: T) -> Result<(), Box<dyn Error>> {
        let total_bb = self.get_total_bounding_box();
        let opleaf = self.get_smallest_leaf_for_data(total_bb, width, height);
        match opleaf {
            None => Err(Box::new(PlacementError {})),
            Some((leaf, _)) => {
                *leaf = Self::partition(Rc::new(data), leaf.bb, width, height);
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

    pub fn flatten(&self) -> Vec<(Rc<T>, BoundingBox)> {
        let mut output: Vec<(Rc<T>, BoundingBox)> = vec![];

        self.flatten_aux(&mut output);

        output
    }

    pub fn get_total_bounding_box(&self) -> BoundingBox {
        self.get_total_bounding_box_aux(BoundingBox {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        })
    }

    fn partition(data: Rc<T>, bb: BoundingBox, width: u32, height: u32) -> Self {
        let width_remainder = bb.width - width;
        let height_remainder = bb.height - height;

        let (bb_right, bb_down) = if height_remainder > width_remainder {
            // ---------------
            // |  data  |    |
            // ---------------
            // |             |
            // |             |
            // ---------------
            (
                BoundingBox {
                    x: bb.x + width,
                    y: bb.y,
                    width: width_remainder,
                    height,
                },
                BoundingBox {
                    x: bb.x,
                    y: bb.y + height,
                    width: bb.width,
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
                    x: bb.x + width,
                    y: bb.y,
                    width: width_remainder,
                    height: bb.height,
                },
                BoundingBox {
                    x: bb.x,
                    y: bb.y + height,
                    width,
                    height: height_remainder,
                },
            )
        };

        Tree2d {
            bb,
            node: Some(Box::new(Node {
                right: Tree2d {
                    bb: bb_right,
                    node: None,
                },
                down: Tree2d {
                    bb: bb_down,
                    node: None,
                },
                data,
                bb: BoundingBox {
                    x: bb.x,
                    y: bb.y,
                    width,
                    height,
                },
            })),
        }
    }

    fn get_smallest_leaf_for_data(
        &mut self,
        current_bb: BoundingBox,
        width: u32,
        height: u32,
    ) -> Option<(&mut Self, BoundingBox)> {
        if self.bb.can_contain(width, height) {
            let is_leaf = match self.node {
                None => true,
                Some(..) => false,
            };

            if is_leaf {
                let bb = BoundingBox {
                    x: self.bb.x,
                    y: self.bb.y,
                    width,
                    height,
                } + current_bb;
                Some((self, bb))
            } else {
                match &mut self.node {
                    None => unreachable!(),
                    Some(node) => {
                        let optree_down = node
                            .down
                            .get_smallest_leaf_for_data(current_bb, width, height);
                        let optree_right = node
                            .right
                            .get_smallest_leaf_for_data(current_bb, width, height);
                        match (optree_down, optree_right) {
                            (Some((tree_down, bb_down)), Some((tree_right, bb_right))) => {
                                if bb_down < bb_right {
                                    Some((tree_down, bb_down))
                                } else {
                                    Some((tree_right, bb_right))
                                }
                            }
                            (Some((tree_down, bb_down)), None) => Some((tree_down, bb_down)),
                            (None, Some((tree_right, bb_right))) => Some((tree_right, bb_right)),
                            (None, None) => None,
                        }
                    }
                }
            }
        } else {
            None
        }
    }

    fn flatten_aux<'a>(
        &self,
        output: &'a mut Vec<(Rc<T>, BoundingBox)>,
    ) -> &'a mut Vec<(Rc<T>, BoundingBox)> {
        match &self.node {
            None => output,
            Some(node) => {
                output.push((Rc::clone(&node.data), self.bb));
                node.right.flatten_aux(output);
                node.down.flatten_aux(output);
                output
            }
        }
    }

    pub fn get_total_bounding_box_aux(&self, bb: BoundingBox) -> BoundingBox {
        match &self.node {
            None => BoundingBox {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            Some(node) => {
                node.bb
                    + node.right.get_total_bounding_box_aux(bb)
                    + node.down.get_total_bounding_box_aux(bb)
            }
        }
    }
}

#[cfg(test)]
mod tree_2d_tests {
    use super::*;

    #[test]
    fn new_tree() -> Result<(), Box<dyn Error>> {
        let data = 1u32;

        let mut tree = Tree2d::<u32>::new();

        tree.insert(2u32, 2u32, data)?;

        let expected_bb_right = BoundingBox {
            x: 2,
            y: 0,
            width: u32::MAX - 2,
            height: u32::MAX,
        };

        let expected_bb_down = BoundingBox {
            x: 0,
            y: 2,
            width: 2,
            height: u32::MAX - 2,
        };

        match tree.node {
            None => assert!(false, "root should be a node"),
            Some(node) => {
                assert_eq!(expected_bb_right, node.right.bb);
                assert_eq!(expected_bb_down, node.down.bb);
            }
        }
        Ok(())
    }
}
