// X-Krawler Neurogos
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 66393507;

pub const LISTED_SERIES: &[u16] = &[SET_KRAWLER];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, st: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_INSECT, Some(lc), st, tp)
    }
}

// Link Summoned from Krawler Spine and Axon (Insects), destroyed by Raigeki, with Krawlers of different names in the Graveyard to
// summon and monsters of the opponent's to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (88316955, MZONE),
    (88316955, MZONE),
    (88316955, MZONE),
    (15710054, MZONE),
    (15710054, MZONE),
    (88316955, GRAVE),
    (15710054, GRAVE),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1a.set_target_card(duel, |duel, e, c| {
        c.is_set_card(duel, &[SET_KRAWLER]) && e.get_handler(duel).get_linked_group(duel).is_contains(c)
    });
    e1a.set_value(duel, 1);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1b.set_value(duel, 300);
    c.register_effect(duel, e1b);
    let e1c = e1b.clone_effect(duel);
    e1c.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1c);
    let e1d = e1a.clone_effect(duel);
    e1d.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1d.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1d.set_target_card(duel, doubledamtg);
    e1d.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
    c.register_effect(duel, e1d);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn doubledamtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_KRAWLER])
        && c.get_battle_target(duel)
            .is_some_and(|bc| bc.is_controler(duel, 1 - e.get_handler_player(duel)))
        && e.get_handler(duel).get_linked_group(duel).is_contains(c)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    (c.is_reason(duel, REASON_BATTLE) || (a.rp == 1 - tp && c.is_reason(duel, REASON_EFFECT) && c.is_previous_controler(duel, tp)))
        && c.is_previous_position(duel, POS_FACEUP)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_KRAWLER]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 2);
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(tg), 2, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() || (tg.len() > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()) {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft > 0 && (ft as usize) < tg.len() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        tg = tg.select(duel, tp, ft as u8, ft as u8, None)?;
    }
    if !tg.is_empty() && duel.special_summon(tg, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)? > 0 {
        duel.confirm_cards(1 - tp, tg)?;
    }
    Ok(())
}
