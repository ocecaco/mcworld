use crate::pos::WorldPos;

#[derive(Debug, Copy, Clone)]
enum NeighborState {
    XPlus,
    XMinus,
    YPlus,
    YMinus,
    ZPlus,
    ZMinus,
}

pub struct NeighborIterator {
    pos: WorldPos,
    state: Option<NeighborState>,
}

impl NeighborIterator {
    pub fn new(pos: WorldPos) -> Self {
        NeighborIterator {
            pos,
            state: Some(NeighborState::XPlus),
        }
    }
}

fn apply_state(pos: &WorldPos, state: NeighborState) -> WorldPos {
    match state {
        NeighborState::XPlus => WorldPos { x: pos.x + 1, y: pos.y, z: pos.z, dimension: pos.dimension },
        NeighborState::XMinus => WorldPos { x: pos.x - 1, y: pos.y, z: pos.z, dimension: pos.dimension },
        NeighborState::YPlus => WorldPos { x: pos.x, y: pos.y + 1, z: pos.z, dimension: pos.dimension },
        NeighborState::YMinus => WorldPos { x: pos.x, y: pos.y - 1, z: pos.z, dimension: pos.dimension },
        NeighborState::ZPlus => WorldPos { x: pos.x, y: pos.y, z: pos.z + 1, dimension: pos.dimension },
        NeighborState::ZMinus => WorldPos { x: pos.x, y: pos.y, z: pos.z - 1, dimension: pos.dimension },
    }
}

fn next_state(state: NeighborState) -> Option<NeighborState> {
    match state {
        NeighborState::XPlus => Some(NeighborState::XMinus),
        NeighborState::XMinus => Some(NeighborState::YPlus),
        NeighborState::YPlus => Some(NeighborState::YMinus),
        NeighborState::YMinus => Some(NeighborState::ZPlus),
        NeighborState::ZPlus => Some(NeighborState::ZMinus),
        NeighborState::ZMinus => None,
    }
}

impl Iterator for NeighborIterator {
    type Item = WorldPos;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(state) = self.state {
            let result = apply_state(&self.pos, state);
            self.state = next_state(state);
            Some(result)
        } else {
            None
        }
    }
}
