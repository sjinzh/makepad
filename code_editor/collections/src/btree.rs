use std::{
    ops::{AddAssign, Deref, Index, SubAssign},
    sync::Arc,
};

#[derive(Clone)]
pub struct BTree<T: Chunk> {
    height: usize,
    root: Node<T>,
}

impl<T: Chunk> BTree<T> {
    pub fn new() -> Self {
        Self {
            height: 0,
            root: Node::Leaf(Leaf::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.root.summed_len()
    }

    pub fn info(&self) -> T::Info {
        self.root.summed_info()
    }

    pub fn prepend(&mut self, other: Self) {
        *self = other.concat(self.clone());
    }

    pub fn append(&mut self, other: Self) {
        *self = self.clone().concat(other);
    }

    pub fn concat(mut self, mut other: Self) -> Self {
        if self.height < other.height {
            if let Some(node) = other
                .root
                .prepend_at_depth(self.root, other.height - self.height)
            {
                let mut branch = Branch::new();
                branch.push_front(other.root);
                branch.push_front(node);
                other.height += 1;
                other.root = Node::Branch(branch);
            }
            other
        } else {
            if let Some(node) = self
                .root
                .append_at_depth(other.root, self.height - other.height)
            {
                let mut branch = Branch::new();
                branch.push_back(self.root);
                branch.push_back(node);
                self.height += 1;
                self.root = Node::Branch(branch);
            }
            self
        }
    }
}

pub struct Cursor<'a, T: Chunk> {
    root: &'a Node<T>,
    path: Vec<(&'a Branch<T>, usize)>,
}

impl<'a, T: Chunk> Cursor<'a, T> {
    pub fn current(&self) -> &T {
        self.current_node().as_leaf().as_chunk()
    }

    pub fn move_left(&mut self) {
        while let Some(&(_, index)) = self.path.last() {
            if index > 0 {
                break;
            }
            self.path.pop();
        }
        let (_, index) = self.path.last_mut().unwrap();
        *index -= 1;
        let mut node = self.current_node();
        loop {
            match node {
                Node::Leaf(_) => break,
                Node::Branch(branch) => {
                    self.path.push((branch, branch.len() - 1));
                    node = branch.first().unwrap();
                }
            }
        }
    }

    pub fn move_right(&mut self) {
        while let Some(&(branch, index)) = self.path.last() {
            if index < branch.len() {
                break;
            }
            self.path.pop();
        }
        let (_, index) = self.path.last_mut().unwrap();
        *index += 1;
        let mut node = self.current_node();
        loop {
            match node {
                Node::Leaf(_) => break,
                Node::Branch(branch) => {
                    self.path.push((branch, 0));
                    node = branch.last().unwrap();
                }
            }
        }
    }

    fn current_node(&self) -> &'a Node<T> {
        self
            .path
            .last()
            .map(|&(ref branch, index)| &branch[index])
            .unwrap_or(&self.root)
    }
}

pub trait Chunk: Clone {
    type Info: Info;

    const MAX_LEN: usize;

    fn new() -> Self;
    fn len(&self) -> usize;
    fn info(&self) -> Self::Info;
    fn move_left(&mut self, other: &mut Self, end: usize);
    fn move_right(&mut self, other: &mut Self, end: usize);
}

pub trait Info: Copy + AddAssign + SubAssign {
    fn new() -> Self;
}

#[derive(Clone)]
enum Node<T: Chunk> {
    Leaf(Leaf<T>),
    Branch(Branch<T>),
}

impl<T: Chunk> Node<T> {
    fn into_leaf(self) -> Leaf<T> {
        match self {
            Self::Leaf(leaf) => leaf,
            _ => panic!(),
        }
    }

    fn into_branch(self) -> Branch<T> {
        match self {
            Self::Branch(branch) => branch,
            _ => panic!(),
        }
    }

    fn as_leaf(&self) -> &Leaf<T> {
        match self {
            Self::Leaf(leaf) => leaf,
            _ => panic!(),
        }
    }

    fn summed_len(&self) -> usize {
        match self {
            Self::Leaf(leaf) => leaf.len(),
            Self::Branch(branch) => branch.summed_len(),
        }
    }

    fn summed_info(&self) -> T::Info {
        match self {
            Self::Leaf(leaf) => leaf.info(),
            Self::Branch(branch) => branch.summed_info(),
        }
    }

    fn as_mut_branch(&mut self) -> &mut Branch<T> {
        match self {
            Self::Branch(branch) => branch,
            _ => panic!(),
        }
    }

    fn prepend_at_depth(&mut self, other: Node<T>, depth: usize) -> Option<Self> {
        if depth == 0 {
            match self {
                Self::Leaf(leaf) => leaf
                    .prepend_or_distribute(other.into_leaf())
                    .map(|other_leaf| Node::Leaf(other_leaf)),
                Self::Branch(branch) => branch
                    .prepend_or_distribute(other.into_branch())
                    .map(|other_branch| Node::Branch(other_branch)),
            }
        } else {
            let branch = self.as_mut_branch();
            let mut node = branch.pop_front().unwrap();
            let other_node = node.prepend_at_depth(other, depth - 1);
            branch.push_front(node);
            other_node.and_then(|other_node| {
                branch
                    .push_front_and_maybe_split(other_node)
                    .map(|other_branch| Node::Branch(other_branch))
            })
        }
    }

    fn append_at_depth(&mut self, other: Node<T>, depth: usize) -> Option<Self> {
        if depth == 0 {
            match self {
                Self::Leaf(leaf) => leaf
                    .append_or_distribute(other.into_leaf())
                    .map(|other_leaf| Node::Leaf(other_leaf)),
                Self::Branch(branch) => branch
                    .append_or_distribute(other.into_branch())
                    .map(|other_branch| Node::Branch(other_branch)),
            }
        } else {
            let branch = self.as_mut_branch();
            let mut node = branch.pop_back().unwrap();
            let other_node = node.prepend_at_depth(other, depth - 1);
            branch.push_back(node);
            other_node.and_then(|other_node| {
                branch
                    .push_back_and_maybe_split(other_node)
                    .map(|other_branch| Node::Branch(other_branch))
            })
        }
    }
}

#[derive(Clone)]
struct Leaf<T> {
    chunk: T,
}

impl<T: Chunk> Leaf<T> {
    const MAX_LEN: usize = T::MAX_LEN;

    fn new() -> Self {
        Self {
            chunk: Chunk::new(),
        }
    }

    fn len(&self) -> usize {
        self.chunk.len()
    }

    fn info(&self) -> T::Info {
        self.chunk.info()
    }

    fn as_chunk(&self) -> &T {
        &self.chunk
    }

    fn prepend_or_distribute(&mut self, mut other: Self) -> Option<Self> {
        if self.len() + other.len() <= Self::MAX_LEN {
            other.move_right(self, self.len());
            return None;
        }
        other.distribute(self);
        Some(other)
    }

    fn append_or_distribute(&mut self, mut other: Self) -> Option<Self> {
        if self.len() + other.len() <= Self::MAX_LEN {
            let other_len = other.len();
            self.move_left(&mut other, other_len);
            return None;
        }
        self.distribute(&mut other);
        Some(other)
    }

    fn distribute(&mut self, other: &mut Self) {
        use std::cmp::Ordering;

        match self.len().cmp(&other.len()) {
            Ordering::Less => self.move_right(other, (other.len() - self.len()) / 2),
            Ordering::Greater => self.move_left(other, (self.len() + other.len()) / 2),
            _ => {}
        }
    }

    fn move_left(&mut self, other: &mut Self, end: usize) {
        self.chunk.move_left(&mut other.chunk, end);
    }

    fn move_right(&mut self, other: &mut Self, start: usize) {
        self.chunk.move_right(&mut other.chunk, start);
    }
}

#[derive(Clone)]
struct Branch<T: Chunk> {
    summed_len: usize,
    summed_info: T::Info,
    nodes: Arc<Vec<Node<T>>>,
}

impl<T: Chunk> Branch<T> {
    const MAX_LEN: usize = 8;

    fn new() -> Self {
        unimplemented!()
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn summed_len(&self) -> usize {
        self.summed_len
    }

    fn summed_info(&self) -> T::Info {
        self.summed_info
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn as_nodes(&self) -> &[Node<T>] {
        &self.nodes
    }

    fn push_front_and_maybe_split(&mut self, node: Node<T>) -> Option<Self> {
        if self.len() < Self::MAX_LEN {
            self.push_front(node);
            return None;
        }
        let mut other = Self::new();
        other.move_left(self, self.len() / 2);
        other.push_front(node);
        Some(other)
    }

    fn push_front(&mut self, node: Node<T>) {
        assert!(self.len() < Self::MAX_LEN);
        self.summed_len += node.summed_len();
        self.summed_info += node.summed_info();
        Arc::make_mut(&mut self.nodes).insert(0, node);
    }

    fn push_back_and_maybe_split(&mut self, node: Node<T>) -> Option<Self> {
        if self.len() < Self::MAX_LEN {
            self.push_back(node);
            return None;
        }
        let mut other = Self::new();
        self.move_right(&mut other, self.len() / 2);
        Some(other)
    }

    fn push_back(&mut self, node: Node<T>) {
        assert!(self.len() < Self::MAX_LEN);
        self.summed_len += node.summed_len();
        self.summed_info += node.summed_info();
        Arc::make_mut(&mut self.nodes).push(node);
    }

    fn pop_front(&mut self) -> Option<Node<T>> {
        if self.is_empty() {
            return None;
        }
        let node = Arc::make_mut(&mut self.nodes).pop().unwrap();
        self.summed_len -= node.summed_len();
        self.summed_info -= node.summed_info();
        Some(node)
    }

    fn pop_back(&mut self) -> Option<Node<T>> {
        if self.is_empty() {
            return None;
        }
        let node = Arc::make_mut(&mut self.nodes).remove(0);
        self.summed_len -= node.summed_len();
        self.summed_info -= node.summed_info();
        Some(node)
    }

    fn prepend_or_distribute(&mut self, mut other: Self) -> Option<Self> {
        if self.len() + other.len() <= Self::MAX_LEN {
            other.move_right(self, self.len());
            return None;
        }
        other.distribute(self);
        Some(other)
    }

    fn append_or_distribute(&mut self, mut other: Self) -> Option<Self> {
        if self.len() + other.len() <= Self::MAX_LEN {
            let other_len = other.len();
            self.move_left(&mut other, other_len);
            return None;
        }
        self.distribute(&mut other);
        Some(other)
    }

    fn distribute(&mut self, other: &mut Self) {
        use std::cmp::Ordering;

        match self.len().cmp(&other.len()) {
            Ordering::Less => self.move_right(other, (other.len() - self.len()) / 2),
            Ordering::Greater => self.move_left(other, (self.len() + other.len()) / 2),
            _ => {}
        }
    }

    fn move_left(&mut self, other: &mut Self, end: usize) {
        let summed_len = sum_lens(&other.nodes[..end]);
        let summed_info = sum_infos(&other.nodes[..end]);
        other.summed_len -= summed_len;
        other.summed_info -= summed_info;
        self.summed_len += summed_len;
        self.summed_info += summed_info;
        let nodes = Arc::make_mut(&mut other.nodes).drain(..end);
        Arc::make_mut(&mut self.nodes).extend(nodes);
    }

    fn move_right(&mut self, other: &mut Self, start: usize) {
        let len = sum_lens(&self.nodes[start..]);
        let info = sum_infos(&self.nodes[start..]);
        self.summed_len -= len;
        self.summed_info -= info;
        other.summed_len += len;
        other.summed_info += info;
        let nodes = Arc::make_mut(&mut self.nodes).drain(start..);
        Arc::make_mut(&mut other.nodes).splice(..0, nodes);
    }
}

impl<T: Chunk> Deref for Branch<T> {
    type Target = [Node<T>];

    fn deref(&self) -> &Self::Target {
        self.as_nodes()
    }
}

impl<T: Chunk> Index<usize> for Branch<T> {
    type Output = Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

fn sum_lens<T: Chunk>(nodes: &[Node<T>]) -> usize {
    let mut summed_len = 0;
    for node in nodes {
        summed_len += node.summed_len();
    }
    summed_len
}

fn sum_infos<T: Chunk>(nodes: &[Node<T>]) -> T::Info {
    let mut summed_info = T::Info::new();
    for node in nodes {
        summed_info += node.summed_info();
    }
    summed_info
}