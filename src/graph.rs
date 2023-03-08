use std::fmt::Debug;

pub trait Graph: Debug {
    fn dot_count(&self) -> usize;

    fn for_each_edge(&self, cb: &mut dyn FnMut(usize, usize));

    fn add_edge(&mut self, from: usize, to: usize);
}

pub trait DrawableGraph : Graph {
    fn draw(&self, backend: crate::gui::DrawBackend) {
        let mut api = crate::gui::DrawingApi::default();
        let dot_coords: Vec<_> = (0..self.dot_count())
            .map(|_| api.draw_dot())
            .collect();

        self.for_each_edge(&mut |from, to| api.draw_edge(dot_coords[from], dot_coords[to]));
        api.draw_with(backend);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GraphBackend {
    EdgeList,
    Matrix,
}

impl clap::ValueEnum for GraphBackend {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::EdgeList, Self::Matrix]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(match self {
            GraphBackend::EdgeList => "edges",
            GraphBackend::Matrix => "matrix",
        }))
    }
}

pub fn with_dots_count(backend: GraphBackend, count: usize) -> Box<dyn DrawableGraph> {
    match backend {
        GraphBackend::EdgeList => Box::new(edge_list::EdgeListGraph::with_dots_count(count)),
        GraphBackend::Matrix => Box::new(matrix::MatrixGraph::with_dots_count(count)),
    }
}

mod edge_list {

    use eframe::epaint::ahash::HashSet;

    #[derive(Debug, Default)]
    pub struct EdgeListGraph {
        dots: HashSet<usize>,
        edges: Vec<(usize, usize)>,
    }

    impl super::Graph for EdgeListGraph {
        fn dot_count(&self) -> usize {
            self.dots.len()
        }

        fn for_each_edge(&self, cb: &mut dyn FnMut(usize, usize)) {
            self.edges.iter()
                .copied()
                .for_each(|(from, to)| cb(from, to))
        }

        fn add_edge(&mut self, from: usize, to: usize) {
            self.dots.insert(from);
            self.dots.insert(to);
            self.edges.push((from, to));
        }
    }

    impl super::DrawableGraph for EdgeListGraph {}

    impl EdgeListGraph {
        pub fn with_dots_count(count: usize) -> Self {
            Self {
                dots: (0..count).collect(),
                edges: vec![],
            }
        }
    }
}

mod matrix {

    #[derive(Debug, Default)]
    pub struct MatrixGraph {
        mtx: Vec<Vec<bool>>,
    }

    impl super::Graph for MatrixGraph {
        fn dot_count(&self) -> usize {
            self.mtx.len()
        }

        fn add_edge(&mut self, from: usize, to: usize) {
            let min_req = from.max(to);

            if min_req >= self.mtx.len() {
                self.mtx.resize_with(min_req + 1, || vec![false; min_req + 1]);
                for line in self.mtx.iter_mut() {
                    line.resize(min_req + 1, false)
                }
            }

            self.mtx[from][to] = true;
        }

        fn for_each_edge(&self, cb: &mut dyn FnMut(usize, usize)) {
            (0..self.mtx.len())
                .flat_map(|from| (from..self.mtx.len()).map(move |to| (from, to)))
                .filter(|(from, to)| self.mtx[*from][*to])
                .for_each(on_tied(cb))
        }
    }

    impl super::DrawableGraph for MatrixGraph {}

    impl MatrixGraph {
        pub fn with_dots_count(count: usize) -> Self {
            Self {
                mtx: vec![vec![false; count]; count],
            }
        }
    }

    fn on_tied<Fst, Sec, F: FnMut(Fst, Sec)>(mut f: F) -> impl FnMut((Fst, Sec)) {
        move |(fst, sec)| f(fst, sec)
    }
}
