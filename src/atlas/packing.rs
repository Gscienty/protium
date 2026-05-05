use std::{cell::RefCell, f64, rc::Rc};

use crate::utils;

#[derive(Debug, Clone)]
pub(crate) struct NodeInner {
    region: utils::Rectangle,
    used: bool,

    parent: Option<Rc<RefCell<NodeInner>>>,
    child_0: Option<Rc<RefCell<NodeInner>>>,
    child_1: Option<Rc<RefCell<NodeInner>>>,
}

pub(crate) type Node = Rc<RefCell<NodeInner>>;

impl NodeInner {
    pub(crate) fn get_region(&self) -> utils::Rectangle {
        self.region
    }

    fn can_free(&self) -> bool {
        if self.used {
            return false;
        }

        if let Some(child) = &self.child_0
            && !child.borrow().can_free()
        {
            return false;
        }
        if let Some(child) = &self.child_1
            && !child.borrow().can_free()
        {
            return false;
        }

        true
    }

    fn alloc(
        parent: &Rc<RefCell<NodeInner>>,
        width: i32,
        height: i32,
    ) -> Option<Rc<RefCell<NodeInner>>> {
        {
            let node = parent.borrow_mut();
            if node.region.get_width() < width || node.region.get_height() < height {
                return None;
            }
            if node.used {
                return None;
            }
        }

        if parent.borrow().child_0.is_none() && parent.borrow().child_1.is_none() {
            if parent.borrow().region.get_width() == width
                && parent.borrow().region.get_height() == height
            {
                parent.borrow_mut().used = true;
                return Some(parent.clone());
            }

            let mut node = parent.borrow_mut();
            if square(node.region.get_width() - width, node.region.get_height())
                >= square(node.region.get_width(), node.region.get_height() - height)
            {
                node.child_0 = Some(Rc::new(RefCell::new(Self {
                    region: utils::Rectangle::new_pos(
                        node.region.min.x,
                        node.region.min.y,
                        node.region.min.x + width,
                        node.region.max.y,
                    ),
                    used: false,

                    parent: Some(parent.clone()),
                    child_0: None,
                    child_1: None,
                })));

                node.child_1 = Some(Rc::new(RefCell::new(Self {
                    region: utils::Rectangle::new_pos(
                        node.region.min.x + width,
                        node.region.min.y,
                        node.region.max.x,
                        node.region.max.y,
                    ),
                    used: false,

                    parent: Some(parent.clone()),
                    child_0: None,
                    child_1: None,
                })));
            } else {
                node.child_0 = Some(Rc::new(RefCell::new(Self {
                    region: utils::Rectangle::new_pos(
                        node.region.min.x,
                        node.region.min.y,
                        node.region.max.x,
                        node.region.min.y + height,
                    ),
                    used: false,

                    parent: Some(parent.clone()),
                    child_0: None,
                    child_1: None,
                })));

                node.child_1 = Some(Rc::new(RefCell::new(Self {
                    region: utils::Rectangle::new_pos(
                        node.region.min.x,
                        node.region.min.y + height,
                        node.region.max.x,
                        node.region.max.y,
                    ),
                    used: false,

                    parent: Some(parent.clone()),
                    child_0: None,
                    child_1: None,
                })));
            }

            if let Some(node) = &node.child_0 {
                return Self::alloc(node, width, height);
            } else {
                return None;
            }
        }

        let node = parent.borrow();
        if node.child_0.is_none() || node.child_1.is_none() {
            return None;
        }
        if let Some(child) = node.child_0.as_ref()
            && let Some(node) = Self::alloc(child, width, height)
        {
            return Some(node);
        }
        if let Some(child) = node.child_1.as_ref()
            && let Some(node) = Self::alloc(child, width, height)
        {
            return Some(node);
        }
        None
    }

    fn walk<F>(node: &Rc<RefCell<Self>>, mut f: F) -> Result<(), ()>
    where
        F: FnMut(&Rc<RefCell<Self>>) -> Result<(), ()> + Copy,
    {
        f(node)?;

        if let Some(child) = &node.borrow().child_0 {
            Self::walk(child, f)?;
        }
        if let Some(child) = &node.borrow().child_1 {
            Self::walk(child, f)?;
        }

        Ok(())
    }
}

pub(crate) fn free(node: &Rc<RefCell<NodeInner>>) -> Result<(), ()> {
    {
        let node = node.borrow();
        if node.child_0.is_some() || node.child_1.is_some() {
            return Err(());
        }
    }
    node.borrow_mut().used = false;

    let (parent, can_free) = if let Some(parent) = &node.borrow().parent {
        let parent_node = parent.borrow();

        let left_can_free = if let Some(left) = &parent_node.child_0 {
            left.borrow().can_free()
        } else {
            return Err(());
        };
        let right_can_free = if let Some(right) = &parent_node.child_1 {
            right.borrow().can_free()
        } else {
            return Err(());
        };

        (parent.clone(), left_can_free && right_can_free)
    } else {
        return Ok(());
    };
    if can_free {
        {
            let mut parent = parent.borrow_mut();
            parent.child_0 = None;
            parent.child_1 = None;
        }

        free(&parent)?;
    }

    Ok(())
}

fn square(width: i32, height: i32) -> f64 {
    if width == 0 && height == 0 {
        return 0.;
    }

    if width <= height {
        return width as f64 / height as f64;
    }
    height as f64 / width as f64
}

#[derive(Debug)]
pub(crate) struct Page {
    root: Option<Rc<RefCell<NodeInner>>>,

    width: i32,
    height: i32,
    max_size: i32,
}

impl Page {
    pub(crate) fn new(width: i32, height: i32, max_size: i32) -> Page {
        Self {
            root: None,

            width,
            height,
            max_size,
        }
    }

    pub(crate) const fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub(crate) fn is_empty(&self) -> bool {
        if let Some(root) = &self.root {
            let root = root.borrow();

            !root.used && root.child_0.is_none() && root.child_1.is_none()
        } else {
            true
        }
    }

    pub(crate) fn alloc(&mut self, width: i32, height: i32) -> Option<Node> {
        if self.root.is_none() {
            self.root = Some(Rc::new(RefCell::new(NodeInner {
                region: utils::Rectangle::new_pos(0, 0, self.width, self.height),
                used: false,

                parent: None,
                child_0: None,
                child_1: None,
            })));
        }

        self.extend_and_alloc(width, height)
    }

    fn extend_and_alloc(&mut self, width: i32, height: i32) -> Option<Node> {
        let Some(root) = &self.root else {
            return None;
        };
        if let Some(node) = NodeInner::alloc(root, width, height) {
            return Some(node);
        };
        if self.width >= self.max_size && self.height >= self.max_size {
            return None;
        }
        let mut i = 0;
        loop {
            i += 1;

            for j in 0..=i {
                let mut new_width = self.width;
                for _ in 0..(i - j) {
                    new_width *= 2;
                }
                let mut new_height = self.height;
                for _ in 0..j {
                    new_height *= 2;
                }

                if new_width > self.max_size || new_height > self.max_size {
                    continue;
                }

                let rollback = self.extend(new_width, new_height);
                let Some(root) = &self.root else {
                    return None;
                };
                if let Some(node) = NodeInner::alloc(root, new_width, new_height) {
                    return Some(node);
                }
                rollback.apply(self);

                if new_width >= self.max_size && new_height >= self.max_size {
                    return None;
                }
            }
        }
    }

    fn extend(&mut self, new_width: i32, new_height: i32) -> ExtendRollback {
        let edge_nodes = Rc::new(RefCell::new(Vec::new()));
        let mut aborted = false;
        if let Some(root) = &self.root {
            aborted = NodeInner::walk(root, |node| {
                if node.borrow().region.max.x < self.width
                    && node.borrow().region.max.y < self.height
                {
                    return Ok(());
                }
                if node.borrow().used {
                    return Err(());
                }
                edge_nodes.borrow_mut().push(node.clone());

                Ok(())
            })
            .is_err();
        }

        let rollback = if aborted {
            let root_value = self.root.as_ref().map(|root| NodeInner {
                region: root.borrow().region,
                used: root.borrow().used,

                parent: None,
                child_0: root.borrow().child_0.clone(),
                child_1: root.borrow().child_1.clone(),
            });

            if new_height > self.height {
                let upper = self.root.clone();
                let lower = Some(Rc::new(RefCell::new(NodeInner {
                    region: utils::Rectangle::new_pos(0, self.height, self.width, new_height),
                    used: false,

                    parent: None,
                    child_0: None,
                    child_1: None,
                })));
                self.root = Some(Rc::new(RefCell::new(NodeInner {
                    region: utils::Rectangle::new_pos(0, 0, self.width, new_height),
                    used: false,

                    parent: None,
                    child_0: upper.clone(),
                    child_1: lower.clone(),
                })));

                if let Some(node) = &upper {
                    node.borrow_mut().parent = self.root.clone();
                }
                if let Some(node) = &lower {
                    node.borrow_mut().parent = self.root.clone();
                }
            }
            if new_width > self.width {
                let left = self.root.clone();
                let right = Some(Rc::new(RefCell::new(NodeInner {
                    region: utils::Rectangle::new_pos(self.width, 0, new_width, new_height),

                    used: false,

                    parent: None,
                    child_0: None,
                    child_1: None,
                })));
                self.root = Some(Rc::new(RefCell::new(NodeInner {
                    region: utils::Rectangle::new_pos(0, 0, new_width, new_height),
                    used: false,

                    parent: None,
                    child_0: left.clone(),
                    child_1: right.clone(),
                })));
                if let Some(node) = &left {
                    node.borrow_mut().parent = self.root.clone();
                }
                if let Some(node) = &right {
                    node.borrow_mut().parent = self.root.clone();
                }
            }

            ExtendRollback::Aborted(ExtendRollbackAborted {
                width: self.width,
                height: self.height,
                root: self.root.clone(),
                root_value,
            })
        } else {
            let mut origin_x = Vec::new();
            let mut origin_y = Vec::new();
            for node in edge_nodes.borrow().iter() {
                if node.borrow().region.max.x == self.width {
                    origin_x.push((node.clone(), node.borrow().region.max.x));
                    node.borrow_mut().region.max.x = new_width;
                }
                if node.borrow().region.max.y == self.height {
                    origin_y.push((node.clone(), node.borrow().region.max.y));
                    node.borrow_mut().region.max.y = new_height;
                }
            }
            ExtendRollback::NonAborted(ExtendRollbackNonAborted {
                width: self.width,
                height: self.height,
                origin_x,
                origin_y,
            })
        };

        self.width = new_width;
        self.height = new_height;

        rollback
    }
}

struct ExtendRollbackAborted {
    width: i32,
    height: i32,
    root: Option<Rc<RefCell<NodeInner>>>,
    root_value: Option<NodeInner>,
}

struct ExtendRollbackNonAborted {
    width: i32,
    height: i32,
    origin_x: Vec<(Rc<RefCell<NodeInner>>, i32)>,
    origin_y: Vec<(Rc<RefCell<NodeInner>>, i32)>,
}

enum ExtendRollback {
    Aborted(ExtendRollbackAborted),
    NonAborted(ExtendRollbackNonAborted),
}

impl ExtendRollback {
    fn apply(self, page: &mut Page) {
        match self {
            Self::Aborted(history) => {
                page.width = history.width;
                page.height = history.height;
                page.root = history.root;
                let Some(history_value) = history.root_value else {
                    return;
                };
                if let Some(root) = &page.root {
                    let mut root = root.borrow_mut();
                    root.region = history_value.region;
                    root.used = history_value.used;
                    root.parent = None;
                    root.child_0 = history_value.child_0;
                    root.child_1 = history_value.child_1;
                }
            }
            Self::NonAborted(history) => {
                page.width = history.width;
                page.height = history.height;

                for (node, x) in history.origin_x {
                    node.borrow_mut().region.max.x = x;
                }
                for (node, y) in history.origin_y {
                    node.borrow_mut().region.max.y = y;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Op {
        width: i32,
        height: i32,
        free_node_id: i32,
    }

    impl Op {
        fn new(width: i32, height: i32, free_node_id: i32) -> Self {
            Self {
                width,
                height,
                free_node_id,
            }
        }
    }

    struct Case {
        name: String,
        in_op: Vec<Op>,
        out_rect: Vec<Option<utils::Rectangle>>,
    }

    #[test]
    fn test_page() {
        let cases = [
            Case {
                name: "alloc & random free".to_string(),
                in_op: vec![
                    Op::new(100, 100, -1),
                    Op::new(100, 100, -1),
                    Op::new(100, 100, -1),
                    Op::new(100, 100, -1),
                    Op::new(100, 100, -1),
                    Op::new(100, 100, -1),
                    Op::new(0, 0, 1),
                    Op::new(0, 0, 3),
                    Op::new(0, 0, 5),
                    Op::new(0, 0, 0),
                    Op::new(0, 0, 2),
                    Op::new(0, 0, 4),
                    Op::new(1024, 1024, -1),
                ],
                out_rect: vec![
                    Some(utils::Rectangle::new_pos(0, 0, 100, 100)),
                    Some(utils::Rectangle::new_pos(0, 100, 100, 200)),
                    Some(utils::Rectangle::new_pos(0, 200, 100, 300)),
                    Some(utils::Rectangle::new_pos(0, 300, 100, 400)),
                    Some(utils::Rectangle::new_pos(0, 400, 100, 500)),
                    Some(utils::Rectangle::new_pos(0, 500, 100, 600)),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(utils::Rectangle::new_pos(0, 0, 1024, 1024)),
                ],
            },
            Case {
                name: "alloc & free".to_string(),
                in_op: vec![
                    Op::new(31, 41, -1),
                    Op::new(59, 26, -1),
                    Op::new(53, 58, -1),
                    Op::new(97, 93, -1),
                    Op::new(28, 84, -1),
                    Op::new(62, 64, -1),
                    Op::new(0, 0, 0),
                    Op::new(0, 0, 1),
                    Op::new(0, 0, 2),
                    Op::new(0, 0, 3),
                    Op::new(0, 0, 4),
                    Op::new(0, 0, 5),
                ],
                out_rect: vec![
                    Some(utils::Rectangle::new_pos(0, 0, 31, 41)),
                    Some(utils::Rectangle::new_pos(31, 0, 31 + 59, 26)),
                    Some(utils::Rectangle::new_pos(31, 26, 31 + 53, 26 + 58)),
                    Some(utils::Rectangle::new_pos(31, 84, 31 + 97, 84 + 93)),
                    Some(utils::Rectangle::new_pos(0, 41, 28, 41 + 84)),
                    Some(utils::Rectangle::new_pos(31, 177, 31 + 62, 177 + 64)),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
            },
            Case {
                name: "fill square".to_string(),
                in_op: vec![
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                    Op::new(256, 256, -1),
                ],
                out_rect: vec![
                    Some(utils::Rectangle::new_pos(0, 0, 256, 256)),
                    Some(utils::Rectangle::new_pos(0, 256, 256, 512)),
                    Some(utils::Rectangle::new_pos(0, 512, 256, 768)),
                    Some(utils::Rectangle::new_pos(0, 768, 256, 1024)),
                    Some(utils::Rectangle::new_pos(256, 0, 512, 256)),
                    Some(utils::Rectangle::new_pos(512, 0, 768, 256)),
                    Some(utils::Rectangle::new_pos(768, 0, 1024, 256)),
                    Some(utils::Rectangle::new_pos(256, 256, 512, 512)),
                    Some(utils::Rectangle::new_pos(256, 512, 512, 768)),
                    Some(utils::Rectangle::new_pos(256, 768, 512, 1024)),
                    Some(utils::Rectangle::new_pos(512, 256, 768, 512)),
                    Some(utils::Rectangle::new_pos(768, 256, 1024, 512)),
                    Some(utils::Rectangle::new_pos(512, 512, 768, 768)),
                    Some(utils::Rectangle::new_pos(512, 768, 768, 1024)),
                    Some(utils::Rectangle::new_pos(768, 512, 1024, 768)),
                    Some(utils::Rectangle::new_pos(768, 768, 1024, 1024)),
                    None,
                ],
            },
        ];

        for case in cases.iter() {
            let mut p = Page::new(1024, 1024, 1024);
            let mut nodes = Vec::new();
            let mut nnodes = 0;

            for inp in case.in_op.iter() {
                if inp.free_node_id == -1 {
                    let n = p.alloc(inp.width, inp.height);
                    nodes.push(n);
                    nnodes += 1;
                } else {
                    let Some(node) = nodes[inp.free_node_id as usize].as_ref() else {
                        continue;
                    };
                    let _ = free(node);
                    nodes.push(None);
                    nnodes -= 1;
                }
            }

            assert_eq!(p.is_empty(), nnodes == 0);

            for (i, &out) in case.out_rect.iter().enumerate() {
                if let Some(node) = nodes[i].as_ref() {
                    let region = node.borrow().region;
                    println!("{:?} {:?}", out, region);
                    assert!(matches!(out, Some(out_region) if out_region == region));
                } else {
                    assert!(out.is_none());
                }
            }
        }
    }
}
