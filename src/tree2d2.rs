use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl BoundingBox {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn perimeter(&self) -> u32 {
        self.width * 2 + self.height * 2
    }

    fn can_contain(&self, width: u32, height: u32) -> bool {
        width <= self.width && height <= self.height
    }

    fn same_shape(&self, width: u32, height: u32) -> bool {
        width == self.width && height == self.height
    }
}

impl std::ops::Add<&BoundingBox> for &BoundingBox {
    type Output = BoundingBox;
    fn add(self, v: &BoundingBox) -> BoundingBox {
        BoundingBox {
            x: self.x.min(v.x),
            y: self.y.min(v.y),
            width: (self.x + self.width).max(v.x + v.width) - self.x.min(v.x),
            height: (self.y + self.height).max(v.y + v.height) - self.y.min(v.y),
        }
    }
}

impl Ord for BoundingBox {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.area().cmp(&other.area()) {
            Ordering::Equal => self.perimeter().cmp(&other.perimeter()),
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
        }
    }
}

impl PartialOrd for BoundingBox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Node<T> {
    right: Box<Tree2d<T>>,
    down: Box<Tree2d<T>>,
    data: Rc<T>,
    data_bb: BoundingBox,
}
pub struct Tree2d<T> {
    bb: BoundingBox,
    node: Option<Node<T>>,
}

impl<T> Tree2d<T> {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            bb: BoundingBox {
                x: 0,
                y: 0,
                width,
                height,
            },
            node: None,
        }
    }

    pub fn insert(&mut self, width: u32, height: u32, data: T) -> bool {
        let total_bb = self.get_total_bounding_box(BoundingBox {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        });
        let opleaf = self.get_smallest_leaf_for_data(total_bb, width, height);
        match opleaf {
            None => false,
            Some((leaf, _)) => {
                *leaf = Self::partition(Rc::new(data), leaf.bb, width, height);
                true
            }
        }
    }

    pub fn flatten(&self) -> Vec<(Rc<T>, BoundingBox)> {
        let mut output: Vec<(Rc<T>, BoundingBox)> = vec![];

        self.flatten_aux(&mut output);

        output
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
            node: Some(Node {
                right: Box::new(Tree2d {
                    bb: bb_right,
                    node: None,
                }),
                down: Box::new(Tree2d {
                    bb: bb_down,
                    node: None,
                }),
                data,
                data_bb: BoundingBox {
                    x: bb.x,
                    y: bb.y,
                    width,
                    height,
                },
            }),
        }
    }

    fn get_smallest_leaf_for_data(
        &mut self,
        current_bb: BoundingBox,
        width: u32,
        height: u32,
    ) -> Option<(&mut Self, BoundingBox)> {
        let is_leaf = match self.node {
            None => true,
            Some(..) => false,
        };

        if is_leaf {
            if self.bb.can_contain(width, height) {
                let bb = &BoundingBox {
                    x: self.bb.x,
                    y: self.bb.y,
                    width,
                    height,
                } + &current_bb;
                Some((self, bb))
            } else {
                None
            }
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

    fn get_total_bounding_box(&self, bb: BoundingBox) -> BoundingBox {
        match &self.node {
            None => BoundingBox {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            Some(node) => {
                &(&(node.data_bb) + &(node.right.get_total_bounding_box(bb)))
                    + &(node.down.get_total_bounding_box(bb))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb_horizontal_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 1,
            y: 0,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_horizontal_overlap_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 1,
            y: 0,
            width: 2,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 3,
            height: 1,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_vertival_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 1,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 2,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_vertival_overlap_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 2,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 1,
            width: 1,
            height: 2,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 3,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_diagonal_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 1,
            y: 1,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_diagonal_overlap_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };
        let bb2 = BoundingBox {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 3,
            height: 3,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_diagonal_disjoint_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 2,
            y: 2,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 3,
            height: 3,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_inside_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_cmp_area() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };
        assert_eq!(true, bb1 < bb2);
        assert_eq!(false, bb2 < bb1);
    }

    #[test]
    fn bb_cmp_perimeter() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 0,
            width: 4,
            height: 1,
        };
        assert_eq!(bb1.area(), bb2.area());
        assert_eq!(true, bb1 < bb2);
        assert_eq!(false, bb2 < bb1);
    }
}
