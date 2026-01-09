use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap},
};

use anyhow::Result;
use util::{
    Solution,
    reader::{parse_grid, read_file},
};

/// Morton encoding and decoding for 3D coordinates
struct Morton;

impl Morton {
    /// Using magic bits to interleave the bits
    ///
    /// <https://www.forceflow.be/2013/10/07/morton-encodingdecoding-through-bit-interleaving-implementations#“Magic_Bits”_method>
    const fn interleave_bits_by_3(x: i64) -> u64 {
        let mut x = x.cast_unsigned() & 0x1ff_fff; // only look at the first 21 bits
        x = (x | (x << 32)) & 0x1f_000_000_00f_fff;
        x = (x | (x << 16)) & 0x1f_000_0ff_000_0ff;
        x = (x | (x << 8)) & 0x1_00f_00f_00f_00f_00f;
        x = (x | (x << 4)) & 0x1_0c3_0c3_0c3_0c3_0c3;
        x = (x | (x << 2)) & 0x1_249_249_249_249_249;
        x
    }

    /// Encode 3D coordinates into a single Morton code
    ///
    /// NOTE: only works for non-negative coordinates within 21 bits
    const fn encode(x: i64, y: i64, z: i64) -> u64 {
        (Self::interleave_bits_by_3(x) << 2)
            | (Self::interleave_bits_by_3(y) << 1)
            | Self::interleave_bits_by_3(z)
    }

    /// Using magic bits to deinterleave the bits
    ///
    /// <https://www.forceflow.be/2013/10/07/morton-encodingdecoding-through-bit-interleaving-implementations#“Magic_Bits”_method>
    const fn uninterleave_bits_by_3(x: u64) -> i64 {
        let mut x = (x & 0x1249_2492_4924_9249).cast_signed();
        x = (x | (x >> 2)) & 0x1_0c3_0c3_0c3_0c3_0c3;
        x = (x | (x >> 4)) & 0x1_00f_00f_00f_00f_00f;
        x = (x | (x >> 8)) & 0x1f_000_0ff_000_0ff;
        x = (x | (x >> 16)) & 0x1f_000_000_00f_fff;
        x = (x | (x >> 32)) & 0x1ff_fff;
        x
    }

    /// Decode a Morton code into 3D coordinates
    const fn decode(morton_code: u64) -> [i64; 3] {
        [
            Self::uninterleave_bits_by_3(morton_code >> 2),
            Self::uninterleave_bits_by_3(morton_code >> 1),
            Self::uninterleave_bits_by_3(morton_code),
        ]
    }
}

struct DisjointSet {
    /// Root of each element
    parent: Vec<usize>,
    /// Map from root to component size
    sizes: BTreeMap<usize, u64>,
}

impl DisjointSet {
    /// Initialize Disjoint Set with n disjoint sets
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            sizes: (0..size).map(|i| (i, 1)).collect::<BTreeMap<_, _>>(),
        }
    }

    /// Find the root of the set containing x with path compression
    fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut curr = x;
        let mut next = self.parent[curr];
        while next != root {
            next = self.parent[curr];
            self.parent[curr] = root;
            curr = next;
        }
        root
    }

    /// Union the sets containing x and y
    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x != root_y {
            // Set the parent of root_y to root_x
            self.parent[root_y] = root_x;
            // Then update sizes map
            let size_y = self.sizes.remove(&root_y).unwrap_or(1);
            self.sizes
                .entry(root_x)
                .and_modify(|s| *s += size_y)
                .or_insert(size_y);
        }
    }
}

/// Processed input node with Morton code and original index
struct Node {
    morton_code: u64,
    index: usize,
    coordinate: [i64; 3],
}

impl Node {
    const fn new(index: usize, coords: &[i64; 3]) -> Self {
        let morton_code = Morton::encode(coords[0], coords[1], coords[2]);
        Self {
            morton_code,
            index,
            coordinate: [coords[0], coords[1], coords[2]],
        }
    }
}

/// Linear octree node representation
#[derive(Clone, Copy)]
struct LinearOctreeNode {
    /// Start index in the nodes array
    start: usize,
    /// End index in the nodes array
    end: usize,
    /// Current depth in the octree
    depth: usize,
    /// Morton code prefix shared by the nodes in this octree node
    prefix: u64,
}

impl LinearOctreeNode {
    /// Get the loose bounding box defined by the Morton code prefix
    #[allow(dead_code)]
    #[deprecated(note = "Use the precomputed octree_bounding_boxes instead")]
    const fn bounding_box(&self) -> ([i64; 3], [i64; 3]) {
        let min = Morton::decode(self.prefix);
        let length = (1 << (22 - self.depth)) - 1;
        let max = [min[0] | length, min[1] | length, min[2] | length];
        (min, max)
    }

    /// Get the number of nodes in this octree node
    const fn size(&self) -> usize {
        self.end - self.start
    }

    /// Split this octree node into its children
    fn split(&self, nodes: &[Node]) -> Vec<Self> {
        let mut children = vec![];
        let mut curr_start = self.start;
        let last_octant = (nodes[self.end - 1].morton_code >> (3 * (20 - self.depth))) & 0b111;

        for octant in 0..last_octant {
            // If the octant has no nodes, skip it
            let curr_octant = (nodes[curr_start].morton_code >> (3 * (20 - self.depth))) & 0b111;
            if curr_octant > octant {
                continue;
            }

            // Binary search to find the end of this octant
            let mut low = curr_start;
            let mut high = self.end;
            while low < high {
                let mid = usize::midpoint(low, high);
                let mid_octant = (nodes[mid].morton_code >> (3 * (20 - self.depth))) & 0b111;
                if mid_octant <= octant {
                    low = mid + 1;
                } else {
                    high = mid;
                }
            }
            children.push(Self {
                start: curr_start,
                end: low,
                depth: self.depth + 1,
                prefix: self.prefix | (octant << (3 * (20 - self.depth))),
            });
            curr_start = low;
        }
        children.push(Self {
            start: curr_start,
            end: self.end,
            depth: self.depth + 1,
            prefix: self.prefix | (last_octant << (3 * (20 - self.depth))),
        });
        children
    }
}

struct Puzzle {
    /// Maximum number of steps to connect nodes (only for part 1)
    max_steps: usize,
    /// Array of nodes (with coordinates and other info)
    nodes: Vec<Node>,
    /// Linear octree representation
    octree: Vec<LinearOctreeNode>,
    /// Children indices for each octree node
    octree_children: Vec<Vec<usize>>,
    /// Tight bounding boxes for each octree node
    octree_bounding_boxes: Vec<([i64; 3], [i64; 3])>,
}

impl Puzzle {
    /// Parse the input and preprocess it
    ///
    /// This includes several steps:
    /// 1. First parse the input into a list of nodes with coordinates
    /// 2. Compute the Morton code for each node and sort by it
    /// 3. Build a linear octree from the sorted nodes
    /// 4. Precompute tight bounding boxes for each octree node
    ///
    /// In reality, if the size of the problem is not known or may be very
    /// large, step 3 and 4 should be omitted, instead we could use virtual tree
    /// traversal during the search and use a loose bounding box based on Morton
    /// code prefix.
    ///
    /// However, for this specific problem, the input size is known and
    /// manageable, so we can afford to build the full octree and precompute
    /// bounding boxes for efficiency.
    fn new(example: bool) -> Result<Self> {
        let content = read_file(Self::DAY, example)?.replace(',', " ");
        let nodes = parse_grid(content, str::parse)?;
        let mut nodes = nodes
            .outer_iter()
            .enumerate()
            .map(|(i, row)| Node::new(i, &[row[0], row[1], row[2]]))
            .collect::<Vec<_>>();
        nodes.sort_unstable_by_key(|n| n.morton_code);
        let max_steps = if example { 10 } else { 1000 };
        let mut queue = vec![(
            None,
            LinearOctreeNode {
                start: 0,
                end: nodes.len(),
                depth: 0,
                prefix: 0,
            },
        )];
        let mut octree = vec![];
        let mut octree_children = vec![];
        while let Some((parent, node)) = queue.pop() {
            let this_index = octree.len();
            octree.push(node);
            octree_children.push(vec![]);
            if let Some(p) = parent {
                let parent_children: &mut Vec<usize> = &mut octree_children[p];
                parent_children.push(this_index);
            }
            if node.size() > 1 {
                for child in node.split(&nodes).into_iter().rev() {
                    queue.push((Some(this_index), child));
                }
            }
        }
        let mut octree_bounding_boxes = vec![([0; 3], [0; 3]); octree.len()];
        for (idx, node) in octree.iter().enumerate().rev() {
            if node.size() == 1 {
                octree_bounding_boxes[idx] =
                    (nodes[node.start].coordinate, nodes[node.start].coordinate);
            } else {
                let mut min_bb = [i64::MAX; 3];
                let mut max_bb = [i64::MIN; 3];
                for &child_idx in &octree_children[idx] {
                    let (child_min, child_max) = octree_bounding_boxes[child_idx];
                    for dim in 0..3 {
                        min_bb[dim] = min_bb[dim].min(child_min[dim]);
                        max_bb[dim] = max_bb[dim].max(child_max[dim]);
                    }
                }
                octree_bounding_boxes[idx] = (min_bb, max_bb);
            }
        }
        Ok(Self {
            max_steps,
            nodes,
            octree,
            octree_children,
            octree_bounding_boxes,
        })
    }

    /// Helper function to compute squared Euclidean distance between nodes
    fn dist(a: &Node, b: &Node) -> i64 {
        (0..3)
            .map(|dim| (a.coordinate[dim] - b.coordinate[dim]).pow(2))
            .sum()
    }

    /// Helper function to deepen the search between two octree nodes
    fn deeper_search_nodes(
        &self,
        left_node_idx: usize,
        right_node_idx: usize,
    ) -> Vec<(usize, usize)> {
        if left_node_idx == right_node_idx {
            // Special case:
            // both octrees are the same, only need to consider pairs between each child
            let children = &self.octree_children[left_node_idx];
            return children
                .iter()
                .enumerate()
                .flat_map(|(idx, &i)| children[idx..].iter().map(move |&j| (i, j)))
                .collect();
        }

        // General case: split the larger octree and cross with the smaller one
        if self.octree[left_node_idx].size() > self.octree[right_node_idx].size() {
            self.octree_children[left_node_idx]
                .iter()
                .map(|&child| (child, right_node_idx))
                .collect()
        } else {
            self.octree_children[right_node_idx]
                .iter()
                .map(|&child| (left_node_idx, child))
                .collect()
        }
    }

    /// Helper function to precompute the components of each octree node
    fn compute_components(&self, dsu: &mut DisjointSet, components: &mut [BTreeSet<usize>]) {
        for idx in (0..self.octree.len()).rev() {
            let node = &self.octree[idx];
            components[idx].clear();
            if node.size() == 1 {
                components[idx].insert(dsu.find(self.nodes[node.start].index));
            } else {
                let mut comp = BTreeSet::new();
                for &child_idx in &self.octree_children[idx] {
                    comp.extend(components[child_idx].iter());
                }
                components[idx] = comp;
            }
        }
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 8;

    fn parse(example: bool) -> Self {
        Self::new(example).unwrap_or_else(|e| panic!("Failed to parse input: {e}"))
    }

    /// To find the k-smallest distances, we can use a max-heap of size k to
    /// keep track of the smallest distances found so far.
    ///
    /// The search is done by a dual traversal of the octrees, and is pruned if
    /// the minimum possible distance between two octrees is already larger than
    /// the largest distance in the heap. This should be very efficient as we
    /// can avoid unnecessary distance calculations by large margins.
    ///
    /// When the octrees are small enough, we can brute-force compute all
    /// pairwise distances between the nodes in the two octrees and update the
    /// heap accordingly.
    fn part1(&self) -> String {
        let mut min_dist = BinaryHeap::new();
        let mut search_stack = vec![(0, 0)];
        while let Some((left_node_idx, right_node_idx)) = search_stack.pop() {
            let left_node = self.octree[left_node_idx];
            let right_node = self.octree[right_node_idx];
            // Prune the search if the minimum possible distance is already larger than the
            // largest distance in the heap
            if let Some(&(delta, _, _)) = min_dist.peek() {
                // Compute minimum possible distance between two octrees
                let (left_min, left_max) = self.octree_bounding_boxes[left_node_idx];
                let (right_min, right_max) = self.octree_bounding_boxes[right_node_idx];
                let mut min_possible_dist = 0;
                for dim in 0..3 {
                    if left_max[dim] < right_min[dim] {
                        min_possible_dist += (right_min[dim] - left_max[dim]).pow(2);
                    } else if right_max[dim] < left_min[dim] {
                        min_possible_dist += (left_min[dim] - right_max[dim]).pow(2);
                    }
                }
                if min_possible_dist >= delta {
                    continue;
                }
            }

            // Brute force when small enough
            if left_node.size() * right_node.size() <= self.max_steps {
                let pairs = if left_node_idx == right_node_idx {
                    // Special case:
                    // both octrees are the same, only need to consider pairs (i, j) where i < j
                    (left_node.start..left_node.end)
                        .flat_map(|i| (i + 1..left_node.end).map(move |j| (i, j)))
                        .collect::<Vec<_>>()
                } else {
                    // General case: consider all pairs between left and right octrees
                    (left_node.start..left_node.end)
                        .flat_map(|i| (right_node.start..right_node.end).map(move |j| (i, j)))
                        .collect::<Vec<_>>()
                };
                for (i, j) in pairs {
                    let d = Self::dist(&self.nodes[i], &self.nodes[j]);
                    if d > min_dist.peek().map_or(i64::MAX, |&(d, _, _)| d) {
                        continue;
                    }
                    min_dist.push((d, i, j));
                    if min_dist.len() > self.max_steps {
                        min_dist.pop();
                    }
                }
                continue;
            }

            // Otherwise, we need to further split the octrees, prioritizing the larger one
            search_stack.extend(self.deeper_search_nodes(left_node_idx, right_node_idx));
        }

        // Finally, build the disjoint set from the minimum distances found
        let mut dsu = DisjointSet::new(self.nodes.len());
        for (_, i, j) in min_dist {
            dsu.union(i, j);
        }
        // Get the first three largest components
        dsu.sizes
            .values()
            .fold(BinaryHeap::new(), |mut heap, &size| {
                heap.push(Reverse(size));
                if heap.len() > 3 {
                    heap.pop();
                }
                heap
            })
            .iter()
            .map(|&Reverse(x)| x)
            .product::<u64>()
            .to_string()
    }

    /// This is essentially find the largest edge in the Minimum Spanning Tree
    /// (MST) of the complete graph between nodes. To build this tree as fast as
    /// possible, we can use Borůvka's algorithm with octree-based dual
    /// traversal to find the minimum outgoing edge for each component in each
    /// iteration.
    ///
    /// To start with, each node is its own component. In each iteration, we
    /// perform a dual traversal of the octrees to find the minimum outgoing
    /// edge for each component. We then add these edges to the MST and merge
    /// the components using a disjoint set. This process is repeated until all
    /// nodes are in a single component. The largest edge added during this
    /// process is then answer.
    fn part2(&self) -> String {
        let mut dsu = DisjointSet::new(self.nodes.len());
        let mut last_edge = (i64::MIN, 0, 0);
        // Precompute components for each octree node so that we don't have to repeatedly find the component of each node during the search
        let mut components = vec![BTreeSet::new(); self.octree.len()];
        while dsu.sizes.len() > 1 {
            let mut min_edges = dsu.sizes.keys().fold(HashMap::new(), |mut map, &comp| {
                map.insert(comp, (i64::MAX, 0, 0));
                map
            });
            self.compute_components(&mut dsu, &mut components);
            let mut search_stack = vec![(0, 0)];
            while let Some((left_node_idx, right_node_idx)) = search_stack.pop() {
                let left_node = self.octree[left_node_idx];
                let right_node = self.octree[right_node_idx];
                // Prune if both octrees belong to the same component
                if components[left_node_idx].len() == 1
                    && components[left_node_idx] == components[right_node_idx]
                {
                    continue;
                }

                // Prune the search if the minimum possible distance is already larger than the
                // largest distance
                let max_min_edge = components[left_node_idx]
                    .iter()
                    .chain(components[right_node_idx].iter())
                    .fold(i64::MIN, |max_edge, &comp| {
                        let Some((d, _, _)) = min_edges.get(&comp) else {
                            unreachable!("Every component should have an entry in min_edges")
                        };
                        max_edge.max(*d)
                    });
                if max_min_edge < i64::MAX {
                    // Compute minimum possible distance between two octrees
                    let (left_min, left_max) = self.octree_bounding_boxes[left_node_idx];
                    let (right_min, right_max) = self.octree_bounding_boxes[right_node_idx];
                    let mut min_possible_dist = 0;
                    for dim in 0..3 {
                        if left_max[dim] < right_min[dim] {
                            min_possible_dist += (right_min[dim] - left_max[dim]).pow(2);
                        } else if right_max[dim] < left_min[dim] {
                            min_possible_dist += (left_min[dim] - right_max[dim]).pow(2);
                        }
                    }
                    if min_possible_dist >= max_min_edge {
                        continue;
                    }
                }

                // Brute force when small enough
                if left_node.size() * right_node.size() <= 64 {
                    let pairs = if left_node_idx == right_node_idx {
                        // Special case:
                        // both octrees are the same, only need to consider pairs (i, j) where i < j
                        let nodes = (left_node.start..left_node.end)
                            .map(|i| (i, dsu.find(self.nodes[i].index)))
                            .collect::<Vec<_>>();
                        nodes
                            .iter()
                            .enumerate()
                            .flat_map(|(idx, &i)| {
                                nodes[idx + 1..]
                                    .iter()
                                    .filter_map(move |&j| (i.1 != j.1).then_some((i, j)))
                            })
                            .collect::<Vec<_>>()
                    } else {
                        // General case: consider all pairs between left and right octrees
                        let left_nodes = (left_node.start..left_node.end)
                            .map(|i| (i, dsu.find(self.nodes[i].index)))
                            .collect::<Vec<_>>();
                        let right_nodes = (right_node.start..right_node.end)
                            .map(|i| (i, dsu.find(self.nodes[i].index)))
                            .collect::<Vec<_>>();

                        left_nodes
                            .into_iter()
                            .flat_map(|i| {
                                right_nodes
                                    .iter()
                                    .filter_map(move |&j| (i.1 != j.1).then_some((i, j)))
                            })
                            .collect::<Vec<_>>()
                    };
                    for ((i, ci), (j, cj)) in pairs {
                        let d = Self::dist(&self.nodes[i], &self.nodes[j]);
                        min_edges.entry(ci).and_modify(|entry| {
                            if d < entry.0 {
                                *entry = (d, i, j);
                            }
                        });
                        min_edges.entry(cj).and_modify(|entry| {
                            if d < entry.0 {
                                *entry = (d, i, j);
                            }
                        });
                    }
                    continue;
                }

                // Otherwise, we need to further split the octrees, prioritizing the larger one
                search_stack.extend(self.deeper_search_nodes(left_node_idx, right_node_idx));
            }
            // Add the minimum edges found to the disjoint set
            for (_, (d, i, j)) in min_edges {
                dsu.union(self.nodes[i].index, self.nodes[j].index);
                if d > last_edge.0 {
                    last_edge = (d, i, j);
                }
            }
        }
        (self.nodes[last_edge.1].coordinate[0] * self.nodes[last_edge.2].coordinate[0]).to_string()
    }
}

fn main() -> Result<()> {
    let puzzle = Puzzle::new(false)?;
    println!("Day {} Part 1: {}", Puzzle::DAY, puzzle.part1());
    println!("Day {} Part 2: {}", Puzzle::DAY, puzzle.part2());

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use util::{Benchmark, Serializable};

    use super::*;

    #[test]
    fn test_part1() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part1(), "40");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "25272");
        Ok(())
    }

    #[test]
    fn benchmark() -> Result<()> {
        Puzzle::bench_all(Duration::from_secs(1)).to_csv(Puzzle::DAY)
    }
}
