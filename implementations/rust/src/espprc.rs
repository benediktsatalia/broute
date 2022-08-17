use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use crate::tsp;
use std::collections::VecDeque;

struct Label {
    at: usize,
    visited: Vec<bool>,
    ignore: Cell<bool>,
    predecessor: Option<Weak<Label>>,
    cost: f64,
    length: i32,
    q: Vec<usize>,
    successors: RefCell<Vec<Rc<Label>>>,
}

impl Label {
    fn empty(d: &tsp::TSPData, nresources: usize) -> Rc<Label> {
        let visited: Vec<bool> = vec![false; d.n];
        let q: Vec<usize> = vec![0; nresources];
        Rc::new(Label {
            at: 0,
            visited,
            ignore: Cell::new(false),
            predecessor: None,
            cost: 0.0,
            length: 0,
            q,
            successors: RefCell::new(Vec::new()),
        })
    }

    fn ignore(&self) {
        self.ignore.set(true);
    }

    fn is_ignored(&self) -> bool {
        self.ignore.get()
    }

    fn dominates(&self, other: &Label) -> bool {
        if self.cost > other.cost || self.length > other.length {
            return false;
        }
        for (v1, v2) in self.visited.iter().zip(other.visited.iter()) {
            if v1 > v2 {
                return false;
            }
        }
        for (q1, q2) in self.q.iter().zip(other.q.iter()) {
            if q1 > q2 {
                return false;
            }
        }
        return true;
    }

    // extend label to given vertex
    fn extend(d: &tsp::TSPData, from: &Rc<Label>, vertex: usize) -> Rc<Label> {
        let mut visited = from.visited.clone();
        visited[vertex] = true;
        let mut q = from.q.clone();
        for (i, value) in q.iter_mut().enumerate() {
            if (vertex & (1 << i)) > 0 {
                *value += 1;
            }
        }
        let cost = from.cost + d.aux(from.at, vertex);
        let length = from.length + d.d(from.at, vertex);
        Rc::new(Label {
            at: vertex,
            visited,
            ignore: Cell::new(false),
            predecessor: Some(Rc::downgrade(from)),
            cost,
            length,
            q,
            successors: RefCell::new(Vec::new()),
        })
    }

    fn addsuccessor(&self, successor: &Rc<Label>) {
        self.successors.borrow_mut().push(Rc::clone(successor));
    }

    fn marksuccessors(&self) {
        for successor in self.successors.borrow().iter() {
            successor.ignore();
            successor.marksuccessors();
        }
    }

    fn updatedominance(labels: &mut Vec<Rc<Label>>, new_label: &Rc<Label>) -> bool {
        let mut i: usize = 0;
        while i < labels.len() {
            if labels[i].dominates(new_label) {
                return false;
            }
            if new_label.dominates(&labels[i]) {
                labels[i].marksuccessors();
                let last = labels.pop();
                if i < labels.len() {
                    labels[i] = last.unwrap();
                }
            } else {
                i += 1;
            }
        }
        // at this point the new label is not dominated so we add it
        labels.push(Rc::clone(new_label));
        true
    }
}

pub fn solve(d: &tsp::TSPData, nresources: usize, resourcecapacity: usize, maxlen: i32) -> f64 {
    // we will store all labels here
    let mut q: VecDeque<usize> = VecDeque::new();
    let mut in_q: Vec<bool> = vec![false; d.n];
    // initial label
    let l0 = Label::empty(d, nresources);
    q.push_back(0);
    in_q[0] = true;
    // considered labels at each node
    let mut labels: Vec<Vec<Rc<Label>>> = vec![Vec::new(); d.n];
    labels[0].push(l0);
    // main DP loop
    while !q.is_empty() {
        let n = q.pop_front().unwrap();
        in_q[n] = false;
        for i in 0..labels[n].len() {
            let lind = Rc::clone(&labels[n][i]);
            if lind.is_ignored() {
                continue;
            }
            for succ in 0..d.n {
                if lind.visited[succ] || succ == n {
                    continue;
                }
                // is the extension length-feasible?
                if lind.length + d.d(n, succ) + d.d(succ, 0) > maxlen {
                    continue;
                }
                // is it resource-feasible?
                let mut rfeas: bool = true;
                for r in 0..nresources {
                    if (succ & (1 << r)) > 0 && lind.q[r] + 1 > resourcecapacity {
                        rfeas = false;
                        break;
                    }
                }
                if !rfeas {
                    continue;
                }
                // at this point we know the extension is feasible
                let nl = Label::extend(d, &lind, succ);
                let added = Label::updatedominance(&mut labels[succ], &nl);
                if added {
                    lind.addsuccessor(&nl);
                    if !in_q[succ] && succ != 0 {
                        q.push_back(succ);
                        in_q[succ] = true;
                    }
                }
            }
            lind.ignore();
        }
    }
    let mut bestcost: f64 = labels[0][0].cost;
    for i in 1..labels[0].len() {
        let cost = labels[0][i].cost;
        if cost < bestcost {
            bestcost = cost;
        }
    }
    labels[0][0].predecessor.as_ref();
    bestcost
}
