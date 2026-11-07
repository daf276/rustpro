// Port of utility.lua and cards_specific_functions.lua: the Auxiliary and Cost helpers and the Card, Effect and Duel
// methods they add.
use engine::{Args, CardId, CardSet, CostFn, Duel, EffectId, Script, Value};

use crate::chain::EffectChain;
use crate::constant::*;
use crate::proc_workaround::{CardWorkaround, DuelWorkaround};

// The chain data slots of cd.cost_detached_materials and cd.cost_discarded_cards, above those card ports use.
const COST_DETACHED_MATERIALS: u8 = u8::MAX;
const COST_DISCARDED_CARDS: u8 = u8::MAX - 1;
const COST_LP_PAID: u8 = u8::MAX - 2;
const COST_CHOICE: u8 = u8::MAX - 3;

// Cost.Choice's {cost, description, additional_check}.
pub type CostChoice = (CostFn, u64, fn(&Duel, &Args) -> bool);

pub mod aux {
    use engine::{Args, CardId, CardSet, Duel, EffectId, ExtraMaterialCall, Operation, Script, Value};

    use crate::chain::DuelChain;
    use crate::constant::*;
    use crate::proc_workaround::CardWorkaround;
    use crate::utility::CardUtility;

    pub const TRUE: fn(CardId, &Duel) -> bool = |_, _| true;

    // The group check (sg, mg, c) of SelectUnselectGroup, whose e and tp are captures; returns (res, stop).
    pub type GroupCheck<'a> = &'a dyn Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool);

    // The emt table of aux.GetExtraMaterials: per EFFECT_EXTRA_MATERIAL effect the extra materials it offers.
    pub type ExtraMaterials = Vec<(CardSet, EffectId)>;

    pub const fn stringid(code: u32, id: u64) -> u64 {
        (id & 0xfffff) | ((code as u64) << 20)
    }

    // s.global_check per script, as script state keyed by the script's id.
    pub fn global_check(duel: &mut Duel, id: u32, func: impl FnOnce(&mut Duel)) {
        let key = (1 << 61) | u64::from(id);
        if duel.get_script_state(key) == 0 {
            duel.set_script_state(key, 1);
            func(duel);
        }
    }

    pub fn stat_change_damage_step_condition(duel: &Duel, _: &Args) -> bool {
        !(duel.is_phase(PHASE_DAMAGE) && duel.is_damage_calculated())
    }

    pub fn exccon(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        duel.get_turn_count() != c.get_turn_id(duel) || c.is_reason(duel, REASON_RETURN)
    }

    pub fn faceup_filter(filter: impl Fn(CardId, &Duel) -> bool) -> impl Fn(CardId, &Duel) -> bool {
        move |c, duel| c.is_faceup(duel) && filter(c, duel)
    }

    pub fn nvfilter(c: CardId, duel: &Duel) -> bool {
        c.is_has_effect(duel, EFFECT_NECRO_VALLEY).is_none()
    }

    pub fn necro_valley_filter(filter: impl Fn(CardId, &Duel) -> bool) -> impl Fn(CardId, &Duel) -> bool {
        move |c, duel| filter(c, duel) && !(c.is_has_effect(duel, EFFECT_NECRO_VALLEY).is_some() && duel.is_chain_disablable(0))
    }

    pub fn sp_elim_filter(c: CardId, duel: &Duel, mustbefaceup: bool, includemzone: bool) -> bool {
        if !c.is_monster(duel) {
            return includemzone || c.is_location(duel, LOCATION_GRAVE);
        }
        if mustbefaceup && c.is_location(duel, LOCATION_MZONE) && c.is_facedown(duel) {
            return false;
        }
        let spirit_elimination = duel.is_player_affected_by_effect(c.get_controler(duel), 69832741).is_some();
        if includemzone {
            return c.is_location(duel, LOCATION_MZONE) || !spirit_elimination;
        }
        c.is_location(duel, if spirit_elimination { LOCATION_MZONE } else { LOCATION_GRAVE })
    }

    pub fn register_client_hint(
        duel: &mut Duel,
        card: CardId,
        property: u32,
        tp: u8,
        player1: u32,
        player2: u32,
        description: Option<u64>,
    ) -> EffectId {
        register_client_hint_full(duel, card, property, tp, player1, player2, description, 0)
    }

    // `description` None takes the card's second string.
    pub fn register_client_hint_full(
        duel: &mut Duel,
        card: CardId,
        property: u32,
        tp: u8,
        player1: u32,
        player2: u32,
        description: Option<u64>,
        reset: u32,
    ) -> EffectId {
        let eff = duel.create_effect(card);
        eff.set_property(duel, (EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT) | property);
        eff.set_target_range(duel, player1, player2);
        let description = description.unwrap_or_else(|| stringid(card.get_original_code(duel), 1));
        eff.set_description(duel, description);
        eff.set_reset(duel, (RESET_PHASE + PHASE_END) | reset);
        duel.register_effect(eff, tp);
        eff
    }

    // Auxiliary.ToHandOrElse for one card: to the hand, or else `oper` where `check` allows it (`str` its option).
    pub fn to_hand_or_else(
        duel: &mut Duel,
        card: CardId,
        player: u8,
        check: impl Fn(CardId, &Duel) -> bool,
        oper: impl FnOnce(&mut Duel) -> Script<()>,
        str: u64,
    ) -> Script<()> {
        let b1 = card.is_able_to_hand(duel);
        let b2 = check(card, duel);
        let opt = if b1 && b2 {
            duel.select_option(player, &[573, str])?
        } else if b1 {
            duel.select_option(player, &[573])?
        } else {
            duel.select_option(player, &[str])? + 1
        };
        if opt != 0 {
            return oper(duel);
        }
        if duel.send_to_hand(card, PLAYER_NONE, REASON_EFFECT)? > 0 {
            let og = duel
                .get_operated_group()
                .filter(duel, |c, duel| c.is_previous_location(duel, LOCATION_DECK), None);
            if !og.is_empty() {
                duel.confirm_cards(1 - player, og)?;
            }
        }
        Ok(())
    }

    fn select_unselect_loop(duel: &Duel, c: CardId, sg: CardSet, mg: CardSet, minc: usize, maxc: usize, rescon: GroupCheck) -> bool {
        if sg.len() >= maxc {
            return false;
        }
        let mut sg = sg;
        sg.insert(c);
        let (mut res, stop) = rescon(duel, sg, mg, Some(c));
        if stop {
            return false;
        }
        if sg.len() < minc || (sg.len() < maxc && !res) {
            res = mg.is_exists(
                duel,
                |other, duel| select_unselect_loop(duel, other, sg, mg, minc, maxc, rescon),
                1,
                Some(sg),
            );
        }
        res
    }

    // Auxiliary.SelectUnselectGroup with chk=0: whether `rescon` accepts some group of `minc` to `maxc` cards of g.
    pub fn select_unselect_group_check(duel: &Duel, g: CardSet, minc: usize, maxc: usize, rescon: GroupCheck) -> bool {
        if g.len() < minc {
            return false;
        }
        let mut eg = g;
        for c in g {
            if select_unselect_loop(duel, c, CardSet::default(), eg, minc, maxc, rescon) {
                return true;
            }
            eg.remove(c);
        }
        false
    }

    pub fn select_unselect_group(
        duel: &mut Duel,
        g: CardSet,
        minc: usize,
        maxc: usize,
        rescon: GroupCheck,
        seltp: u8,
        hintmsg: u64,
    ) -> Script<CardSet> {
        select_unselect_group_full(duel, g, minc, maxc, rescon, seltp, hintmsg, None, false)
    }

    // Auxiliary.SelectUnselectGroup with chk=1: `seltp` picks or drops one card at a time.
    pub fn select_unselect_group_full(
        duel: &mut Duel,
        g: CardSet,
        minc: usize,
        maxc: usize,
        rescon: GroupCheck,
        seltp: u8,
        hintmsg: u64,
        finishcon: Option<GroupCheck>,
        cancelable: bool,
    ) -> Script<CardSet> {
        let mut sg = CardSet::default();
        loop {
            let finishable = sg.len() >= minc && finishcon.is_none_or(|finishcon| finishcon(duel, sg, g, None).0);
            let mg = g.filter(duel, |c, duel| select_unselect_loop(duel, c, sg, g, minc, maxc, rescon), Some(sg));
            if mg.is_empty() || sg.len() >= maxc {
                break;
            }
            duel.hint(HINT_SELECTMSG, seltp, hintmsg);
            let cancel = finishable || (cancelable && sg.is_empty());
            let Some(tc) = mg.select_unselect(duel, Some(sg), seltp, finishable, cancel, minc as u8, maxc as u8)? else {
                break;
            };
            if sg.is_contains(tc) {
                sg.remove(tc);
            } else {
                sg.insert(tc);
            }
        }
        Ok(sg)
    }

    // Clock Lizard's check, which nothing ported reads; the effects are registered all the same.
    fn create_lizard_check(duel: &mut Duel, c: CardId, filter: fn(&Duel, EffectId, CardId) -> bool) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, CARD_CLOCK_LIZARD);
        e1.set_target_range(duel, 0xff, 0);
        e1.set_target_card(duel, filter);
        e1.set_value(duel, 1);
        e1
    }

    // Auxiliary.DoubleSnareValidity(c, range), a marker effect that nothing ported reads.
    pub fn double_snare_validity(duel: &mut Duel, c: CardId, range: u32) {
        let eff = duel.create_effect(c);
        eff.set_type(duel, EFFECT_TYPE_SINGLE);
        eff.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE | EFFECT_FLAG_SINGLE_RANGE);
        eff.set_range(duel, range);
        eff.set_code(duel, 3682106);
        c.register_effect(duel, eff);
    }

    pub fn add_lizard_check(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, CARD_CLOCK_LIZARD);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_value(duel, 1);
        c.register_effect(duel, e1);
        e1
    }

    pub fn add_temp_lizard_check(duel: &mut Duel, c: CardId, tp: u8) -> EffectId {
        add_temp_lizard_check_full(duel, c, tp, |_, _, _| true)
    }

    pub fn add_temp_lizard_check_full(duel: &mut Duel, c: CardId, tp: u8, filter: fn(&Duel, EffectId, CardId) -> bool) -> EffectId {
        let e1 = create_lizard_check(duel, c, filter);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        e1
    }

    pub fn add_continuous_lizard_check(duel: &mut Duel, c: CardId, location: u32, filter: fn(&Duel, EffectId, CardId) -> bool) -> EffectId {
        let e1 = create_lizard_check(duel, c, filter);
        e1.set_range(duel, location);
        c.register_effect(duel, e1);
        e1
    }

    // The oper of DelayedOperation and RemoveUntil, as a type because effect operations cannot capture it.
    pub trait DelayedOperation {
        fn operate(duel: &mut Duel, rg: CardSet, a: &Args) -> Script<()>;
    }

    // delayed_operation_id, the global counter of utility.lua.
    const DELAYED_OPERATION_ID: u64 = 3;

    // The delayed effect keeps the flag beside the fid in its label, where the Lua closures capture it.
    fn affected_group(duel: &Duel, e: EffectId) -> CardSet {
        let &[fid, flag] = e.get_labels(duel) else {
            return CardSet::default();
        };
        let g = e.get_label_object_group(duel).unwrap_or_default();
        g.filter(duel, |c, duel| c.get_flag_effect_label(duel, flag as u32) == Some(fid), None)
    }

    pub fn delayed_operation<O: DelayedOperation>(
        duel: &mut Duel,
        g: CardSet,
        phase: u32,
        flag: u32,
        e: EffectId,
        tp: u8,
    ) -> Option<EffectId> {
        if g.is_empty() {
            return None;
        }
        let fid = duel.get_script_state(DELAYED_OPERATION_ID) + 1;
        duel.set_script_state(DELAYED_OPERATION_ID, fid);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_code(duel, EVENT_PHASE | phase);
        e1.set_reset(duel, RESET_PHASE | phase);
        e1.set_count_limit(duel, 1, 0);
        e1.set_labels(duel, &[fid, flag.into()]);
        e1.set_label_object_group(duel, Some(g));
        e1.set_condition(duel, |duel, a| !affected_group(duel, a.e).is_empty());
        e1.set_operation(duel, |duel, a| O::operate(duel, affected_group(duel, a.e), a));
        duel.register_effect(e1, tp);
        for tc in g {
            let flag_effect = tc.register_flag_effect_full(duel, flag, RESET_EVENT + RESETS_STANDARD, 0, 1, fid, 0);
            flag_effect.set_label_object(duel, Some(e1));
            flag_effect.set_condition(duel, |duel, a| !a.e.get_label_object(duel).is_some_and(|e1| e1.is_deleted(duel)));
        }
        Some(e1)
    }

    pub fn remove_until<O: DelayedOperation>(
        duel: &mut Duel,
        g: impl Into<CardSet>,
        position: u32,
        reason: u32,
        phase: u32,
        flag: u32,
        e: EffectId,
        tp: u8,
    ) -> Script<Option<EffectId>> {
        let g = g.into();
        if g.is_empty() || duel.remove(g, position, reason | REASON_TEMPORARY)? == 0 {
            return Ok(None);
        }
        let g = g.filter(duel, |c, duel| c.is_location(duel, LOCATION_REMOVED), None);
        Ok(if g.is_empty() {
            None
        } else {
            delayed_operation::<O>(duel, g, phase, flag, e, tp)
        })
    }

    // Auxiliary.IsZone: whether the card's zone, as seen from tp, is in `zone`.
    pub fn is_zone(c: CardId, duel: &Duel, zone: u32, tp: u8) -> bool {
        let sequence = c.get_sequence(duel);
        let own = c.is_controler(duel, tp);
        let mut rzone = if own { 1 << sequence } else { 1 << (16 + sequence) };
        if c.is_sequence(duel, 5) || c.is_sequence(duel, 6) {
            rzone |= if own { 1 << (16 + 11 - sequence) } else { 1 << (11 - sequence) };
        }
        rzone & zone > 0
    }

    // The lowest limit of the player's Summon Gate effects, which come with the cards that register them.
    pub fn check_summon_gate(duel: &Duel, tp: u8) -> Option<i32> {
        if !duel.get_player_effect(tp, CARD_SUMMON_GATE).is_empty() {
            duel.reach_unported(concat!(file!(), ":", line!()));
        }
        None
    }

    // Auxiliary.GetMustBeMaterialGroup: the monsters EFFECT_MUST_BE_MATERIAL effects of the player force on a summon;
    // those effects come with the cards that register them.
    pub fn get_must_be_material_group(
        duel: &Duel,
        tp: u8,
        _eg: Option<CardSet>,
        _sump: u8,
        _sc: Option<CardId>,
        _g: Option<CardSet>,
        _r: u32,
    ) -> CardSet {
        unported!(duel, !duel.get_player_effect(tp, EFFECT_MUST_BE_MATERIAL).is_empty());
        CardSet::default()
    }

    // Auxiliary.GetExtraMaterials: per EFFECT_EXTRA_MATERIAL effect whose count limit allows a use, the materials its
    // value offers (chk 0, which may register effects), and all of them together.
    pub fn get_extra_materials(duel: &mut Duel, tp: u8, mustg: CardSet, sc: CardId, summon_type: u32) -> (ExtraMaterials, CardSet) {
        let (mut t, mut tg) = (ExtraMaterials::new(), CardSet::default());
        for te in duel.get_player_effect(tp, EFFECT_EXTRA_MATERIAL) {
            if te.check_count_limit(duel, tp) {
                let Value::ExtraMaterial(value) = te.get_value(duel) else {
                    continue;
                };
                let eg = value(duel, te, ExtraMaterialCall::Get { summon_type, tp, sc }).difference(mustg);
                tg = tg.union(eg);
                t.push((eg, te));
            }
        }
        (t, tg)
    }

    // efun(c, e, tp, sg, mg, lc, og, chk) of an extra material entry; a missing operation is aux.TRUE.
    pub fn extra_material_filter(
        duel: &Duel,
        (eg, te): (CardSet, EffectId),
        c: CardId,
        tp: u8,
        sg: CardSet,
        mg: CardSet,
        lc: CardId,
        chk: i32,
    ) -> bool {
        match te.get_operation(duel) {
            Some(Operation::ExtraMaterial(efun)) => efun(duel, c, te, tp, sg, mg, lc, eg, chk),
            _ => true,
        }
    }

    // Auxiliary.CheckValidExtra: whether an extra material entry takes c, adding every such entry to `filt`.
    pub fn check_valid_extra(
        duel: &Duel,
        c: CardId,
        tp: u8,
        sg: CardSet,
        mg: CardSet,
        lc: CardId,
        emt: &ExtraMaterials,
        filt: &mut Vec<usize>,
    ) -> bool {
        let mut res = false;
        for (index, &ex) in emt.iter().enumerate() {
            if ex.0.is_contains(c) && extra_material_filter(duel, ex, c, tp, sg, mg, lc, 0) {
                res = true;
                filt.push(index);
            }
        }
        res
    }

    // Auxiliary.DeleteExtraMaterialGroups: value chk 2 of every entry, which resets what chk 0 registered.
    pub fn delete_extra_material_groups(duel: &mut Duel, emt: &ExtraMaterials) {
        for &(eg, te) in emt {
            if let Value::ExtraMaterial(value) = te.get_value(duel) {
                value(duel, te, ExtraMaterialCall::Delete { eg });
            }
        }
    }

    const KEPT_EXTRA_MATERIALS: u64 = 1 << 60;

    // Link.Target keeps {sg, filters, emt} as the label object of e for Link.Operation: the filters (indices into emt,
    // at most 15) packed into the script state, each entry's group in the script groups.
    pub fn keep_extra_materials(duel: &mut Duel, e: EffectId, emt: &ExtraMaterials, filt: &[usize]) {
        let key = KEPT_EXTRA_MATERIALS | (e.index() as u64) << 16;
        for (index, &(eg, _)) in emt.iter().enumerate() {
            duel.set_script_group(key | index as u64, Some(eg));
        }
        duel.set_script_state(key, filt.iter().rev().fold(0, |packed, &index| packed << 4 | (index as i64 + 1)));
    }

    // What keep_extra_materials kept. The effects are looked up again, in the state the target left.
    pub fn kept_extra_materials(duel: &Duel, e: EffectId, tp: u8) -> (ExtraMaterials, Vec<usize>) {
        let key = KEPT_EXTRA_MATERIALS | (e.index() as u64) << 16;
        let effects = duel.get_player_effect(tp, EFFECT_EXTRA_MATERIAL);
        let usable = effects.iter().filter(|te| te.check_count_limit(duel, tp));
        let emt = usable
            .enumerate()
            .map(|(index, &te)| (duel.get_script_group(key | index as u64).unwrap_or_default(), te))
            .collect();
        let packed = std::iter::successors(Some(duel.get_script_state(key)), |packed| Some(packed >> 4));
        (
            emt,
            packed
                .take_while(|&packed| packed != 0)
                .map(|packed| (packed & 0xf) as usize - 1)
                .collect(),
        )
    }

    // Auxiliary.dpcheck(fun): a SelectUnselectGroup check that the selected cards all differ in `fun`.
    pub fn dpcheck(fun: impl Fn(CardId, &Duel) -> i64) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
        move |duel, sg, _, _| {
            let (c1, c2) = (sg.get_class_count(duel, &fun), sg.len());
            (c1 == c2, c1 != c2)
        }
    }

    // Auxiliary.ChangeBattleDamage(PLAYER, VALUE): -1 leaves the damage alone.
    pub fn change_battle_damage<const PLAYER: u8, const VALUE: u32>(duel: &Duel, e: EffectId, damp: u8, _: CardId) -> i64 {
        let target = if PLAYER == 0 { damp } else { 1 - damp };
        if e.get_owner_player(duel) == target { i64::from(VALUE) } else { -1 }
    }

    // Auxiliary.indoval: EFFECT_INDESTRUCTABLE_EFFECT for the opponent's effects.
    pub fn indoval(duel: &Duel, e: EffectId, _re: EffectId, rp: u8) -> bool {
        rp == 1 - e.get_handler_player(duel)
    }

    // Auxiliary.tgoval: EFFECT_CANNOT_BE_EFFECT_TARGET for the opponent's effects.
    pub fn tgoval(duel: &Duel, e: EffectId, _re: EffectId, rp: u8) -> bool {
        rp != e.get_handler_player(duel)
    }

    // Auxiliary.cannotmatfilter(...) applied: whether `sumtype` is one of the summon types in `allowed`.
    pub fn cannotmatfilter(allowed: &[u64], sumtype: u64) -> bool {
        let tot = allowed.iter().fold(0, |tot, &value| tot | value);
        let sum = tot & sumtype;
        allowed.contains(&sum)
    }

    fn select_field_return_cards(duel: &mut Duel, p: u8, g: CardSet) -> Script<()> {
        if g.is_empty() {
            return Ok(());
        }
        let mut g = g;
        let ft = duel.get_location_count(p, LOCATION_MZONE);
        if ft > 0 && g.len() > ft as usize {
            duel.hint(HINT_SELECTMSG, p, HINTMSG_TOFIELD);
            let tg = g.select(duel, p, ft as u8, ft as u8, None)?;
            for tc in tg {
                duel.return_to_field(tc)?;
            }
            g = g.difference(tg);
        }
        for tc in g {
            duel.return_to_field(tc)?;
        }
        Ok(())
    }

    // Auxiliary.DefaultFieldReturnOp: the banished cards return to the monster zones, the turn player's first.
    pub struct DefaultFieldReturnOp;

    impl DelayedOperation for DefaultFieldReturnOp {
        fn operate(duel: &mut Duel, rg: CardSet, _: &Args) -> Script<()> {
            if rg.is_empty() {
                return Ok(());
            }
            let turn_p = duel.get_turn_player();
            let g0 = rg.filter(duel, |c, duel| c.is_previous_controler(duel, turn_p), None);
            let g1 = rg.difference(g0);
            select_field_return_cards(duel, turn_p, g0)?;
            select_field_return_cards(duel, 1 - turn_p, g1)
        }
    }
}

pub struct Cost;

impl Cost {
    pub fn self_discard(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_discardable(duel));
        }
        duel.send_to_grave(c, REASON_DISCARD | REASON_COST)?;
        Ok(false)
    }

    pub fn self_to_grave(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_able_to_grave_as_cost(duel));
        }
        duel.send_to_grave(c, REASON_COST)?;
        Ok(false)
    }

    pub fn self_banish(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_able_to_remove_as_cost(duel));
        }
        duel.remove(c, POS_FACEUP, REASON_COST)?;
        Ok(false)
    }

    // Cost.Replaceable(base) applied: the base cost, unless one of the player's EFFECT_COST_REPLACE effects stands
    // in for it (those come with the cards that register them).
    pub fn replaceable(duel: &mut Duel, args: &Args, chk: i32, base: CostFn) -> Script<bool> {
        let base_chk = base(duel, args, 0)?;
        if !duel.get_player_effect(args.tp, EFFECT_COST_REPLACE).is_empty() {
            duel.reach_unported(concat!(file!(), ":", line!()));
        }
        if chk == 0 {
            return Ok(base_chk);
        }
        base(duel, args, 1)
    }

    pub fn self_tribute(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_releasable(duel));
        }
        duel.release(c, REASON_COST)?;
        Ok(false)
    }

    // Cost.Discard() applied: one card from the hand; the discarded cards are kept as the chain data's
    // cost_discarded_cards.
    pub fn discard(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        Cost::discard_full(duel, args, chk, aux::TRUE, false, (1, 1))
    }

    // Cost.Discard(filter, other, min, max) applied, with the counts given.
    pub fn discard_full(
        duel: &mut Duel,
        args: &Args,
        chk: i32,
        filter: impl Fn(CardId, &Duel) -> bool,
        other: bool,
        (min, max): (u8, u8),
    ) -> Script<bool> {
        let (e, tp) = (args.e, args.tp);
        let exclude = other.then(|| CardSet::from(e.get_handler(duel)));
        let filter_final = |c: CardId, duel: &Duel| filter(c, duel) && c.is_discardable(duel);
        if chk == 0 {
            return Ok(min > 0
                && max >= min
                && duel.is_existing_matching_card(filter_final, tp, LOCATION_HAND, 0, usize::from(min), exclude));
        }
        duel.discard_hand_full(tp, filter_final, min, max, REASON_COST | REASON_DISCARD, exclude)?;
        let discarded = duel.get_operated_group();
        e.set_chain_data_group(duel, COST_DISCARDED_CARDS, discarded);
        Ok(false)
    }

    // Cost.DetachFromSelf(min, max) applied; the detached materials are kept as the chain data's
    // cost_detached_materials.
    pub fn detach_from_self<const MIN: u16, const MAX: u16>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        Cost::detach_from_self_op::<MIN, MAX>(duel, args, chk, |_, _, _| {})
    }

    // Cost.DetachFromSelf(min, max, op): `op(e, og)` runs on the detached materials.
    pub fn detach_from_self_op<const MIN: u16, const MAX: u16>(
        duel: &mut Duel,
        args: &Args,
        chk: i32,
        op: fn(&mut Duel, EffectId, CardSet),
    ) -> Script<bool> {
        let (e, tp) = (args.e, args.tp);
        let c = e.get_handler(duel);
        if chk == 0 {
            return Ok(MIN > 0 && MAX >= MIN && c.check_remove_overlay_card(duel, tp, MIN, REASON_COST));
        }
        if c.remove_overlay_card(duel, tp, MIN, MAX, REASON_COST)? > 0 {
            let detached = duel.get_operated_group();
            e.set_chain_data_group(duel, COST_DETACHED_MATERIALS, detached);
            op(duel, e, detached);
        }
        Ok(false)
    }

    // Cost.PayLP(LP) applied for an LP of at least 1; the LP paid is kept as the chain data's cost_lp_paid.
    pub fn pay_lp<const LP: u32>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(duel.check_lp_cost(args.tp, LP));
        }
        duel.pay_lp_cost(args.tp, LP)?;
        args.e.set_chain_data(duel, COST_LP_PAID, i64::from(LP));
        Ok(false)
    }

    // Cost.AND(...) applied. A cost that returns false when executed, which stops the others, has no caller.
    pub fn and(duel: &mut Duel, args: &Args, chk: i32, costs: &[CostFn]) -> Script<bool> {
        for cost in costs {
            if !cost(duel, args, chk)? && chk == 0 {
                return Ok(false);
            }
        }
        Ok(chk == 0)
    }

    // Cost.Choice(...) applied to {cost, description, additional_check}; the chosen index (from 1) is kept as the chain
    // data's cost_choice.
    pub fn choice(duel: &mut Duel, args: &Args, chk: i32, choices: &[CostChoice]) -> Script<bool> {
        let mut ops = Vec::with_capacity(choices.len());
        for &(cost, description, additional_check) in choices {
            let check = cost(duel, args, 0)? && additional_check(duel, args);
            ops.push((check, description));
        }
        if chk == 0 {
            return Ok(ops.iter().any(|&(check, _)| check));
        }
        let Some(choice) = duel.select_effect(args.tp, &ops)? else {
            return Ok(false);
        };
        args.e.set_chain_data(duel, COST_CHOICE, choice as i64);
        if let Some(&(cost, ..)) = choice.checked_sub(1).and_then(|index| choices.get(index)) {
            cost(duel, args, 1)?;
        }
        Ok(false)
    }

    pub fn get_choice(duel: &Duel, e: EffectId) -> Option<i64> {
        e.get_chain_data(duel, COST_CHOICE)
    }

    // Cost.Reveal(filter, other, min, max, op, location) applied, with the counts given.
    pub fn reveal(
        duel: &mut Duel,
        args: &Args,
        chk: i32,
        filter: fn(CardId, &Duel, EffectId, u8) -> bool,
        other: bool,
        (min, max): (u8, u8),
        op: Option<fn(&mut Duel, &Args, CardSet)>,
        location: u32,
    ) -> Script<bool> {
        let (e, tp) = (args.e, args.tp);
        let exclude = other.then(|| CardSet::from(e.get_handler(duel)));
        let filter_final = |c: CardId, duel: &Duel| filter(c, duel, e, tp) && !c.is_public(duel);
        if chk == 0 {
            return Ok(min > 0 && max >= min && duel.is_existing_matching_card(filter_final, tp, location, 0, usize::from(min), exclude));
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let g = duel.select_matching_card(tp, filter_final, tp, location, 0, min, max, exclude)?;
        duel.confirm_cards(1 - tp, g)?;
        if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_HAND), 1, None) {
            duel.shuffle_hand(tp)?;
        }
        if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
            duel.shuffle_deck(tp)?;
        }
        if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), 1, None) {
            duel.shuffle_extra(tp)?;
        }
        if let Some(op) = op {
            op(duel, args, g);
        }
        Ok(false)
    }

    // Cost.RemoveCounterFromField(counter_type, count) applied.
    pub fn remove_counter_from_field<const COUNTER_TYPE: u16, const COUNT: u16>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(duel.is_can_remove_counter(args.tp, 1, 0, COUNTER_TYPE, COUNT, REASON_COST));
        }
        duel.remove_counter(args.tp, 1, 0, COUNTER_TYPE, COUNT, REASON_COST)?;
        Ok(false)
    }
}

pub trait CardUtility: Copy {
    fn is_monster(self, duel: &Duel) -> bool;
    fn is_spell(self, duel: &Duel) -> bool;
    fn is_trap(self, duel: &Duel) -> bool;
    fn is_spell_trap(self, duel: &Duel) -> bool;
    fn is_monster_card(self, duel: &Duel) -> bool;
    fn is_owner(self, duel: &Duel, player: u8) -> bool;
    fn is_field_spell(self, duel: &Duel) -> bool;
    fn is_ritual_spell(self, duel: &Duel) -> bool;
    fn is_ritual_monster(self, duel: &Duel) -> bool;
    fn is_continuous_spell(self, duel: &Duel) -> bool;
    fn is_continuous_trap(self, duel: &Duel) -> bool;
    fn is_quick_play_spell(self, duel: &Duel) -> bool;
    fn is_synchro_monster(self, duel: &Duel) -> bool;
    fn is_non_effect_monster(self, duel: &Duel) -> bool;
    fn is_in_main_mzone(self, duel: &Duel, tp: Option<u8>) -> bool;
    fn is_sequence(self, duel: &Duel, sequence: u32) -> bool;
    fn has_level(self, duel: &Duel) -> bool;
    fn is_original_level(self, duel: &Duel, level: u32) -> bool;
    fn lists_code(self, duel: &Duel, code: u32) -> bool;
    fn is_negatable_monster(self, duel: &Duel) -> bool;
    fn is_negatable_spell_trap(self, duel: &Duel) -> bool;
    fn is_negatable(self, duel: &Duel) -> bool;
    fn add_must_be_special_summoned(self, duel: &mut Duel) -> EffectId;
    fn add_must_first_be_fusion_summoned(self, duel: &mut Duel) -> EffectId;

    fn negate_effects(self, duel: &mut Duel, c: CardId, reset: u32, negates_cards: bool);
    fn is_fusion_monster(self, duel: &Duel) -> bool;
    fn is_link_monster(self, duel: &Duel) -> bool;
    fn is_fusion_summoned(self, duel: &Duel) -> bool;
    fn is_synchro_summoned(self, duel: &Duel) -> bool;
    fn is_link_summoned(self, duel: &Duel) -> bool;
    fn is_xyz_summoned(self, duel: &Duel) -> bool;
    fn is_ritual_summoned(self, duel: &Duel) -> bool;
    fn has_flag_effect(self, duel: &Duel, id: u32) -> bool;
    fn has_flag_effect_full(self, duel: &Duel, id: u32, count: usize) -> bool;
    fn announce_another_attribute(self, duel: &mut Duel, tp: u8) -> Script<u32>;
    fn is_race_except(self, duel: &Duel, race: u64) -> bool;
    fn update_level(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>) -> i32;
    fn update_attack(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>, reset_count: Option<u16>) -> i32;
}

impl CardUtility for CardId {
    fn is_fusion_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_FUSION)
    }

    fn is_link_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_LINK)
    }

    fn is_fusion_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_FUSION])
    }

    fn is_synchro_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_SYNCHRO])
    }

    fn is_link_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_LINK])
    }

    fn is_xyz_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_XYZ])
    }

    fn is_ritual_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_RITUAL])
    }

    fn has_flag_effect(self, duel: &Duel, id: u32) -> bool {
        self.has_flag_effect_full(duel, id, 1)
    }

    fn has_flag_effect_full(self, duel: &Duel, id: u32, count: usize) -> bool {
        self.get_flag_effect(duel, id) >= count
    }

    fn announce_another_attribute(self, duel: &mut Duel, tp: u8) -> Script<u32> {
        announce_another_attribute(duel, self.get_attribute(duel), tp)
    }

    // Card.IsRaceExcept(c, race, scard, sumtype, playerid): the scard arguments only reach race effects that apply as
    // material.
    fn is_race_except(self, duel: &Duel, race: u64) -> bool {
        let own = self.get_race(duel);
        own & race != own
    }

    // Card.UpdateLevel(c, amt, reset, rc): `reset` None is RESET_EVENT with the standard resets (and RESET_DISABLE
    // when `rc` is the card itself, its default).
    fn update_level(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>) -> i32 {
        let c = self;
        let rc = rc.unwrap_or(c);
        let r = if c == rc { RESETS_STANDARD_DISABLE } else { RESETS_STANDARD };
        let reset = reset.unwrap_or(RESET_EVENT + r);
        let lv = c.get_level(duel) as i32;
        if !c.is_level_below(duel, 2_147_483_647) {
            return 0;
        }
        let amt = if lv + amt <= 0 { -(lv - 1) } else { amt };
        let e1 = duel.create_effect(rc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, amt);
        e1.set_reset(duel, reset);
        c.register_effect(duel, e1);
        c.get_level(duel) as i32 - lv
    }

    fn update_attack(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>, reset_count: Option<u16>) -> i32 {
        let c = self;
        let rc = rc.unwrap_or(c);
        let r = if c == rc { RESETS_STANDARD_DISABLE } else { RESETS_STANDARD };
        let reset = reset.unwrap_or(RESET_EVENT + r);
        let atk = c.get_attack(duel);
        let e1 = duel.create_effect(rc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, amt);
        e1.set_reset_full(duel, reset, reset_count.unwrap_or(1));
        c.register_effect(duel, e1);
        c.get_attack(duel) - atk
    }

    fn is_monster(self, duel: &Duel) -> bool {
        self.is_type(duel, TYPE_MONSTER)
    }

    fn is_spell(self, duel: &Duel) -> bool {
        self.is_type(duel, TYPE_SPELL)
    }

    fn is_trap(self, duel: &Duel) -> bool {
        self.is_type(duel, TYPE_TRAP)
    }

    fn is_spell_trap(self, duel: &Duel) -> bool {
        self.is_type(duel, TYPE_SPELL | TYPE_TRAP)
    }

    fn is_monster_card(self, duel: &Duel) -> bool {
        self.is_original_type(duel, TYPE_MONSTER)
    }

    fn is_owner(self, duel: &Duel, player: u8) -> bool {
        self.get_owner(duel) == player
    }

    fn is_field_spell(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_SPELL | TYPE_FIELD)
    }

    fn is_ritual_spell(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_SPELL | TYPE_RITUAL)
    }

    fn is_ritual_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_RITUAL)
    }

    fn is_continuous_spell(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_SPELL | TYPE_CONTINUOUS)
    }

    fn is_continuous_trap(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_TRAP | TYPE_CONTINUOUS)
    }

    fn is_quick_play_spell(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_SPELL | TYPE_QUICKPLAY)
    }

    fn is_synchro_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_SYNCHRO)
    }

    fn is_non_effect_monster(self, duel: &Duel) -> bool {
        self.is_monster(duel) && !self.is_type(duel, TYPE_EFFECT)
    }

    fn is_in_main_mzone(self, duel: &Duel, tp: Option<u8>) -> bool {
        self.is_location(duel, LOCATION_MMZONE) && tp.is_none_or(|tp| self.is_controler(duel, tp))
    }

    fn is_sequence(self, duel: &Duel, sequence: u32) -> bool {
        self.get_sequence(duel) == sequence
    }

    fn has_level(self, duel: &Duel) -> bool {
        if self.is_monster(duel) {
            let card_type = self.get_type(duel);
            return card_type & TYPE_LINK != TYPE_LINK
                && card_type & TYPE_XYZ != TYPE_XYZ
                && !(self.is_has_effect(duel, EFFECT_RANK_LEVEL).is_some() || self.is_has_effect(duel, EFFECT_RANK_LEVEL_S).is_some())
                && !self.is_status(duel, STATUS_NO_LEVEL);
        }
        if self.is_original_type(duel, TYPE_MONSTER) {
            return !(self.is_original_type(duel, TYPE_XYZ + TYPE_LINK) || self.is_status(duel, STATUS_NO_LEVEL));
        }
        false
    }

    fn is_original_level(self, duel: &Duel, level: u32) -> bool {
        self.has_level(duel) && self.get_original_level(duel) == level
    }

    fn lists_code(self, duel: &Duel, code: u32) -> bool {
        crate::official::listed_names(self.get_original_code(duel)).contains(&code)
    }

    fn is_negatable_monster(self, duel: &Duel) -> bool {
        self.is_faceup(duel)
            && !self.is_disabled(duel)
            && (!self.is_non_effect_monster(duel) || self.get_original_type(duel) & TYPE_EFFECT != 0)
    }

    fn is_negatable_spell_trap(self, duel: &Duel) -> bool {
        self.is_faceup(duel) && !self.is_disabled(duel) && self.is_spell_trap(duel)
    }

    fn is_negatable(self, duel: &Duel) -> bool {
        self.is_negatable_monster(duel) || self.is_negatable_spell_trap(duel)
    }

    fn add_must_be_special_summoned(self, duel: &mut Duel) -> EffectId {
        let e0 = duel.create_effect(self);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
        self.register_effect(duel, e0);
        e0
    }

    fn add_must_first_be_fusion_summoned(self, duel: &mut Duel) -> EffectId {
        let e0 = duel.create_effect(self);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
        e0.set_value(
            duel,
            Value::SpSummonCondition(|duel, e, _, _, sum_type, _, _| {
                !e.get_handler(duel).is_location(duel, LOCATION_EXTRA) || sum_type & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
            }),
        );
        self.register_effect(duel, e0);
        e0
    }

    // Card.NegateEffects(tc, c, reset, negates_cards) with ct 1.
    fn negate_effects(self, duel: &mut Duel, c: CardId, reset: u32, negates_cards: bool) {
        let tc = self;
        let mut reset = reset | (RESET_EVENT | RESETS_STANDARD);
        let trap_monster_chk = negates_cards && tc.is_type(duel, TYPE_TRAPMONSTER);
        if trap_monster_chk {
            reset &= !(RESET_TOFIELD | RESET_LEAVE | RESET_TURN_SET);
        }
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, reset);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        tc.register_effect(duel, e2);
        if trap_monster_chk {
            let e3 = e1.clone_effect(duel);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            tc.register_effect(duel, e3);
        }
    }
}

pub trait EffectUtility: Copy {
    fn is_monster_effect(self, duel: &Duel) -> bool;
    fn is_spell_effect(self, duel: &Duel) -> bool;
    fn is_spell_trap_effect(self, duel: &Duel) -> bool;
}

impl EffectUtility for EffectId {
    fn is_spell_trap_effect(self, duel: &Duel) -> bool {
        self.is_active_type(duel, TYPE_SPELL | TYPE_TRAP)
    }

    fn is_monster_effect(self, duel: &Duel) -> bool {
        self.is_active_type(duel, TYPE_MONSTER)
    }

    fn is_spell_effect(self, duel: &Duel) -> bool {
        self.is_active_type(duel, TYPE_SPELL)
    }
}

// Card.AnnounceAnotherAttribute and Duel.AnnounceAnotherAttribute: any attribute but the one of `attribute`, if it is single.
fn announce_another_attribute(duel: &mut Duel, attribute: u32, tp: u8) -> Script<u32> {
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTRIBUTE);
    let available = if attribute & attribute.wrapping_sub(1) == 0 {
        !attribute & ATTRIBUTE_ALL
    } else {
        ATTRIBUTE_ALL
    };
    duel.announce_attribute(tp, 1, available)
}

pub trait DuelUtility {
    fn select_effect(&mut self, tp: u8, effects: &[(bool, u64)]) -> Script<Option<usize>>;
    fn has_flag_effect(&self, tp: u8, id: u32) -> bool;
    fn has_flag_effect_full(&self, tp: u8, id: u32, ct: usize) -> bool;
    fn get_target_cards(&self, e: EffectId) -> Option<CardSet>;
    fn announce_another_attribute(&mut self, g: CardSet, tp: u8) -> Script<u32>;
    fn move_to_deck_bottom_count(&mut self, count: usize, tp: u8) -> Script<()>;
    fn move_to_deck_bottom(&mut self, targets: CardSet) -> Script<()>;
    fn place_field_spell(&mut self, c: CardId, tp: u8) -> Script<bool>;
}

impl DuelUtility for Duel {
    // Duel.GetTargetCards: the chain link's targets still related to `e`, nil without targets.
    fn get_target_cards(&self, e: EffectId) -> Option<CardSet> {
        let tg = self.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)?;
        Some(tg.filter(self, |c, duel| c.is_relate_to_effect(duel, e), None))
    }

    // The 1-based index of the effect chosen among those whose condition holds, None if none does.
    fn select_effect(&mut self, tp: u8, effects: &[(bool, u64)]) -> Script<Option<usize>> {
        let (sel, eff): (Vec<usize>, Vec<u64>) = effects
            .iter()
            .enumerate()
            .filter(|(_, (available, _))| *available)
            .map(|(index, &(_, description))| (index + 1, description))
            .unzip();
        if eff.is_empty() {
            return Ok(None);
        }
        let option = self.select_option(tp, &eff)?;
        Ok(usize::try_from(option).ok().and_then(|option| sel.get(option)).copied())
    }

    fn has_flag_effect(&self, tp: u8, id: u32) -> bool {
        self.has_flag_effect_full(tp, id, 1)
    }

    fn has_flag_effect_full(&self, tp: u8, id: u32, ct: usize) -> bool {
        self.get_flag_effect(tp, id) >= ct
    }

    // Duel.PlaceFieldSpell(c, tp, target_p) with target_p left out.
    fn place_field_spell(&mut self, c: CardId, tp: u8) -> Script<bool> {
        let fc = self.get_field_card(tp, LOCATION_FZONE, 0);
        if self.is_duel_type(DUEL_1_FIELD) {
            if let Some(fc) = fc {
                self.destroy(fc, REASON_RULE)?;
            }
        } else if let Some(fc) = fc
            && self.send_to_grave(fc, REASON_RULE)? == 0
        {
            self.send_to_grave(c, REASON_RULE)?;
            return Ok(false);
        }
        self.break_effect()?;
        let res = self.move_to_field(c, tp, tp, LOCATION_FZONE, POS_FACEUP, true)?;
        if res
            && self.is_duel_type(DUEL_1_FIELD)
            && let Some(fc) = self.get_field_card(1 - tp, LOCATION_FZONE, 0)
            && self.destroy(fc, REASON_RULE)? == 0
        {
            self.send_to_grave(c, REASON_RULE)?;
            return Ok(false);
        }
        Ok(res)
    }

    // Duel.MoveToDeckBottom(count, tp): the top card of the Deck, `count` times.
    fn move_to_deck_bottom_count(&mut self, count: usize, tp: u8) -> Script<()> {
        for _ in 0..count {
            if let Some(top) = self.get_decktop_group(tp, 1).get_first() {
                self.move_sequence(top, SEQ_DECKBOTTOM as u8)?;
            }
        }
        Ok(())
    }

    // Duel.MoveToDeckBottom(group): the cards in the Deck, in group order.
    fn move_to_deck_bottom(&mut self, targets: CardSet) -> Script<()> {
        for c in targets.filter(self, |c, duel| c.is_location(duel, LOCATION_DECK), None) {
            self.move_sequence(c, SEQ_DECKBOTTOM as u8)?;
        }
        Ok(())
    }

    fn announce_another_attribute(&mut self, g: CardSet, tp: u8) -> Script<u32> {
        let attribute = g.get_bitwise_or(self, |c, duel| u64::from(c.get_attribute(duel))) as u32;
        announce_another_attribute(self, attribute, tp)
    }
}
