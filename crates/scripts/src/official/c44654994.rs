// Artmage Impasto -Recapture-
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 44654994;

pub const LISTED_SERIES: &[u16] = &[SET_ARTMAGE];
// Set with Artmage Diactorus on the field to banish, Mystical Elf and Stone Ogre Grotto for three Races, and Ash Blossom & Joyous
// Spring in the opponent's hand to negate Silvy of the White Forest's search, a monster effect to respond to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (27184601, MZONE),
    (15025844, MZONE),
    (15023985, MZONE),
    (14558127, HAND | OPPONENT),
    (98385955, HAND),
    (99289828, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e0.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e0.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ARTMAGE]));
        duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
    });
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && duel.is_chain_negatable(a.ev as u8)
}

fn banfilter(c: CardId, duel: &Duel) -> bool {
    c.is_fusion_monster(duel) && c.is_faceup(duel) && c.is_able_to_remove(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, re) = (a.tp, a.re);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(banfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    let Some(re) = re else { return Ok(false) };
    let rc = re.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn spelltrap_to_hand(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, ev) = (a.e, a.tp, a.ev as u8);
    let (Some(re), Some(eg)) = (a.re, a.eg) else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, banfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if aux::remove_until::<aux::DefaultFieldReturnOp>(duel, g, 0, REASON_EFFECT, PHASE_END, ID, e, tp)?.is_some()
        && duel.negate_activation(ev)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
        && duel.destroy(eg, REASON_EFFECT)? > 0
        && duel
            .get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, 0, None)
            .get_bin_class_count(duel, CardId::get_race)
            >= 3
        && duel.is_existing_matching_card(spelltrap_to_hand, tp, 0, LOCATION_ONFIELD, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        let rg = duel.get_matching_group(spelltrap_to_hand, tp, 0, LOCATION_ONFIELD, None);
        if !rg.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(rg, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
