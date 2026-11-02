#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TileId(pub(crate) u64);

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BehaviorState {
    pub(crate) pointer: (f32, f32),
    pub(crate) pointer_near: bool,
    pub(crate) hovered: Option<TileId>,
    pub(crate) hover_since: u32,
    pub(crate) tooltip: Option<TileId>,
    pub(crate) hide_armed: bool,
    pub(crate) autohide: bool,
    pub(crate) slide_target: f32,
    pub(crate) next_tile_id: u64,
}

impl BehaviorState {
    pub(crate) fn new(autohide: bool) -> Self {
        Self {
            pointer: (-9999.0, -9999.0),
            pointer_near: false,
            hovered: None,
            hover_since: 0,
            tooltip: None,
            hide_armed: false,
            autohide,
            slide_target: 0.0,
            next_tile_id: 0,
        }
    }

    pub(crate) fn allocate_tile_id(&mut self) -> TileId {
        let id = TileId(self.next_tile_id);
        self.next_tile_id = self.next_tile_id.wrapping_add(1);
        id
    }
}

pub(crate) fn tile_index_by_id<I>(ids: I, target: TileId) -> Option<usize>
where
    I: IntoIterator<Item = TileId>,
{
    ids.into_iter().position(|id| id == target)
}

pub(crate) fn move_item<T>(items: &mut Vec<T>, from: usize, to: usize) -> bool {
    if from >= items.len() || to >= items.len() || from == to { return false; }
    let item = items.remove(from);
    items.insert(to, item);
    true
}

#[cfg(test)]
mod tests {
    use super::{move_item, tile_index_by_id, TileId};

    #[test]
    fn resolves_a_tile_by_stable_id_after_vector_reordering() {
        let ids = [TileId(8), TileId(2), TileId(13)];
        assert_eq!(tile_index_by_id(ids.iter().copied(), TileId(13)), Some(2));
        assert_eq!(tile_index_by_id(ids.iter().copied(), TileId(5)), None);
    }

    #[test]
    fn moving_item_preserves_order_and_rejects_invalid_positions() {
        let mut items = vec!["a", "b", "c", "d"];
        assert!(move_item(&mut items, 1, 3));
        assert_eq!(items, ["a", "c", "d", "b"]);
        assert!(!move_item(&mut items, 3, 3));
        assert!(!move_item(&mut items, 4, 0));
        assert_eq!(items, ["a", "c", "d", "b"]);
    }
}
