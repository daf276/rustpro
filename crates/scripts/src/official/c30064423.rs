// Topologic Blaster Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 30064423;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Sangans (Effect Monsters); Special Summons of the opponent's Sangans trigger the effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE,
    );
    e0.set_code(duel, EFFECT_FORCE_MZONE);
    e0.set_range(duel, LOCATION_MZONE);
    e0.set_target_range(duel, 1, 0);
    e0.set_value(
        duel,
        Value::Zones(|duel, e, _, _, _| !((e.get_handler(duel).get_linked_zone(duel) & ZONES_EMZ) as i32)),
    );
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_EMZONE);
    e1.set_condition(duel, effcon);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    if eg.is_contains(a.e.get_handler(duel)) {
        return false;
    }
    let linkg = duel.get_matching_group(CardId::is_link_monster, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    linkg.iter().any(|lc| aux::zptgroupcon(duel, eg, aux::TRUE, lc))
}

fn sptfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_deck(duel)
}

fn choices(duel: &Duel, tp: u8, exception: Option<CardId>) -> [bool; 3] {
    [
        !duel.has_flag_effect(tp, ID)
            && duel.is_existing_matching_card(
                CardId::is_able_to_deck,
                tp,
                LOCATION_MZONE,
                LOCATION_MZONE,
                1,
                exception.map(Into::into),
            ),
        !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_matching_card(sptfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None),
        !duel.has_flag_effect(tp, ID + 2) && duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_EXTRA, 1, None),
    ]
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let [b1, b2, b3] = choices(duel, tp, Some(c));
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    if !duel.has_flag_effect(tp, ID) {
        let g = duel.get_matching_group(CardId::is_able_to_deck, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
        duel.set_possible_operation_info(0, CATEGORY_TODECK, Some(g), g.len() as u8, tp, 0);
    }
    if !duel.has_flag_effect(tp, ID + 1) {
        let g = duel.get_matching_group(sptfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        duel.set_possible_operation_info(0, CATEGORY_TODECK, Some(g), g.len() as u8, tp, 0);
    }
    if !duel.has_flag_effect(tp, ID + 2) {
        let g = duel.get_matching_group(
            |c, duel| c.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT),
            tp,
            0,
            LOCATION_EXTRA,
            None,
        );
        duel.set_possible_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then_some(c);
    let [b1, b2, b3] = choices(duel, tp, exc);
    if !(b1 || b2 || b3) {
        return Ok(());
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        let g = duel.get_matching_group(CardId::is_able_to_deck, tp, LOCATION_MZONE, LOCATION_MZONE, exc.map(Into::into));
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    } else if op == Some(2) {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        let g = duel.get_matching_group(sptfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    } else if op == Some(3) {
        duel.register_flag_effect(tp, ID + 2, RESET_PHASE | PHASE_END, 0, 1);
        let g = duel.get_field_group(tp, 0, LOCATION_EXTRA);
        duel.confirm_cards(tp, g)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let sg = g.filter_select(duel, tp, CardId::is_able_to_remove, 1, 1, None)?;
        if !sg.is_empty() {
            duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
        }
        duel.shuffle_extra(1 - tp)?;
    }
    Ok(())
}
