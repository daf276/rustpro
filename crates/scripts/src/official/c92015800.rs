// Number 76: Harmonizer Gradielle
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 92015800;

// In the Extra Deck, Xyz Summoned with Gaia The Fierce Knight, Seiyaryu and Ryu-Ran, and monsters in the opponent's Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (6368038, MZONE),
    (6740720, MZONE),
    (2964201, MZONE),
    (15025844, GRAVE | OPPONENT),
    (3134241, GRAVE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_ADD_ATTRIBUTE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| {
            e.get_handler(duel)
                .get_overlay_group(duel)
                .get_bitwise_or(duel, |c, duel| u64::from(c.get_attribute(duel))) as i32
        }),
    );
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(c.is_attribute(duel, e.get_handler(duel).get_attribute(duel)))),
    );
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(duel, Value::EffectPlayer(indval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, mttg);
    e4.set_operation(duel, mtop);
    c.register_effect(duel, e4);
}

fn indval(duel: &Duel, e: EffectId, re: EffectId, rp: u8) -> bool {
    re.is_activated(duel)
        && re.is_monster_effect(duel)
        && rp == 1 - e.get_handler_player(duel)
        && re.is_card_attribute(duel, e.get_handler(duel).get_attribute(duel))
}

fn mttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && chkc.is_monster(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_monster, tp, 0, LOCATION_GRAVE, 1, None)
            && a.e.get_handler(duel).check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    let g = duel.select_target(tp, CardId::is_monster, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, e)
        && c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? > 0
        && let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e))
    {
        duel.overlay(c, tc)?;
    }
    Ok(())
}
