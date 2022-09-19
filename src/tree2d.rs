use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BoundingBox {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl BoundingBox {
    fn size(&self) -> u32 {
        self.width * self.height
    }

    fn can_contain(&self, width: u32, height: u32) -> bool {
        width < self.width && height < self.height
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

pub enum Tree2d<T> {
    Leaf {
        bb: BoundingBox,
    },
    Node {
        bb: BoundingBox,
        right: Box<Self>,
        down: Box<Self>,
        data: Rc<T>,
    },
}

impl<T> Tree2d<T> {
    fn new(width: u32, height: u32) -> Self {
        Self::Leaf {
            bb: BoundingBox {
                x: 0,
                y: 0,
                width,
                height,
            },
        }
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

        Tree2d::Node {
            bb,
            right: Box::new(Self::Leaf { bb: bb_right }),
            down: Box::new(Self::Leaf { bb: bb_down }),
            data,
        }
    }

    fn insert(&mut self, data: Rc<T>, width: u32, height: u32) -> bool {
        match self {
            Self::Leaf { bb } => {
                if bb.can_contain(width, height) {
                    *self = Self::partition(data, *bb, width, height);
                    true
                } else {
                    false
                }
            }
            Self::Node {
                bb, right, down, ..
            } => {
                if bb.can_contain(width, height) {
                    match (&**right, &**down) {
                        (Self::Leaf { .. }, Self::Leaf { .. }) => {
                            if right.insert(Rc::clone(&data), width, height) {
                                true
                            } else {
                                down.insert(Rc::clone(&data), width, height)
                            }
                        }
                        (Self::Leaf { .. }, Self::Node { .. }) => {
                            if right.insert(Rc::clone(&data), width, height) {
                                true
                            } else {
                                down.insert(Rc::clone(&data), width, height)
                            }
                        }
                        (Self::Node { .. }, Self::Leaf { .. }) => {
                            if down.insert(Rc::clone(&data), width, height) {
                                true
                            } else {
                                right.insert(Rc::clone(&data), width, height)
                            }
                        }
                        (Self::Node { .. }, Self::Node { .. }) => {
                            if right.insert(Rc::clone(&data), width, height) {
                                true
                            } else {
                                down.insert(Rc::clone(&data), width, height)
                            }
                        }
                    }
                } else {
                    false
                }
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
}

#[cfg(test)]
mod tree_2d_tests {
    use super::*;

    #[test]
    fn partition() {
        let data = Rc::new(1u32);
        let width = 2u32;
        let height = 2u32;

        let tree = Tree2d::partition(
            Rc::clone(&data),
            BoundingBox {
                x: 0,
                y: 0,
                width: 4,
                height: 4,
            },
            width,
            height,
        );

        let expected_bb_right = BoundingBox {
            x: 2,
            y: 0,
            width: 2,
            height: 4,
        };

        let expected_bb_down = BoundingBox {
            x: 0,
            y: 2,
            width: 2,
            height: 2,
        };

        match tree {
            Tree2d::Leaf { .. } => {
                assert!(false, "root should be a node")
            }
            Tree2d::Node { right, down, .. } => {
                match *right {
                    Tree2d::Leaf { bb } => {
                        assert_eq!(expected_bb_right, bb);
                    }
                    Tree2d::Node { .. } => {
                        assert!(false, "right remainder should be a leaf")
                    }
                };
                match *down {
                    Tree2d::Leaf { bb } => {
                        assert_eq!(expected_bb_down, bb);
                    }
                    Tree2d::Node { .. } => {
                        assert!(false, "down remainder should be a leaf")
                    }
                };
            }
        }
    }

    #[test]
    fn new_tree() {
        let data = Rc::new(1u32);
        let width = 4u32;
        let height = 4u32;

        let mut tree = Tree2d::<u32>::new(width, height);

        tree.insert(data, 2u32, 2u32);

        let expected_bb_right = BoundingBox {
            x: 2,
            y: 0,
            width: 2,
            height: 4,
        };

        let expected_bb_down = BoundingBox {
            x: 0,
            y: 2,
            width: 2,
            height: 2,
        };

        match tree {
            Tree2d::Leaf { .. } => {
                assert!(false, "root should be a node")
            }
            Tree2d::Node { right, down, .. } => {
                match *right {
                    Tree2d::Leaf { bb } => {
                        assert_eq!(expected_bb_right, bb);
                    }
                    Tree2d::Node { .. } => {
                        assert!(false, "right remainder should be a leaf")
                    }
                };
                match *down {
                    Tree2d::Leaf { bb } => {
                        assert_eq!(expected_bb_down, bb);
                    }
                    Tree2d::Node { .. } => {
                        assert!(false, "down remainder should be a leaf")
                    }
                };
            }
        }
    }
}
