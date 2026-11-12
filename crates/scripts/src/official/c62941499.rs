// Springans Ship - Exblowrer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 62941499;

// In the Extra Deck, Xyz Summoned from Level 8 Normal Monsters (Tri-Horned Dragon, Gogiga Gagagigo), with cards of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (39111158, MZONE),
    (39111158, MZONE),
    (39674352, MZONE),
    (39674352, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, SZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, bancon);
    e2.set_target(duel, bantg);
    e2.set_operation(duel, banop);
    c.register_effect(duel, e2);
}

// Lua shifts by a negative amount the other way.
fn shift_left(value: u32, amount: i32) -> u32 {
    if amount >= 0 { value << amount } else { value >> -amount }
}

fn adjzone(loc: u32, seq: u32) -> u32 {
    let seq = seq as i32;
    if loc == LOCATION_MZONE {
        if seq < 5 {
            (shift_left(7, seq - 1) & ZONES_MMZ) | (1 << (seq + 8))
        } else {
            (1 << seq) | (2 + 6 * (seq - 5)) as u32
        }
    } else {
        (shift_left(7, seq + 7) & 0x1F00) | (1 << seq)
    }
}

fn groupfrombit(duel: &Duel, bit: u32, p: u8) -> CardSet {
    let loc = if bit & 0x7F > 0 { LOCATION_MZONE } else { LOCATION_SZONE };
    let seq = if loc == LOCATION_MZONE { bit } else { bit >> 8 }.trailing_zeros();
    let mut g = CardSet::default();
    let mut optadd = |loc: u32, seq: u32| {
        if let Some(c) = duel.get_field_card(p, loc, seq) {
            g.insert(c);
        }
    };
    optadd(loc, seq);
    if seq <= 4 {
        if seq < 4 {
            optadd(loc, seq + 1);
        }
        if seq >= 1 {
            optadd(loc, seq - 1);
        }
    }
    if loc == LOCATION_MZONE {
        if seq < 5 {
            optadd(LOCATION_SZONE, seq);
            if seq == 1 {
                optadd(LOCATION_MZONE, 5);
            }
            if seq == 3 {
                optadd(LOCATION_MZONE, 6);
            }
        } else if seq == 5 {
            optadd(LOCATION_MZONE, 1);
        } else if seq == 6 {
            optadd(LOCATION_MZONE, 3);
        }
    } else {
        optadd(LOCATION_MZONE, seq);
    }
    g
}

fn filter(c: CardId, duel: &Duel) -> bool {
    !c.is_location(duel, LOCATION_FZONE) && !(duel.is_duel_type(DUEL_SEPARATE_PZONE) && c.is_location(duel, LOCATION_PZONE))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(filter, tp, 0, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(e.get_handler(duel).get_overlay_count(duel) > 0 && !g.is_empty());
    }
    let zones = g
        .into_iter()
        .fold(0, |zones, oc| zones | adjzone(oc.get_location(duel), oc.get_sequence(duel)));
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
    let zone = duel.select_field_zone_full(tp, 1, 0, LOCATION_ONFIELD as u16, !zones << 16)?;
    unported!(duel, zone.is_none());
    let zone = zone.unwrap_or_default();
    duel.hint(HINT_ZONE, tp, u64::from(zone));
    duel.hint(HINT_ZONE, 1 - tp, u64::from(zone >> 16));
    e.set_label(duel, i64::from(zone));
    let sg = groupfrombit(duel, zone >> 16, 1 - tp);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = groupfrombit(duel, (e.get_label(duel) as u32) >> 16, 1 - tp);
    if g.is_empty() || c.get_overlay_count(duel) == 0 || c.remove_overlay_card(duel, tp, 1, g.len() as u16, REASON_EFFECT)? < 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let sg = g.select(duel, tp, 1, duel.get_operated_group().len() as u8, None)?;
    if !sg.is_empty() {
        duel.destroy(sg, REASON_EFFECT)?;
    }
    Ok(())
}

fn bancon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_CHAINING)
        && duel.is_turn_player(1 - a.tp)
        && (duel.is_main_phase() || duel.is_battle_phase())
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && a.e.get_handler(duel).is_able_to_remove(duel))
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        duel.remove(c, POS_FACEUP, REASON_EFFECT | REASON_TEMPORARY)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        e1.set_label_object_group(duel, Some(c.into()));
        e1.set_count_limit(duel, 1, 0);
        e1.set_operation(duel, retop);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(c) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(c)?;
    }
    Ok(())
}
