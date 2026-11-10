// Akashic Magician
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 28776350;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
    }

    // The Lua takes (g, lc, tp) but is handed the summon type where the player goes, and passes it on as the player.
    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, _: u8) -> bool {
        aux::check_same_property(duel, sg, &|c, duel| {
            c.get_race_full(duel, Some(lc), SUMMON_TYPE_LINK.into(), sumtype as u8)
        })
    }
}

// Copies on the field to link each other, Link Summoned from the Extra Deck with two Sangans (Fiend).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, linksummoned);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, linksummoned);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DECKDES);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, actg);
    e3.set_operation(duel, acop);
    c.register_effect(duel, e3);
}

fn linksummoned(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_link_summoned(duel)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        c.is_code(duel, ID) && sumtype & SUMMON_TYPE_LINK == SUMMON_TYPE_LINK
    });
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn linked_hand_group(duel: &Duel, a: &Args) -> CardSet {
    a.e.get_handler(duel)
        .get_linked_group(duel)
        .filter(duel, CardId::is_able_to_hand, None)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let lg = linked_hand_group(duel, a);
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(lg), lg.len() as u8, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let lg = linked_hand_group(duel, a);
    duel.send_to_hand(lg, PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}

fn mutual_link_total(duel: &Duel, c: CardId, only_monsters: bool) -> usize {
    let lg = c.get_mutual_linked_group(duel);
    let lg = if only_monsters {
        lg.filter(duel, CardId::is_monster, None)
    } else {
        lg
    };
    lg.get_sum(duel, |c, duel| i64::from(c.get_link(duel))) as usize
}

fn actg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let ct = mutual_link_total(duel, a.e.get_handler(duel), true);
        if ct == 0 || !duel.is_player_can_discard_deck(tp, ct) {
            return Ok(false);
        }
        return Ok(!duel
            .get_decktop_group(tp, ct)
            .filter(duel, CardId::is_able_to_hand, None)
            .is_empty());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CODE);
    let ac = duel.announce_card(tp, &[u64::from(TYPE_EXTRA), OPCODE_ISTYPE, OPCODE_NOT])?;
    duel.set_target_param(ac);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD_FILTER as i32);
    Ok(false)
}

fn acop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = mutual_link_total(duel, a.e.get_handler(duel), false);
    if ct == 0 || !duel.is_player_can_discard_deck(tp, ct) {
        return Ok(());
    }
    duel.confirm_decktop(tp, ct)?;
    let mut g = duel.get_decktop_group(tp, ct);
    let ac: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let hg = g.filter(duel, |c, duel| c.is_code(duel, ac) && c.is_able_to_hand(duel), None);
    g = g.difference(hg);
    if !hg.is_empty() {
        duel.disable_shuffle_check(true);
        duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, hg)?;
        duel.shuffle_hand(tp)?;
    }
    if !g.is_empty() {
        duel.disable_shuffle_check(true);
        duel.send_to_grave(g, REASON_EFFECT | REASON_EXCAVATE)?;
    }
    Ok(())
}
