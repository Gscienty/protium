use std::{cell, rc};

use crate::utils;

pub(super) type Tree<T> = Option<rc::Rc<cell::RefCell<TreeNode<T>>>>;

#[derive(Debug)]
pub(super) struct TreeNode<T> {
    value: T,

    parent: Tree<T>,
    child_0: Tree<T>,
    child_1: Tree<T>,
}

#[derive(Debug)]
pub(super) struct PackingData {
    used: bool,
    region: utils::Rectangle,
}

#[derive(Debug)]
pub(super) struct Page {
    root: Tree<PackingData>,

    width: i32,
    height: i32,
    max_size: i32,
}

impl<T> TreeNode<T> {
    fn new(parent: Tree<T>, value: T) -> Tree<T> {
        Some(rc::Rc::new(cell::RefCell::new(Self {
            value,

            parent,
            child_0: None,
            child_1: None,
        })))
    }

    fn set_child_0(&mut self, parent: &Tree<T>, value: T) {
        self.child_0 = Self::new(parent.clone(), value);
    }

    fn set_child_1(&mut self, parent: &Tree<T>, value: T) {
        self.child_1 = Self::new(parent.clone(), value);
    }

    fn assign_child_0(parent: &Tree<T>, child: &Tree<T>) {
        {
            let Some(parent) = parent.as_ref() else {
                return;
            };
            parent.borrow_mut().child_0 = child.clone();
        }
        {
            let Some(child) = child.as_ref() else {
                return;
            };
            child.borrow_mut().parent = parent.clone();
        }
    }

    fn assign_child_1(parent: &Tree<T>, child: &Tree<T>) {
        {
            let Some(parent) = parent.as_ref() else {
                return;
            };
            parent.borrow_mut().child_1 = child.clone();
        }
        {
            let Some(child) = child.as_ref() else {
                return;
            };
            child.borrow_mut().parent = parent.clone();
        }
    }

    fn get_value(&self) -> &T {
        &self.value
    }
}

impl TreeNode<PackingData> {
    pub(crate) fn get_region(&self) -> utils::Rectangle {
        self.value.region
    }

    fn allow_free(&self) -> bool {
        if self.value.used {
            return false;
        }

        if let Some(node) = self.child_0.as_ref() {
            if !node.borrow().allow_free() {
                return false;
            }
        }
        if let Some(node) = self.child_1.as_ref() {
            if !node.borrow().allow_free() {
                return false;
            }
        }

        true
    }

    fn check_out_of_region(&self, width: i32, height: i32) -> bool {
        self.value.region.get_width() < width || self.value.region.get_height() < height
    }

    fn check_equal_region(&self, width: i32, height: i32) -> bool {
        self.value.region.get_width() == width && self.value.region.get_height() == height
    }

    fn is_used(&self) -> bool {
        self.value.used
    }

    fn set_used(&mut self, used: bool) {
        self.value.used = used;
    }

    fn split_horizontal_child(&mut self, parent: &Tree<PackingData>, width: i32) {
        self.set_child_0(
            parent,
            PackingData {
                region: utils::Rectangle::new_pos(
                    self.value.region.min.x,
                    self.value.region.min.y,
                    self.value.region.min.x + width,
                    self.value.region.max.y,
                ),
                used: false,
            },
        );

        self.set_child_1(
            parent,
            PackingData {
                region: utils::Rectangle::new_pos(
                    self.value.region.min.x + width,
                    self.value.region.min.y,
                    self.value.region.max.x,
                    self.value.region.max.y,
                ),
                used: false,
            },
        );
    }

    fn split_vertical_child(&mut self, parent: &Tree<PackingData>, height: i32) {
        self.set_child_0(
            parent,
            PackingData {
                region: utils::Rectangle::new_pos(
                    self.value.region.min.x,
                    self.value.region.min.y,
                    self.value.region.max.x,
                    self.value.region.min.y + height,
                ),
                used: false,
            },
        );

        self.set_child_1(
            parent,
            PackingData {
                region: utils::Rectangle::new_pos(
                    self.value.region.min.x,
                    self.value.region.min.y + height,
                    self.value.region.max.x,
                    self.value.region.max.y,
                ),
                used: false,
            },
        );
    }

    fn alloc(parent: &Tree<PackingData>, width: i32, height: i32) -> Tree<PackingData> {
        let node = parent.as_ref()?;
        if node.borrow().check_out_of_region(width, height) || node.borrow().is_used() {
            return None;
        }

        if node.borrow().child_0.is_none() && node.borrow().child_1.is_none() {
            if node.borrow().check_equal_region(width, height) {
                node.borrow_mut().set_used(true);
                return parent.clone();
            }

            let (node_width, node_height) = (
                node.borrow().get_value().region.get_width(),
                node.borrow().get_value().region.get_height(),
            );

            if Self::square(node_width - width, node_height)
                >= Self::square(node_width, node_height - height)
            {
                node.borrow_mut().split_horizontal_child(parent, width);
            } else {
                node.borrow_mut().split_vertical_child(parent, height);
            }

            return Self::alloc(&node.borrow().child_0, width, height);
        }

        if node.borrow().child_0.is_none() || node.borrow().child_1.is_none() {
            return None;
        }
        if let Some(child) = Self::alloc(&node.borrow().child_0, width, height) {
            return Some(child);
        }
        if let Some(child) = Self::alloc(&node.borrow().child_1, width, height) {
            return Some(child);
        }

        None
    }

    pub(super) fn free(node: &Tree<PackingData>) {
        let Some(node) = node.as_ref() else {
            return;
        };

        if node.borrow().child_0.is_some() || node.borrow().child_1.is_some() {
            return;
        }
        node.borrow_mut().set_used(false);

        let allow_free = if let Some(parent) = node.borrow().parent.as_ref() {
            let bro_0_allow_free = if let Some(bro_0) = parent.borrow().child_0.as_ref() {
                bro_0.borrow().allow_free()
            } else {
                return;
            };
            let bro_1_allow_free = if let Some(bro_1) = parent.borrow().child_1.as_ref() {
                bro_1.borrow().allow_free()
            } else {
                return;
            };

            let allow_free = bro_0_allow_free && bro_1_allow_free;

            if allow_free {
                parent.borrow_mut().child_0 = None;
                parent.borrow_mut().child_1 = None;
            }

            allow_free
        } else {
            return;
        };

        if allow_free {
            Self::free(&node.borrow().parent);
        }
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

    fn walk<F>(node: &Tree<PackingData>, mut cb: F) -> bool
    where
        F: FnMut(&Tree<PackingData>) -> bool + Copy,
    {
        if !cb(node) {
            return false;
        }

        if let Some(node) = node.as_ref() {
            if !Self::walk(&node.borrow().child_0, cb) {
                return false;
            }
            if !Self::walk(&node.borrow().child_1, cb) {
                return false;
            }
        }

        true
    }

    fn clone_tree(tree: &Tree<PackingData>) -> Tree<PackingData> {
        let tree = tree.as_ref()?;

        Some(rc::Rc::new(cell::RefCell::new(Self {
            value: PackingData {
                used: tree.borrow().value.used,
                region: tree.borrow().value.region,
            },

            parent: tree.borrow().parent.clone(),
            child_0: tree.borrow().child_0.clone(),
            child_1: tree.borrow().child_1.clone(),
        })))
    }
}

impl Page {
    pub(crate) fn new(width: i32, height: i32, max_size: i32) -> Self {
        Self {
            root: None,

            width,
            height,
            max_size,
        }
    }

    pub(crate) fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    // pub(crate) fn is_empty(&self) -> bool {
    //     let Some(root) = self.root.as_ref() else {
    //         return true;
    //     };
    //     let Ok(root) = root.lock() else {
    //         return true;
    //     };
    //
    //     !root.value.used && root.child_0.is_none() && root.child_1.is_none()
    // }

    pub(crate) fn alloc(&mut self, width: i32, height: i32) -> Tree<PackingData> {
        if self.root.is_none() {
            self.root = TreeNode::new(
                None,
                PackingData {
                    region: utils::Rectangle::new_pos(0, 0, width, height),
                    used: false,
                },
            );
        }

        self.extend_and_alloc(width, height)
    }

    fn extend_and_alloc(&mut self, width: i32, height: i32) -> Tree<PackingData> {
        if let Some(node) = TreeNode::alloc(&self.root, width, height) {
            return Some(node);
        }
        if self.width >= self.max_size && self.height >= self.max_size {
            return None;
        }

        let mut i = 0;
        loop {
            i += 1;

            for j in 0..=i {
                let mut new_page_width = self.width;
                for _ in 0..(i - j) {
                    new_page_width *= 2;
                }
                let mut new_page_height = self.height;
                for _ in 0..j {
                    new_page_height *= 2;
                }
                if new_page_width > self.max_size || new_page_height > self.max_size {
                    continue;
                }

                let rollback = self.extend(new_page_width, new_page_height);
                self.root.as_ref()?;
                if let Some(node) = TreeNode::alloc(&self.root, width, height) {
                    return Some(node);
                }
                self.rollback(rollback);

                if new_page_width >= self.max_size && new_page_height >= self.max_size {
                    return None;
                }
            }
        }
    }

    fn extend(&mut self, width: i32, height: i32) -> ExtendRollback {
        let edge_nodes = rc::Rc::new(cell::RefCell::new(Vec::new()));
        let aborted = !TreeNode::walk(&self.root, |node_| {
            let Some(node) = node_.as_ref() else {
                return true;
            };
            if node.borrow().value.region.max.x < self.width
                && node.borrow().value.region.max.y < self.height
            {
                return true;
            }
            if node.borrow().is_used() {
                return false;
            }
            edge_nodes.borrow_mut().push(node_.clone());

            true
        });

        let rollback = if aborted {
            let cloned_root = TreeNode::clone_tree(&self.root);

            if height > self.height {
                let upper = self.root.clone();
                let lower = TreeNode::new(
                    None,
                    PackingData {
                        used: false,
                        region: utils::Rectangle::new(0, self.height, self.width, height),
                    },
                );

                self.root = TreeNode::new(
                    None,
                    PackingData {
                        used: false,
                        region: utils::Rectangle::new(0, 0, self.width, height),
                    },
                );
                TreeNode::assign_child_0(&self.root, &upper);
                TreeNode::assign_child_1(&self.root, &lower);
            }
            if width > self.width {
                let left = self.root.clone();
                let right = TreeNode::new(
                    None,
                    PackingData {
                        used: false,
                        region: utils::Rectangle::new(self.width, 0, width, height),
                    },
                );
                self.root = TreeNode::new(
                    None,
                    PackingData {
                        used: false,
                        region: utils::Rectangle::new(0, 0, width, height),
                    },
                );
                TreeNode::assign_child_0(&self.root, &left);
                TreeNode::assign_child_1(&self.root, &right);
            }

            ExtendRollback::Aborted(Aborted {
                width: self.width,
                height: self.height,
                root: cloned_root,
            })
        } else {
            let mut x_edges = Vec::new();
            let mut y_edges = Vec::new();
            for node_ in edge_nodes.borrow().iter() {
                let Some(node) = node_.as_ref() else {
                    continue;
                };

                if node.borrow().value.region.max.x == self.width {
                    x_edges.push((node_.clone(), node.borrow().value.region.max.x));
                    node.borrow_mut().value.region.max.x = width;
                }
                if node.borrow().value.region.max.y == self.height {
                    y_edges.push((node_.clone(), node.borrow().value.region.max.y));
                    node.borrow_mut().value.region.max.y = height;
                }
            }

            ExtendRollback::NonAborted(NonAborted {
                width: self.width,
                height: self.height,
                x_edges,
                y_edges,
            })
        };

        self.width = width;
        self.height = height;

        rollback
    }

    fn rollback(&mut self, rollback: ExtendRollback) {
        match rollback {
            ExtendRollback::Aborted(Aborted {
                width,
                height,
                root,
            }) => {
                self.width = width;
                self.height = height;
                self.root = root;
            }
            ExtendRollback::NonAborted(NonAborted {
                width,
                height,
                x_edges,
                y_edges,
            }) => {
                self.width = width;
                self.height = height;
                for (node, x) in x_edges {
                    let Some(node) = node.as_ref() else {
                        continue;
                    };
                    node.borrow_mut().value.region.max.x = x;
                }
                for (node, y) in y_edges {
                    let Some(node) = node.as_ref() else {
                        continue;
                    };
                    node.borrow_mut().value.region.max.y = y;
                }
            }
        }
    }
}

struct Aborted {
    width: i32,
    height: i32,
    root: Tree<PackingData>,
}

struct NonAborted {
    width: i32,
    height: i32,
    x_edges: Vec<(Tree<PackingData>, i32)>,
    y_edges: Vec<(Tree<PackingData>, i32)>,
}

enum ExtendRollback {
    Aborted(Aborted),
    NonAborted(NonAborted),
}
