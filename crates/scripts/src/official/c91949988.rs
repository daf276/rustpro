// Gaia Dragon, the Thunder Charger
use crate::prelude::*;

const ID: u32 = 91949988;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e1);
}

fn ovfilter(c: CardId, duel: &Duel, _: u8, _: CardId) -> bool {
    let rk = c.get_rank(duel);
    c.is_faceup(duel) && (rk == 5 || rk == 6)
}
