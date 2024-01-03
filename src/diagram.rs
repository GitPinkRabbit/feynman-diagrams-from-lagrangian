use std::fmt::Display;

use crate::{field::*, lagrangian::*};

#[derive(Clone, Copy, Debug)]
pub enum Inout {
    In,
    Out,
    Unknown,
}

pub use Inout::*;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Unique {
    Nah,
    Yeah,
    Used,
}

use Unique::*;

#[derive(Clone, Debug)]
enum VertexKind {
    External { field: Field, inout: Inout },
    Internal { interaction: Interaction },
}

use VertexKind::*;

#[derive(Clone, Debug)]
pub struct Vertex {
    kind: VertexKind,
    adj: Vec<Option<(usize, usize)>>,
    uniq: Unique,
}

impl Vertex {
    pub fn external(field: Field) -> Self {
        Self {
            kind: External {
                field,
                inout: Unknown,
            },
            adj: vec![None; 1],
            uniq: Yeah,
        }
    }

    pub fn inout(field: Field, inout: Inout) -> Self {
        Self {
            kind: External { field, inout },
            adj: vec![None; 1],
            uniq: Yeah,
        }
    }

    pub fn internal(interaction: Interaction) -> Self {
        let len = interaction.factors().len();
        Self {
            kind: Internal { interaction },
            adj: vec![None; len],
            uniq: Nah,
        }
    }

    fn sign(&self) -> Vec<Field> {
        match &self.kind {
            External { field, inout: _ } => vec![field.clone()],
            Internal { interaction } => interaction.factors().to_vec(),
        }
    }

    fn left(&self) -> usize {
        self.adj.iter().map(|&x| x.map_or(1, |_| 0)).sum()
    }

    fn enumerate_zipped(&self) -> Vec<(usize, Field, Option<(usize, usize)>)> {
        self.sign()
            .into_iter()
            .zip(self.adj.clone())
            .enumerate()
            .map(|(x, (y, z))| (x, y, z))
            .collect()
    }

    fn ports(&self, f: &Field) -> Vec<usize> {
        self.enumerate_zipped()
            .into_iter()
            .filter(|t| &t.1 == f && t.2.is_none())
            .map(|t| t.0)
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Diagram {
    vertices: Vec<Vertex>,
    left: usize,
}

impl Diagram {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        let left = vertices.iter().map(|x| x.left()).sum();
        Self { vertices, left }
    }

    pub fn is_connected(&self) -> bool {
        if self.left != 0 {
            for i in 0..self.vertices.len() {
                let mut vis = vec![false; self.vertices.len()];
                let mut stack = vec![];
                vis[i] = true;
                stack.push(i);
                let mut flag = false;
                let mut cnt = 0;
                while let Some(u) = stack.pop() {
                    let u = &self.vertices[u];
                    cnt += 1;
                    for v in &u.adj {
                        if let Some((v, _)) = v.clone() {
                            if !vis[v] {
                                vis[v] = true;
                                stack.push(v);
                            }
                        } else {
                            flag = true;
                        }
                    }
                }
                if cnt < self.vertices.len() && !flag {
                    return false;
                }
            }
            return true;
        }
        let mut vis = vec![false; self.vertices.len()];
        let mut stack = vec![];
        vis[0] = true;
        stack.push(0usize);
        let mut cnt = 0;
        while let Some(u) = stack.pop() {
            let u = &self.vertices[u];
            cnt += 1;
            for v in &u.adj {
                let v = v.unwrap().0;
                if !vis[v] {
                    vis[v] = true;
                    stack.push(v);
                }
            }
        }
        cnt == self.vertices.len()
    }

    pub fn draw(self) -> Vec<Self> {
        if self.left % 2 != 0 || !self.is_connected() {
            return vec![];
        }
        if self.left == 0 {
            return vec![self];
        }
        let vs = &self.vertices;
        let mut candidates: Vec<Self> = vec![];
        if let Some(i) = vs.iter().position(|x| x.uniq == Yeah) {
            let u = &vs[i];
            if u.left() == 0 {
                let mut now = self.clone();
                now.vertices[i].uniq = Used;
                candidates.push(now);
            } else {
                let zipped = u.enumerate_zipped();
                let zipped = zipped.into_iter().filter(|t| t.2.is_none());
                let f: Field = zipped.clone().next().unwrap().1.clone();
                let indices: Vec<usize> = zipped.filter(|t| t.1 == f).map(|t| t.0).collect();
                let g = f.anti();
                match indices.len() {
                    1 => {
                        let ki = indices[0];
                        for (j, v) in vs.iter().enumerate() {
                            if f == g && i == j {
                                continue;
                            }
                            let kjs = v.ports(&g);
                            if kjs.is_empty() {
                                continue;
                            }
                            let kj = kjs[0];
                            let mut now = self.clone();
                            now.vertices[i].adj[ki] = Some((j, kj));
                            now.vertices[j].adj[kj] = Some((i, ki));
                            now.vertices[j].uniq = Yeah;
                            now.left -= 2;
                            candidates.push(now);
                            if v.uniq == Nah {
                                break;
                            }
                        }
                    }
                    2 => {
                        let (ki0, ki1) = (indices[0], indices[1]);
                        for (j, v) in vs.iter().enumerate() {
                            if f == g && i == j {
                                // self loop
                                let mut now = self.clone();
                                now.vertices[i].adj[ki0] = Some((i, ki1));
                                now.vertices[i].adj[ki1] = Some((i, ki0));
                                now.left -= 2;
                                candidates.push(now);
                                continue;
                            }
                            let kjs = v.ports(&g);
                            if kjs.len() <= 1 {
                                continue;
                            }
                            let (kj0, kj1) = (kjs[0], kjs[1]);
                            let mut now = self.clone();
                            now.vertices[i].adj[ki0] = Some((j, kj0));
                            now.vertices[i].adj[ki1] = Some((j, kj1));
                            now.vertices[j].adj[kj0] = Some((i, ki0));
                            now.vertices[j].adj[kj1] = Some((i, ki1));
                            now.vertices[j].uniq = Yeah;
                            now.left -= 4;
                            candidates.push(now);
                            if v.uniq == Nah {
                                break;
                            }
                        }
                        for (j1, v1) in vs.iter().enumerate() {
                            if f == g && i == j1 {
                                continue;
                            }
                            let kj1s = v1.ports(&g);
                            if kj1s.is_empty() {
                                continue;
                            }
                            let kj1 = kj1s[0];
                            for (j0, v0) in vs.iter().enumerate().take(j1) {
                                if f == g && i == j0 {
                                    continue;
                                }
                                let kj0s = v0.ports(&g);
                                if kj0s.is_empty() {
                                    continue;
                                }
                                let kj0 = kj0s[0];
                                let mut now = self.clone();
                                now.vertices[i].adj[ki0] = Some((j0, kj0));
                                now.vertices[i].adj[ki1] = Some((j1, kj1));
                                now.vertices[j0].adj[kj0] = Some((i, ki0));
                                now.vertices[j1].adj[kj1] = Some((i, ki1));
                                now.left -= 4;
                                candidates.push(now);
                            }
                        }
                    }
                    3 => {
                        let (ki0, ki1, ki2) = (indices[0], indices[1], indices[2]);
                        for (j, v) in vs.iter().enumerate() {
                            if f == g && i == j {
                                continue;
                            }
                            let kjs = v.ports(&g);
                            if f == g && !kjs.is_empty() {
                                // self loop and a line
                                let kj = kjs[0];
                                let mut now = self.clone();
                                now.vertices[i].adj[ki0] = Some((i, ki1));
                                now.vertices[i].adj[ki1] = Some((i, ki0));
                                now.vertices[i].adj[ki2] = Some((j, kj));
                                now.vertices[j].adj[kj] = Some((i, ki2));
                                now.vertices[j].uniq = Yeah;
                                now.left -= 4;
                                candidates.push(now);
                            }
                            if kjs.len() <= 2 {
                                continue;
                            }
                            let (kj0, kj1, kj2) = (kjs[0], kjs[1], kjs[2]);
                            let mut now = self.clone();
                            now.vertices[i].adj[ki0] = Some((j, kj0));
                            now.vertices[i].adj[ki1] = Some((j, kj1));
                            now.vertices[i].adj[ki2] = Some((j, kj2));
                            now.vertices[j].adj[kj0] = Some((i, ki0));
                            now.vertices[j].adj[kj1] = Some((i, ki1));
                            now.vertices[j].adj[kj2] = Some((i, ki2));
                            now.vertices[j].uniq = Yeah;
                            now.left -= 6;
                            candidates.push(now);
                            if v.uniq == Nah {
                                break;
                            }
                        }
                        for (j1, v1) in vs.iter().enumerate() {
                            if f == g && i == j1 {
                                continue;
                            }
                            let kj1s = v1.ports(&g);
                            if kj1s.len() <= 1 {
                                continue;
                            }
                            let (kj10, kj11) = (kj1s[0], kj1s[1]);
                            for (j0, v0) in vs.iter().enumerate().take(j1) {
                                if f == g && i == j0 {
                                    continue;
                                }
                                let kj0s = v0.ports(&g);
                                if kj0s.is_empty() {
                                    continue;
                                }
                                let kj0 = kj0s[0];
                                let mut now = self.clone();
                                now.vertices[i].adj[ki0] = Some((j0, kj0));
                                now.vertices[i].adj[ki1] = Some((j1, kj10));
                                now.vertices[i].adj[ki2] = Some((j1, kj11));
                                now.vertices[j0].adj[kj0] = Some((i, ki0));
                                now.vertices[j1].adj[kj10] = Some((i, ki1));
                                now.vertices[j1].adj[kj11] = Some((i, ki2));
                                now.vertices[j0].uniq = Yeah;
                                now.vertices[j1].uniq = Yeah;
                                now.left -= 6;
                                candidates.push(now);
                            }
                        }
                        for (j2, v2) in vs.iter().enumerate() {
                            if f == g && i == j2 {
                                continue;
                            }
                            let kj2s = v2.ports(&g);
                            if kj2s.is_empty() {
                                continue;
                            }
                            let kj2 = kj2s[0];
                            for (j1, v1) in vs.iter().enumerate().take(j2) {
                                if f == g && i == j1 {
                                    continue;
                                }
                                let kj1s = v1.ports(&g);
                                if kj1s.is_empty() {
                                    continue;
                                }
                                let kj1 = kj1s[0];
                                for (j0, v0) in vs.iter().enumerate().take(j1) {
                                    if f == g && i == j0 {
                                        continue;
                                    }
                                    let kj0s = v0.ports(&g);
                                    if kj0s.is_empty() {
                                        continue;
                                    }
                                    let kj0 = kj0s[0];
                                    let mut now = self.clone();
                                    now.vertices[i].adj[ki0] = Some((j0, kj0));
                                    now.vertices[i].adj[ki1] = Some((j1, kj1));
                                    now.vertices[i].adj[ki2] = Some((j2, kj2));
                                    now.vertices[j0].adj[kj0] = Some((i, ki0));
                                    now.vertices[j1].adj[kj1] = Some((i, ki1));
                                    now.vertices[j2].adj[kj2] = Some((i, ki2));
                                    now.left -= 6;
                                    candidates.push(now);
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        } else {
            let i = self.vertices.iter().position(|x| x.left() != 0).unwrap();
            let u = &vs[i];
            let ki = u.adj.iter().position(|x| x.is_none()).unwrap();
            let f = u.sign()[ki].clone();
            let g = f.anti();
            for (j, v) in vs.iter().enumerate() {
                let kjs = v.ports(&g);
                if kjs.is_empty() {
                    continue;
                }
                if f == g && i == j {
                    if kjs.len() >= 2 {
                        // self loop
                        let kj = kjs[1];
                        let mut now = self.clone();
                        now.vertices[i].adj[ki] = Some((j, kj));
                        now.vertices[j].adj[kj] = Some((i, ki));
                        now.left -= 2;
                        candidates.push(now);
                    }
                    continue;
                }
                let kj = kjs[0];
                let mut now = self.clone();
                now.vertices[i].adj[ki] = Some((j, kj));
                now.vertices[j].adj[kj] = Some((i, ki));
                now.left -= 2;
                candidates.push(now);
            }
        }
        candidates.into_iter().flat_map(|d| d.draw()).collect()
    }
}

impl Display for Diagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, u) in self.vertices.iter().enumerate() {
            for (ki, (j, kj)) in u.adj.iter().enumerate().map(|t| (t.0, t.1.unwrap())) {
                if (i, ki) < (j, kj) {
                    let t = &u.sign()[ki];
                    let mut text: String = match t.kind() {
                        RealScalar => "scalar",
                        ComplexScalar(x) => if x { "charged scalar" } else { "anti charged scalar" },
                        RealVector => "boson",
                        ComplexVector(x) => if x { "charged boson" } else { "anti charged boson" },
                        Spinor(x) => if x { "anti fermion" } else { "fermion" },
                    }.to_string();
                    if i == j {
                        text += ", min distance=2.5cm";
                    }
                    write!(f, "\t{0} -- [{2}] {1},\n", i, j, text)?;
                }
            }
        }
        Ok(())
    }
}
