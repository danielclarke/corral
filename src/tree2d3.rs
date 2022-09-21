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
    bb: BoundingBox,
    right: Box<Tree2d<T>>,
    down: Box<Tree2d<T>>,
    data: Rc<T>,
}

struct Leaf {
    bb: BoundingBox,
}

enum Tree2d<T> {
    Leaf(Leaf),
    Node(Node<T>),
}

impl<T> Tree2d<T> {
    pub fn insert(&mut self, width: u32, height: u32, data: T) -> bool {
        let optree = self.get_smallest_leaf_for_data(width, height);
        match optree {
            None => false,
            Some((tree, _)) => {
                match tree {
                    Self::Leaf(leaf) => {
                        *tree = Self::partition(Rc::new(data), leaf.bb, width, height);
                        true
                    },
                    Self::Node( .. ) => unreachable!()  
                }
                
            }
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

        Self::Node(Node {
            bb,
            right: Box::new(Self::Leaf(Leaf { bb: bb_right })),
            down: Box::new(Self::Leaf(Leaf { bb: bb_down })),
            data,
        })
    }

    fn get_smallest_leaf_for_data(
        &mut self,
        width: u32,
        height: u32,
    ) -> Option<(&mut Self, BoundingBox)> {
        // match self {
        //     Self::Leaf(leaf) => Some((self, leaf.bb)),
        //     Self::Node(node) => node.right.get_smallest_leaf_for_data(width, height),
        // }

        let is_leaf = match self {
            Self::Leaf(..) => true,
            Self::Node(..) => false,
        };

        if is_leaf {
            Some((
                self,
                BoundingBox {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
            ))
        } else {
            match self {
                Self::Leaf(..) => unreachable!(),
                Self::Node(node) => node.right.get_smallest_leaf_for_data(width, height),
            }
        }
    }
}