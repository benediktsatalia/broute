use std::{cell::RefCell, rc::Rc};

use crate::tsp;
use std::collections::VecDeque;

type LabelRef = Rc<RefCell<Label>>;

struct Label {
    at: usize,
    visited: Vec<bool>,
    ignore: bool,
    predecessor: Option<LabelRef>,
    cost: f64,
    length: i32,
    q: Vec<usize>,
    successors: Vec<LabelRef>,
}

impl Label {
    fn empty(d: &tsp::TSPData, nresources: usize) -> LabelRef {
        let visited: Vec<bool> = vec![false; d.n];
        let q: Vec<usize> = vec![0; nresources];
        Rc::new(RefCell::new(Label {
            at: 0,
            visited: visited,
            ignore: false,
            predecessor: None,
            cost: 0.0,
            length: 0,
            q: q,
            successors: Vec::new(),
        }))
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
    fn extend(d: &tsp::TSPData, from: &LabelRef, vertex: usize) -> LabelRef {
        let mut visited = from.borrow().visited.clone();
        visited[vertex] = true;
        let mut q = from.borrow().q.clone();
        for i in 0..q.len() {
            if (vertex & (1 << i)) > 0 {
                q[i] += 1;
            }
        }
        let cost = from.borrow().cost + d.aux(from.borrow().at, vertex);
        let length = from.borrow().length + d.d(from.borrow().at, vertex);
        Rc::new(RefCell::new(Label {
            at: vertex,
            visited: visited,
            ignore: false,
            predecessor: Some(from.clone()),
            cost: cost,
            length: length,
            q: q,
            successors: Vec::new(),
        }))
    }

    fn addsuccessor(&mut self, successor: &LabelRef) {
        self.successors.push(successor.clone());
    }

    fn marksuccessors(&self) {
        for successor in &self.successors {
            let mut successor = successor.borrow_mut();
            successor.ignore = true;
            successor.marksuccessors();
        }
    }

    fn updatedominance(labels: &mut Vec<LabelRef>, new_label: &LabelRef) -> bool {
        let mut i: usize = 0;
        while i < labels.len() {
            if labels[i].borrow().dominates(&new_label.borrow()) {
                return false;
            }
            if new_label.borrow().dominates(&labels[i].borrow()) {
                labels[i].borrow().marksuccessors();
                let last = labels.pop();
                if i < labels.len() {
                    labels[i] = last.unwrap();
                }
            } else {
                i += 1;
            }
        }
        // at this point the new label is not dominated so we add it
        labels.push(new_label.clone());
        return true;
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
    let mut labels: Vec<Vec<LabelRef>> = vec![Vec::new(); d.n];
    labels[0].push(l0);
    // main DP loop
    while !q.is_empty() {
        let n = q.pop_front().unwrap();
        in_q[n] = false;
        for i in 0..labels[n].len() {
            let lind = labels[n][i].clone();
            if lind.borrow().ignore == true {
                continue;
            }
            for succ in 0..d.n {
                if lind.borrow().visited[succ] || succ == n {
                    continue;
                }
                // is the extension length-feasible?
                if lind.borrow().length + d.d(n, succ) + d.d(succ, 0) > maxlen {
                    continue;
                }
                // is it resource-feasible?
                let mut rfeas: bool = true;
                for r in 0..nresources {
                    if (succ & (1 << r)) > 0 && lind.borrow().q[r] + 1 > resourcecapacity {
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
                    lind.borrow_mut().addsuccessor(&nl);
                    if !in_q[succ] && succ != 0 {
                        q.push_back(succ);
                        in_q[succ] = true;
                    }
                }
            }
            lind.borrow_mut().ignore = true;
        }
    }
    let mut bestcost: f64 = labels[0][0].borrow().cost;
    for i in 1..labels[0].len() {
        let cost = labels[0][i].borrow().cost;
        if cost < bestcost {
            bestcost = cost;
        }
    }
    return bestcost;
}
