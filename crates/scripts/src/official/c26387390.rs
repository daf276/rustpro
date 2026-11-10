// Junk Signal
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 26387390;

pub const LISTED_NAMES: &[u32] = &[CARD_JUNK_WARRIOR, CARD_STARDUST_DRAGON];
// Scrap Synchron, which mentions Junk Warrior, in the Deck to Special Summon by Tributing a monster; Scrap Warrior Synchro
// Summoned in the Extra Deck, with the opponent's Ash Blossom (a Tuner, chained to its effect) to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (16449363, DECK),
    (16449363, DECK),
    (16449363, MZONE),
    (26976414, MZONE),
    (26976414, MZONE),
    (42711820, EXTRA),
    (42711820, EXTRA),
    (14558127, HAND | OPPONENT),
    (14558127, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(16449363)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_code(duel, CARD_JUNK_WARRIOR)
        || c.is_code(duel, CARD_STARDUST_DRAGON)
        || c.lists_code(duel, CARD_JUNK_WARRIOR)
        || c.lists_code(duel, CARD_STARDUST_DRAGON))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spcostfilter(c, duel, e, tp);
    let b1 = duel.check_release_group_cost(tp, filter, (1, 1), false, None, None);
    let event = duel.check_event_full(EVENT_CHAINING);
    let b2 = event.is_some_and(|event| {
        let chain_count = (event.event_value as u8).wrapping_sub(1);
        let trig_player: u8 = duel.get_chain_info(chain_count, CHAININFO_TRIGGERING_PLAYER);
        let trig_type: u32 = duel.get_chain_info(chain_count, CHAININFO_TRIGGERING_TYPE);
        event.event_player == 1 - tp
            && duel.is_chain_disablable(event.event_value as u8)
            && trig_player == tp
            && trig_type & TYPE_SYNCHRO > 0
    });
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        let sg = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
        if let Some(sc) = sg.get_first() {
            duel.release(sc, REASON_COST)?;
            e.set_label_object_group(duel, Some(sc.into()));
            sc.create_effect_relation(duel, e);
        }
    } else if op == Some(2)
        && let Some(event) = event
    {
        e.set_labels(duel, &[2, i64::from(event.event_value)]);
    }
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    match e.get_label(duel) {
        1 => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            duel.set_operation_info(
                0,
                CATEGORY_SPECIAL_SUMMON,
                None,
                1,
                tp,
                (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
            );
        }
        2 => {
            e.set_category(duel, CATEGORY_DISABLE);
            duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let labels = e.get_labels(duel);
    let (op, event_value) = (labels.first().copied().unwrap_or(0), labels.get(1).copied().unwrap_or(0));
    if op == 1 {
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(());
        }
        let sc = e.get_label_object_group(duel).and_then(|g| g.get_first());
        let exc = sc.filter(|sc| sc.is_relate_to_effect(duel, e)).map(CardSet::from);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, exc)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    } else if op == 2 {
        duel.negate_effect(event_value as u8);
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_synchro_monster(duel)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    Ok(())
}
