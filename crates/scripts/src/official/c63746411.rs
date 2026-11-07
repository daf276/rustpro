// Number 106: Giant Hand
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 63746411;

// In the Extra Deck, Xyz Summoned with Mystical Elf and Flying Kamakiri #2, and the opponent's Effect Monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (3134241, MZONE),
    (79629370, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
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
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let triggering_location: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.rp != a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && triggering_location == LOCATION_MZONE
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_faceup(duel)
        && c.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_type(duel, TYPE_EFFECT)
        && !tc.is_immune_to_effect(duel, e)
    {
        c.set_card_target(duel, tc);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_condition(duel, rcon);
        tc.register_effect_full(duel, e1, true);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
        tc.register_effect_full(duel, e2, true);
    }
    Ok(())
}

fn rcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_owner(duel).is_has_card_target(duel, a.e.get_handler(duel))
}
