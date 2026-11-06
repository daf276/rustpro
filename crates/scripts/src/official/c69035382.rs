// Contract with the Abyss
use crate::prelude::*;

struct Params;

impl RitualParams for Params {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_attribute(duel, ATTRIBUTE_DARK)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_equal::<Params>(duel, c);
}
