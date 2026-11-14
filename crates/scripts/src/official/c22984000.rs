// Mercurium the Living Quicksilver
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 22984000;

// On the field for the End Phase effect, and in the Extra Deck, Xyz Summoned from Level 10 monsters (Super Express Bullet Train,
// Heavy Armored Knight Babeldecker) to attach a monster from the Deck; monsters in each Graveyard for the activation lock.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (52481437, MZONE),
    (52481437, MZONE),
    (45116390, MZONE),
    (45116390, MZONE),
    (26202165, GRAVE),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(10);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, attachtg);
    e1.set_operation(duel, attachop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e2.set_value(duel, Value::EffectPlayer(cannotactval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, detachtg);
    e3.set_operation(duel, detachop);
    c.register_effect(duel, e3);
}

fn attachfilter(c: CardId, duel: &Duel, xyzc: CardId, tp: u8) -> bool {
    c.is_monster(duel) && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    Ok(chk == 0 && duel.is_existing_matching_card(|sc, duel| attachfilter(sc, duel, c, tp), tp, LOCATION_DECK, 0, 1, None))
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, a.e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
        let g = duel.select_matching_card(tp, |sc, duel| attachfilter(sc, duel, c, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.overlay(c, sc)?;
        }
    }
    Ok(())
}

fn cannotactval(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel)
        && rc.is_location(duel, LOCATION_GRAVE)
        && duel.is_existing_matching_card(
            |c, duel| c.is_attribute(duel, rc.get_attribute(duel)),
            e.get_handler_player(duel),
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn detachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    if c.get_overlay_count(duel) == 0 {
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, a.tp, 3000);
    }
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, a.tp, 3000);
    Ok(false)
}

fn detachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, a.e) {
        duel.damage(tp, 3000, REASON_EFFECT)?;
        return Ok(());
    }
    let b1 = c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT);
    let op = match b1 {
        true => duel
            .select_effect(tp, &[(b1, aux::stringid(ID, 2)), (true, aux::stringid(ID, 3))])?
            .unwrap_or(2),
        false => 2,
    };
    if op == 1 {
        c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
    } else if op == 2 {
        duel.damage(tp, 3000, REASON_EFFECT)?;
    }
    Ok(())
}
