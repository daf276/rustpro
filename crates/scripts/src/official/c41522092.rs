// Number F0: Utopic Future Zexal
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 41522092;

// In the Extra Deck, Xyz Summoned with two Evilswarm Exciton Knights, and an opponent's monster to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (46772449, MZONE),
    (46772449, MZONE),
    (46772449, MZONE),
    (15025844, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_type_full(duel, TYPE_XYZ, Some(scard), sumtype, tp));
    const LV: Option<u32> = None;
    const CT: usize = 2;

    fn exchk(duel: &Duel, g: CardSet, _: u8, _: CardId) -> bool {
        let mg = g.filter(duel, |c, duel| c.is_has_effect(duel, EFFECT_EQUIP_SPELL_XYZ_MAT).is_none(), None);
        mg.get_class_count(duel, |c, duel| i64::from(c.get_rank(duel))) == 1
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| xyz_ranks(duel, e.get_handler_player(duel)) as i32 * 500),
    );
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE);
    e4.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, LOCATION_ONFIELD, LOCATION_ONFIELD);
    e4.set_target_card(duel, |duel, e, c| c != e.get_handler(duel));
    e4.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_CONTROL);
    e5.set_type(duel, EFFECT_TYPE_QUICK_O);
    e5.set_code(duel, EVENT_CHAINING);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, ctcon);
    e5.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e5.set_target(duel, cttg);
    e5.set_operation(duel, ctop);
    c.register_effect(duel, e5);
}

fn xyz_ranks(duel: &Duel, player: u8) -> i64 {
    let g = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_XYZ)),
        player,
        LOCATION_MZONE,
        LOCATION_GRAVE,
        None,
    );
    g.get_sum(duel, |c, duel| i64::from(c.get_rank(duel)))
}

fn ctcon(duel: &Duel, a: &Args) -> bool {
    let trig_loc: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_LOCATION);
    let trig_p: u8 = duel.get_chain_info(0, CHAININFO_TRIGGERING_PLAYER);
    trig_p == 1 - a.tp && trig_loc & LOCATION_ONFIELD > 0
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_controler_can_be_changed, a.tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_CONTROL, None, 1, a.tp, 0);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_matching_card(tp, CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.get_control(g, tp)?;
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3008);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        c.register_effect(duel, e2);
    }
    Ok(())
}
