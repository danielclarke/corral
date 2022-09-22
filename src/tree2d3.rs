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

pub struct Tree<T> {
    root: Branch<T>
}

type Branch<T> = Option<Box<Node<T>>>;

struct Node<T> {
    bb: BoundingBox,
    right: Branch<T>,
    down: Branch<T>,
    data: Option<Rc<T>>
}

impl<T>