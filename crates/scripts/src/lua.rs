// What the Lua runtime decides that ports must reproduce.

// The order `pairs` visits integer keys set in this order on a new table (`{}`), as Lua 5.4's ltable.c places them in
// the hash part: a key takes its main position (key % (size - 1 | 1)); on a collision it takes the last free node below
// the previous one found, and moves the occupant there instead when the occupant is not in its own main position; a full
// table grows to the next power of two and reinserts its keys in node order. Card codes never go to the array part.
pub fn pairs(keys: impl IntoIterator<Item = u32>) -> Vec<u32> {
    let mut table = Table::default();
    for key in keys {
        if !table.nodes.iter().any(|node| node.key == Some(key)) {
            table.insert(key);
        }
    }
    table.nodes.iter().filter_map(|node| node.key).collect()
}

// No nodes is Lua's dummy node. Positions are always below the node count, so the scratch node is never reached.
#[derive(Default)]
struct Table {
    nodes: Vec<Node>,
    last_free: usize,
    scratch: Node,
}

// `next` is the offset to the next node of the same main position, 0 for none.
#[derive(Debug, Clone, Copy, Default)]
struct Node {
    key: Option<u32>,
    next: isize,
}

impl Table {
    fn main_position(&self, key: u32) -> usize {
        key as usize % ((self.nodes.len() - 1) | 1)
    }

    fn node(&mut self, index: usize) -> &mut Node {
        let Self { nodes, scratch, .. } = self;
        nodes.get_mut(index).unwrap_or(scratch)
    }

    fn insert(&mut self, key: u32) {
        if self.nodes.is_empty() {
            return self.rehash(key);
        }
        let mut position = self.main_position(key);
        if let Some(occupant) = self.node(position).key {
            let Some(free) = self.free_position() else {
                return self.rehash(key);
            };
            let other = self.main_position(occupant);
            if other != position {
                let mut previous = other;
                while previous.wrapping_add_signed(self.node(previous).next) != position {
                    previous = previous.wrapping_add_signed(self.node(previous).next);
                }
                self.node(previous).next = free as isize - previous as isize;
                let moved = *self.node(position);
                *self.node(free) = moved;
                if moved.next != 0 {
                    self.node(free).next += position as isize - free as isize;
                    self.node(position).next = 0;
                }
            } else {
                let next = self.node(position).next;
                if next != 0 {
                    self.node(free).next = position as isize + next - free as isize;
                }
                self.node(position).next = free as isize - position as isize;
                position = free;
            }
        }
        self.node(position).key = Some(key);
    }

    fn free_position(&mut self) -> Option<usize> {
        while self.last_free > 0 {
            self.last_free -= 1;
            if self.node(self.last_free).key.is_none() {
                return Some(self.last_free);
            }
        }
        None
    }

    fn rehash(&mut self, key: u32) {
        let keys: Vec<u32> = self.nodes.iter().filter_map(|node| node.key).collect();
        let size = (keys.len() + 1).next_power_of_two();
        *self = Self {
            nodes: vec![Node::default(); size],
            last_free: size,
            scratch: Node::default(),
        };
        keys.into_iter().chain([key]).for_each(|key| self.insert(key));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_visit_keys_in_lua_node_order() {
        // As the oracle's Lua visits them; checked there against 300 random sets of up to 59 codes.
        let keys = [46986420, 50546208, 50213848, 68078978, 78135071, 15259703];
        assert_eq!(pairs(keys), [78135071, 68078978, 15259703, 46986420, 50213848, 50546208]);
    }
}
