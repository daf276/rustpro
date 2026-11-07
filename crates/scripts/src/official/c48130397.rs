// Super Polymerization
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 48130397;

struct Params;

impl FusionSummonParams for Params {
    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_grave(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let g = duel.get_matching_group(
            |c, duel| Fusion::is_monster_filter(c, duel, CardId::is_faceup),
            tp,
            0,
            LOCATION_ONFIELD,
            None,
        );
        (Some(g), None)
    }

    fn extratg(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
        if a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
            duel.set_chain_limit(|_, _, _, _, _| false, 0);
        }
        Ok(true)
    }
}

// Fallen of Albaz and the opponent's LIGHT Petit Angel, for Albion the Branded Dragon in the Elfnote Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (68468459, MZONE), (38142739, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, cost);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}
