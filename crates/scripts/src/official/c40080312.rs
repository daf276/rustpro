// Elemental HERO Nebula Neos
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 40080312;
const GRAND_MOLE: u32 = 80344569;
const DARK_PANTHER: u32 = 43237273;
pub const LISTED_NAMES: &[u32] = &[CARD_NEOS, GRAND_MOLE, DARK_PANTHER];
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO, SET_NEOS, SET_NEO_SPACIAN];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_NEOS),
        FusionMaterial::Code(GRAND_MOLE),
        FusionMaterial::Code(DARK_PANTHER),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(CardId::is_able_to_deck_or_extra_as_cost, tp, LOCATION_ONFIELD, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, tp: u8, _: CardId) -> Script<()> {
        duel.confirm_cards(1 - tp, g)?;
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

struct Return;

impl aux::NeosReturn for Return {
    const EXTRACAT: u64 = CATEGORY_REMOVE;

    fn extrainfo(duel: &mut Duel, a: &Args) {
        let g = removable(duel, a.tp);
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
        duel.set_operation_info(0, CATEGORY_TODECK, Some(a.e.get_handler(duel).into()), 1, a.tp, 0);
    }

    fn extraop(duel: &mut Duel, a: &Args) -> Script<()> {
        let g = removable(duel, a.tp);
        duel.remove(g, POS_FACEDOWN, REASON_EFFECT)?;
        Ok(())
    }
}

// In the Extra Deck for its contact fusion with Neos, Grand Mole and Dark Panther on the field, against the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_NEOS, MZONE),
    (GRAND_MOLE, MZONE),
    (DARK_PANTHER, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(0),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    aux::enable_neos_return::<Return>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_EXTRA));
    e1.set_target(duel, drtg);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    !e.get_handler(duel).is_location(duel, LOCATION_EXTRA)
}

// The Lua passes Card.IsAbleToRemove the arguments (1, card or nil, tp, POS_FACEDOWN): player 1, the default position and
// the reason tp (a mirrored oracle quirk), with no exception.
fn removable(duel: &Duel, tp: u8) -> CardSet {
    let filter = |c: CardId, duel: &Duel| c.is_able_to_remove_full(duel, Some(1), POS_FACEUP, u32::from(tp));
    duel.get_matching_group(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_field_group_count(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d = duel.get_field_group_count(tp, 0, LOCATION_ONFIELD);
    let g = duel.get_matching_group(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if duel.draw(p, d as u32, REASON_EFFECT)? > 0 && !g.is_empty() {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
        if let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            tc.negate_effects(duel, e.get_handler(duel), RESETS_STANDARD_PHASE_END, true);
        }
    }
    Ok(())
}
