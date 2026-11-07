// Maple Maiden
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 73104892;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Ash Blossom & Joyous Spring and Mystical Elf; and on the field against two monsters
// of the opponent's side by side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (14558127, MZONE),
    (15025844, MZONE),
    (79629370, MZONE | OPPONENT),
    (79629370, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, atttg);
    e1.set_operation(duel, attop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, gytg);
    e2.set_operation(duel, gyop);
    c.register_effect(duel, e2);
}

fn atttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTRIBUTE);
    let rc = duel.announce_attribute(tp, 1, ATTRIBUTE_ALL)?;
    duel.set_target_param(rc);
    Ok(false)
}

fn attop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let rc: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
    e1.set_value(duel, rc as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    Ok(())
}

// The monsters and Spell & Trap Zone card beside the card in the given sequence of the player's zones.
fn group(duel: &Duel, seq: i32, tp: u8) -> CardSet {
    let mut g = CardSet::default();
    let mut optadd = |location: u32, seq: i32, player: u8| {
        if let Some(c) = u32::try_from(seq).ok().and_then(|seq| duel.get_field_card(player, location, seq)) {
            g.insert(c);
        }
    };
    if seq < 4 {
        optadd(LOCATION_MZONE, seq + 1, tp);
    }
    if seq > 0 {
        optadd(LOCATION_MZONE, seq - 1, tp);
    }
    if seq < 5 {
        optadd(LOCATION_SZONE, seq, tp);
        if seq == 1 {
            optadd(LOCATION_MZONE, 5, tp);
            optadd(LOCATION_MZONE, 6, 1 - tp);
        }
        if seq == 3 {
            optadd(LOCATION_MZONE, 6, tp);
            optadd(LOCATION_MZONE, 5, 1 - tp);
        }
    } else if seq == 5 {
        optadd(LOCATION_MZONE, 1, tp);
        optadd(LOCATION_MZONE, 3, 1 - tp);
    } else if seq == 6 {
        optadd(LOCATION_MZONE, 3, tp);
        optadd(LOCATION_MZONE, 1, 1 - tp);
    }
    g
}

fn gyfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    !group(duel, c.get_sequence(duel) as i32, 1 - tp).is_empty()
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MMZONE) && gyfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| gyfilter(c, duel, tp), tp, 0, LOCATION_MMZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| gyfilter(c, duel, tp), tp, 0, LOCATION_MMZONE, 1, 1, None)?;
    Ok(false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        let g = group(duel, tc.get_sequence(duel) as i32, 1 - tp);
        if !g.is_empty() {
            duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_TOGRAVE);
            let sg = g.select(duel, 1 - tp, 1, 1, None)?;
            duel.hint_selection(sg);
            duel.send_to_grave_full(sg, REASON_RULE, PLAYER_NONE, 1 - tp)?;
        }
    }
    Ok(())
}
