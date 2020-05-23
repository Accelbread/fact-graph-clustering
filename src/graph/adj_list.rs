use std::collections::HashMap;

#[derive(Default)]
pub struct AdjList {
    data: HashMap<String, HashMap<String, f32>>,
}

struct Vertex {
    label: String
}

struct Edge {
    weight: f32
}

impl AdjList {
    pub fn new() -> Self {
        AdjList {
            data: HashMap::new(),
        }
    }

    pub fn add_vertex(&mut self, identifier: &str) -> bool {
        if self.data.contains_key(identifier) {
            false
        } else {
            self.data.insert(identifier.to_string(), HashMap::new());
            true
        }
    }

    pub fn contains_vertex(&self, identifier: &str) -> bool {
        self.data.contains_key(identifier)
    }

    fn write_edge(&mut self, vertex_1: &str, vertex_2: &str, weight: f32) {
        self.add_vertex(vertex_1);
        self.add_vertex(vertex_2);
        self.data
            .get_mut(vertex_1)
            .unwrap()
            .insert(vertex_2.to_string(), weight);
        self.data
            .get_mut(vertex_2)
            .unwrap()
            .insert(vertex_1.to_string(), weight);
    }

    pub fn apply_edge<F>(&mut self, vertex_1: &str, vertex_2: &str, f: F)
    where
        F: Fn(Option<f32>) -> f32,
    {
        let val = match self.data.get(vertex_1) {
            Some(v1) => match v1.get(vertex_2) {
                Some(w) => Some(*w),
                None => None,
            },
            None => None,
        };
        self.write_edge(vertex_1, vertex_2, f(val));
    }
}
