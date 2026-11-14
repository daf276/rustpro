// Ghostrick Festival
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 35871958;

pub const LISTED_SERIES: &[u16] = &[SET_GHOSTRICK];

const GHOSTRICK_JIANGSHI: u32 = 80885284;
const GHOSTRICK_FAIRY: u32 = 36239585;
const ELFNOTE_LUCINA: u32 = 13597785;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_GHOSTRICK], Some(lc), sumtype, tp) && !c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

// On the field, Link Summoned from the Extra Deck with Ghostrick Jiangshi (often face-down), with an attacker of the opponent's to
// tribute it for and Ghostrick Fairy in the Deck to Special Summon. No Ghostrick Field Spell is ported, so the direct attack
// is not reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (GHOSTRICK_JIANGSHI, MZONE),
    (GHOSTRICK_JIANGSHI, MZONE),
    (GHOSTRICK_JIANGSHI, MZONE),
    (GHOSTRICK_FAIRY, DECK),
    (GHOSTRICK_FAIRY, DECK),
    (ELFNOTE_LUCINA, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_code(duel, EFFECT_EXTRA_MATERIAL);
    e0.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET | EFFECT_FLAG_CANNOT_DISABLE | EFFECT_FLAG_SET_AVAILABLE,
    );
    e0.set_target_range(duel, 1, 0);
    e0.set_value(duel, Value::ExtraMaterial(extraval));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_condition(duel, dacon);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_GHOSTRICK]));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn extraval(duel: &mut Duel, e: EffectId, call: ExtraMaterialCall) -> CardSet {
    match call {
        ExtraMaterialCall::Get { summon_type, tp, sc } if summon_type == SUMMON_TYPE_LINK && sc == e.get_handler(duel) => {
            duel.get_matching_group(CardId::is_facedown, tp, LOCATION_MZONE, 0, None)
        }
        _ => CardSet::default(),
    }
}

fn dacon(duel: &Duel, _: &Args) -> bool {
    let ghostrick = |c: CardId, duel: &Duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_GHOSTRICK]);
    duel.is_existing_matching_card(ghostrick, 0, LOCATION_FZONE, LOCATION_FZONE, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GHOSTRICK]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
