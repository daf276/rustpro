// Cherubini, Ebon Angel of the Burning Abyss
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 58699500;

pub const LISTED_SERIES: &[u16] = &[SET_BURNING_ABYSS];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_level(duel, &[3])
    }
}

// In the Extra Deck, Link Summoned with Nekogal #1 and Burglar (Level 3), and on the field with Dark Hole and Raigeki in the opponent's
// hand to destroy it, Trap Hole and Mirror Force to send to the Graveyard instead and Level 3 monsters in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (1761063, MZONE),
    (6297941, MZONE),
    (1761063, DECK),
    (6297941, DECK),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (4206964, SZONE),
    (44095762, SZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Replaced, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| e.get_handler(duel).get_linked_group(duel).is_contains(c));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_target(duel, desreptg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_label(duel, 0);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, |duel, a, chk| {
        a.e.set_label(duel, 1);
        Ok(chk == 0)
    });
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    !c.is_immune_to_effect(duel, e) && !c.is_status(duel, STATUS_DESTROY_CONFIRMED)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && (c.is_reason(duel, REASON_BATTLE) || rp != tp)
            && duel.is_existing_matching_card(|c, duel| desfilter(c, duel, e), tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| desfilter(c, duel, e), tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    duel.send_to_grave(g, REASON_EFFECT | REASON_REPLACE)?;
    Ok(true)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[3]) && (c.get_base_attack(duel) > 0 || c.get_base_defense(duel) > 0) && c.is_able_to_grave_as_cost(duel)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BURNING_ABYSS])
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        if e.get_label(duel) != 1 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_DECK, 0, 1, None)
            && duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    e.set_label(duel, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, cfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    e.set_label_object_group(duel, Some(g));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let (Some(tc), Some(sc)) = (duel.get_first_target(), e.get_label_object_group(duel).and_then(|g| g.get_first())) else {
        return Ok(());
    };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, sc.get_attack(duel));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        e2.set_value(duel, sc.get_defense(duel));
        tc.register_effect(duel, e2);
    }
    Ok(())
}
