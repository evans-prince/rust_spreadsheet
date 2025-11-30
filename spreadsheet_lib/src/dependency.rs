use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Dependency {
    Cell(usize, usize),
    Range(usize, usize, usize, usize), // r1, c1, r2, c2
}

#[derive(Debug, Clone)]
pub struct DependencyManager {
    cell_dependents: HashMap<(usize, usize), HashSet<(usize, usize)>>,
    range_dependents: HashMap<(usize, usize), Vec<(Dependency, (usize, usize))>>,
    cell_dependencies: HashMap<(usize, usize), Vec<Dependency>>,
}

impl DependencyManager {
    pub fn new() -> Self {
        Self {
            cell_dependents: HashMap::new(),
            range_dependents: HashMap::new(),
            cell_dependencies: HashMap::new(),
        }
    }

    pub fn clear_dependencies(&mut self, consumer: (usize, usize)) {
        if let Some(deps) = self.cell_dependencies.remove(&consumer) {
            for dep in deps {
                match dep {
                    Dependency::Cell(r, c) => {
                        if let Some(set) = self.cell_dependents.get_mut(&(r, c)) {
                            set.remove(&consumer);
                        }
                    }
                    Dependency::Range(r1, c1, r2, c2) => {
                        let start_region_r = r1 / 256;
                        let end_region_r = r2 / 256;
                        let start_region_c = c1 / 256;
                        let end_region_c = c2 / 256;
                        for rr in start_region_r..=end_region_r {
                            for rc in start_region_c..=end_region_c {
                                if let Some(list) = self.range_dependents.get_mut(&(rr, rc)) {
                                    list.retain(|(_, c)| *c != consumer);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn add_dependency(&mut self, consumer: (usize, usize), provider: Dependency) {
        self.cell_dependencies
            .entry(consumer)
            .or_default()
            .push(provider.clone());
        match provider {
            Dependency::Cell(r, c) => {
                self.cell_dependents
                    .entry((r, c))
                    .or_default()
                    .insert(consumer);
            }
            Dependency::Range(r1, c1, r2, c2) => {
                let start_region_r = r1 / 256;
                let end_region_r = r2 / 256;
                let start_region_c = c1 / 256;
                let end_region_c = c2 / 256;
                for rr in start_region_r..=end_region_r {
                    for rc in start_region_c..=end_region_c {
                        self.range_dependents
                            .entry((rr, rc))
                            .or_default()
                            .push((Dependency::Range(r1, c1, r2, c2), consumer));
                    }
                }
            }
        }
    }

    pub fn get_dependents(&self, provider_cell: (usize, usize)) -> HashSet<(usize, usize)> {
        let mut dependents = HashSet::new();
        if let Some(direct) = self.cell_dependents.get(&provider_cell) {
            for &d in direct {
                dependents.insert(d);
            }
        }
        let (r, c) = provider_cell;
        let region_coords = (r / 256, c / 256);
        if let Some(ranges) = self.range_dependents.get(&region_coords) {
            for (dep, consumer) in ranges {
                if let Dependency::Range(r1, c1, r2, c2) = dep {
                    if r >= *r1 && r <= *r2 && c >= *c1 && c <= *c2 {
                        dependents.insert(*consumer);
                    }
                }
            }
        }
        dependents
    }

    pub fn check_circular_reference(
        &self,
        consumer: (usize, usize),
        provider: (usize, usize),
    ) -> bool {
        if consumer == provider {
            return true;
        }

        // We want to know if 'consumer' affects 'provider'.
        // So we start traversing from 'consumer'.
        let mut visited = HashSet::new();
        let mut stack = vec![consumer]; // <--- CHANGED FROM provider TO consumer

        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }

            // If we reach the provider, it means Consumer -> ... -> Provider.
            // Since we are adding Provider -> Consumer, this would close the loop.
            if current == provider {
                return true;
            }

            for dep in self.get_dependents(current) {
                stack.push(dep);
            }
        }
        false
    }
}
