// Number 103: Ragnazero
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 94380860;
pub const XYZ_NUMBER: u32 = 103;

// In the Extra Deck, Xyz Summoned with two Level 4 monsters, with the opponent's Mage Power in hand to change the ATK of their monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (3134241, MZONE),
    (79629370, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
    (83746708, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_position(duel, POS_FACEUP_ATTACK) && c.get_attack(duel) != c.get_base_attack(duel) && c.is_can_be_effect_target(duel, e)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, e));
    }
    if chk == 0 {
        let g = duel.get_matching_group(|c, duel| filter(c, duel, e), tp, 0, LOCATION_MZONE, None);
        if g.is_empty() || !duel.is_player_can_draw(tp, 1) {
            return Ok(false);
        }
        e.set_label_object_group(duel, Some(g));
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = e.get_label_object_group(duel).unwrap_or_default();
    let sg = g.select(duel, tp, 1, 1, None)?;
    duel.set_target_card(sg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && duel.destroy(tc, REASON_EFFECT)? != 0 {
        duel.draw(a.tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
