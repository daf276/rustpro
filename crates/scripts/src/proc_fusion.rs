// Port of proc_fusion.lua and proc_fusion_spell.lua; AddProcMixRepUnfix and matcheck outside AddProcMix have no caller.
use std::cell::Cell;

use engine::{Args, CardId, CardSet, ConditionFn, Duel, EffectId, FusionCheck, FusionMaterialFn, Operation, OperationFn, Script, Value};

use crate::constant::*;
use crate::keys;
use crate::proc_workaround::CardWorkaround;
use crate::utility::{CardUtility, aux};

pub struct Fusion;

pub type MaterialFilter = fn(CardId, &Duel, CardId, u64, u8) -> bool;

// A filter that reads sg, the materials chosen with c (nil in the first pass, as the Lua's globals are).
pub type GroupMaterialFilter = fn(CardId, &Duel, CardId, u64, u8, Option<CardSet>) -> bool;

// A filter that reads contact, whether this is a contact fusion.
pub type ContactMaterialFilter = fn(CardId, &Duel, CardId, u64, u8, bool) -> bool;

// matcheck(c, fc, tp, contact, sumtype), the value a card sets on the effect AddProcMix returns.
pub type MatCheck = fn(&Duel, CardId, CardId, u8, bool, u64) -> bool;

// preselect(e, tc): false stops the summon.
pub type Preselect = fn(&mut Duel, EffectId, CardId) -> Script<bool>;

#[derive(Debug, Clone, Copy)]
pub enum FusionMaterial {
    Code(u32),
    Filter(MaterialFilter),
    GroupFilter(GroupMaterialFilter),
    ContactFilter(ContactMaterialFilter),
    // A Lua table of codes and filters, any of which fits (Fusion.ParseMaterialTable).
    Table(&'static [FusionMaterial]),
}

// Fusion.AddProcMix and the procedures that call it (AddProcMixN, AddProcCode2, ...), with each material written out.
pub trait FusionMix {
    const SUB: bool;
    const INSF: bool;
    const MATERIALS: &'static [FusionMaterial];
    const MATCHECK: Option<MatCheck> = None;
}

pub trait FusionMixRep {
    const SUB: bool;
    const INSF: bool;
    const FUN1: FusionMaterial;
    const MINC: i32;
    const MAXC: i32;
    const MATERIALS: &'static [FusionMaterial] = &[];
}

// Fusion.AddContactProc: sumcon None for nil or false, Value::Int(0) for true; summonToPlayer and summonToZones unused.
pub trait ContactFusion {
    const SUMCON: Option<Value> = None;
    const CONDITION: Option<fn(&Duel, u8) -> bool> = None;
    const SUMTYPE: Option<u32> = None;
    const DESC: u64 = 2;
    const CANNOT_BE_LIZARD: bool = true;

    fn group(duel: &Duel, tp: u8) -> CardSet;

    fn op(duel: &mut Duel, g: CardSet, tp: u8, c: CardId) -> Script<()>;
}

// The named parameters of Fusion.CreateSummonEff, SummonEffTG and SummonEffOP; nil ones keep the defaults below.
pub trait FusionSummonParams {
    const DESC: u64 = 1170;
    const EXACTCOUNT: Option<usize> = None;
    const MINCOUNT: Option<usize> = None;
    const MAXCOUNT: Option<usize> = None;
    const VALUE: u64 = 0;
    const LOCATION: u32 = LOCATION_EXTRA;
    const CHKF: Option<u64> = None;
    const NOSUMMONCHECK: bool = false;
    const SUMPOS: u32 = POS_FACEUP;
    const PRESELECT: Option<Preselect> = None;
    // extraop(e, tc, tp, sg), a constant since SummonEffOP compares it with an EFFECT_EXTRA_FUSION_MATERIAL operation.
    const EXTRAOP: Option<FusionMaterialFn> = None;

    fn fusfilter(_c: CardId, _duel: &Duel) -> bool {
        true
    }

    // matfilter(c, e, tp, chk); nil is aux.LimitParams(Card.IsAbleToGrave).
    fn matfilter(c: CardId, duel: &Duel, _e: EffectId, _tp: u8) -> bool {
        c.is_able_to_grave(duel)
    }

    // extrafil(e, tp, mg): the extra materials and the additional check.
    fn extrafil(_duel: &Duel, _e: EffectId, _tp: u8, _mg: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (None, None)
    }

    // gc(e, tp, ...): the materials that must be used.
    fn gc(_duel: &Duel, _e: EffectId, _tp: u8) -> Option<CardSet> {
        None
    }

    fn stage2(_duel: &mut Duel, _e: EffectId, _tc: Option<CardId>, _tp: u8, _mg: Option<CardSet>, _chk: i32) -> Script<()> {
        Ok(())
    }

    fn extratg(_duel: &mut Duel, _a: &Args, _chk: i32) -> Script<bool> {
        Ok(true)
    }
}

// Effect codes of unofficial cards the procedures look up.
const EFFECT_ANY_MATERIAL: u32 = 511002961;
const EFFECT_NOT_MATERIAL: u32 = 6205579;
const EFFECT_OPPONENT_SELECTS: u32 = 511004008;

// Marks a chkf upvalue ([keys::fusion_chkf]) as set.
const SET: i64 = 1 << 40;

fn check(duel: &Duel, key: u64) -> Option<usize> {
    usize::try_from(duel.get_script_state(key)).ok().filter(|&count| count != 0)
}

fn set_check(duel: &mut Duel, key: u64, count: Option<usize>) {
    duel.set_script_state(key, count.map_or(0, |count| count as i64));
}

fn set_checks(duel: &mut Duel, counts: [Option<usize>; 3]) {
    [keys::FUSION_CHECK_EXACT, keys::FUSION_CHECK_MIN, keys::FUSION_CHECK_MAX]
        .into_iter()
        .zip(counts)
        .for_each(|(key, count)| set_check(duel, key, count));
}

fn check_additional(duel: &Duel, tp: u8, sg: CardSet, fc: CardId) -> bool {
    duel.get_fusion_check_additional()
        .is_none_or(|(check, c)| check(duel, c, tp, sg, fc))
}

impl FusionMaterial {
    // The codes mt.material lists for it.
    fn codes(self) -> Vec<u32> {
        match self {
            Self::Code(code) => vec![code],
            Self::Table(materials) => materials.iter().flat_map(|material| material.codes()).collect(),
            Self::Filter(_) | Self::GroupFilter(_) | Self::ContactFilter(_) => Vec::new(),
        }
    }

    // The material function the procedures build: fun(c, fc, sub, sub2, mg, sg, tp, contact, sumtype).
    fn check(self, duel: &Duel, c: CardId, s: Proc, sub: bool, sub2: bool, sg: Option<CardSet>) -> bool {
        let Proc {
            fc, tp, sumtype, contact, ..
        } = s;
        let any = |duel| sub2 && c.is_has_effect(duel, EFFECT_ANY_MATERIAL).is_some();
        let usable = |duel| c.is_has_effect(duel, EFFECT_NOT_MATERIAL).is_none();
        let passes = |material: Self| match material {
            Self::Filter(f) => f(c, duel, fc, sumtype, tp),
            Self::GroupFilter(f) => f(c, duel, fc, sumtype, tp, sg),
            Self::ContactFilter(f) => f(c, duel, fc, sumtype, tp, contact),
            Self::Code(_) | Self::Table(_) => false,
        };
        let by_codes = |codes: &[u32]| {
            c.is_summon_code(duel, Some(fc), sumtype, fc.get_controler(duel), codes)
                || (sub && c.check_fusion_substitute(duel, fc))
                || any(duel)
        };
        match self {
            Self::Filter(_) | Self::GroupFilter(_) | Self::ContactFilter(_) => (passes(self) || any(duel)) && usable(duel),
            Self::Code(code) => by_codes(&[code]),
            Self::Table(materials) => {
                let by_filter = |&material: &Self| passes(material) || (!matches!(material, Self::Code(_)) && any(duel));
                let codes = self.codes();
                (materials.iter().any(by_filter) && usable(duel)) || (!codes.is_empty() && by_codes(&codes))
            }
        }
    }
}

// What the procedures read from chkfnf and the procedure.
#[derive(Clone, Copy)]
struct Proc {
    fc: CardId,
    tp: u8,
    chkf: u8,
    zone: u64,
    contact: bool,
    cancelable: bool,
    sumtype: u64,
}

impl Proc {
    // Also returns sub as the condition and operation narrow it.
    fn new(duel: &Duel, e: EffectId, chkfnf: u64, sub: bool) -> (Self, bool) {
        let fc = e.get_handler(duel);
        let notfusion = chkfnf & FUSPROC_NOTFUSION != 0;
        let contact = chkfnf & FUSPROC_CONTACTFUS != 0;
        let listedmats = chkfnf & FUSPROC_LISTEDMATS != 0;
        let sumtype = if listedmats {
            0
        } else if contact || notfusion {
            MATERIAL_FUSION
        } else {
            u64::from(SUMMON_TYPE_FUSION) | MATERIAL_FUSION
        };
        let proc = Self {
            fc,
            tp: fc.get_controler(duel),
            chkf: chkfnf as u8,
            zone: match (chkfnf >> 40) & 0xff {
                0 => 0xff,
                zone => zone,
            },
            contact,
            cancelable: chkfnf & (FUSPROC_CONTACTFUS | FUSPROC_CANCELABLE) != 0,
            sumtype,
        };
        (proc, !listedmats && (sub || notfusion) && !contact)
    }

    // Fusion.ConditionFilterMix.
    fn condition_filter(
        self,
        duel: &Duel,
        c: CardId,
        sub: bool,
        matcheck: Option<MatCheck>,
        funs: impl IntoIterator<Item = FusionMaterial>,
    ) -> bool {
        matcheck.is_none_or(|matcheck| matcheck(duel, c, self.fc, self.tp, self.contact, self.sumtype))
            && c.is_can_be_fusion_material(duel, Some(self.fc), self.sumtype)
            && funs.into_iter().any(|f| f.check(duel, c, self, sub, sub, None))
    }

    fn cannot_be_material(self, duel: &Duel, g: CardSet) -> bool {
        g.is_exists(
            duel,
            |c, duel| !c.is_can_be_fusion_material(duel, Some(self.fc), self.sumtype),
            1,
            None,
        )
    }

    fn must_materials(
        self,
        duel: &Duel,
        mg: CardSet,
        mut mustg: CardSet,
        gc: Option<CardSet>,
        filter: impl Fn(CardId, &Duel) -> bool,
    ) -> Option<(CardSet, CardSet)> {
        if !mg.includes(mustg) || self.cannot_be_material(duel, mustg) {
            return None;
        }
        if let Some(gc) = gc {
            if self.cannot_be_material(duel, gc) || gc.is_exists(duel, |mc, duel| !filter(mc, duel), 1, None) {
                return None;
            }
            mustg.merge(gc);
        }
        Some((mg.union(mustg), mustg))
    }

    // The field check of CheckMixGoal (`mzone`) and of the MixRep goals, which ask the Extra Deck count only.
    fn location_count(self, duel: &Duel, sg: CardSet, mzone: bool) -> bool {
        if self.chkf == PLAYER_NONE {
            return true;
        }
        unported!(duel, self.zone != 0xff);
        let count = match mzone && !self.fc.is_location(duel, LOCATION_EXTRA) {
            true => duel.get_mzone_count(self.chkf, Some(sg)),
            false => duel.get_location_count_from_ex(self.chkf, self.tp, Some(sg), self.fc),
        };
        count > 0
    }

    // The EFFECT_FUSION_MAT_RESTRICTION checks of SelectMix and SelectMixRep.
    fn restrictions(self, duel: &Duel, c: CardId, sg: CardSet) {
        if self.contact {
            return;
        }
        let restricted = |c: CardId, duel: &Duel| c.is_has_effect(duel, EFFECT_FUSION_MAT_RESTRICTION).is_some();
        unported!(duel, restricted(c, duel));
        unported!(duel, sg.is_exists(duel, restricted, 1, None));
    }

    // The player who selects the materials, the opponent under unofficial effect 511004008.
    fn selecting_player(self, duel: &Duel) -> u8 {
        unported!(
            duel,
            !self.contact && duel.is_player_affected_by_effect(self.tp, EFFECT_OPPONENT_SELECTS).is_some()
        );
        self.tp
    }

    // Toggles tc in the selection sg unless it is one of the materials that must be used.
    fn toggle(sg: &mut CardSet, mustg: CardSet, tc: CardId) {
        if !mustg.is_empty() && mustg.is_contains(tc) {
            return;
        }
        match sg.is_contains(tc) {
            false => sg.insert(tc),
            true => sg.remove(tc),
        }
    }
}

impl Fusion {
    fn register_material_effect(duel: &mut Duel, c: CardId, condition: ConditionFn, operation: OperationFn) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_code(duel, EFFECT_FUSION_MATERIAL);
        e1.set_condition(duel, condition);
        e1.set_operation(duel, operation);
        c.register_effect(duel, e1);
        e1
    }

    // mt.material of a Fusion Monster: the distinct codes its Fusion.AddProcMix* call named, empty for nil.
    pub fn material(duel: &Duel, code: u32) -> Vec<u32> {
        (0..)
            .map_while(|i| {
                u32::try_from(duel.get_script_state(keys::fusion_material(code, i)))
                    .ok()
                    .filter(|&code| code != 0)
            })
            .collect()
    }

    fn set_material(duel: &mut Duel, c: CardId, materials: impl Iterator<Item = FusionMaterial>) {
        let original = c.get_original_code(duel);
        if duel.get_script_state(keys::fusion_material(original, 0)) != 0 {
            return;
        }
        let mut codes = Vec::new();
        for code in materials.flat_map(FusionMaterial::codes) {
            if !codes.contains(&code) {
                codes.push(code);
            }
        }
        for (i, code) in codes.into_iter().enumerate() {
            duel.set_script_state(keys::fusion_material(original, i as u64), i64::from(code));
        }
    }

    pub fn add_proc_mix<M: FusionMix>(duel: &mut Duel, c: CardId) -> Option<EffectId> {
        if c.is_status(duel, STATUS_COPYING_EFFECT) {
            return None;
        }
        Fusion::set_material(duel, c, M::MATERIALS.iter().copied());
        Some(Fusion::register_material_effect(
            duel,
            c,
            Fusion::condition_mix::<M>,
            Fusion::operation_mix::<M>,
        ))
    }

    pub fn add_proc_mix_n<M: FusionMix>(duel: &mut Duel, c: CardId) -> Option<EffectId> {
        Fusion::add_proc_mix::<M>(duel, c)
    }

    // Fusion.AddProcFun2(c, f1, f2, insf), with SUB false.
    pub fn add_proc_fun2<M: FusionMix>(duel: &mut Duel, c: CardId) -> Option<EffectId> {
        Fusion::add_proc_mix::<M>(duel, c)
    }

    pub fn add_proc_mix_rep<M: FusionMixRep>(duel: &mut Duel, c: CardId) -> Option<EffectId> {
        if c.is_status(duel, STATUS_COPYING_EFFECT) {
            return None;
        }
        Fusion::set_material(duel, c, std::iter::once(M::FUN1).chain(M::MATERIALS.iter().copied()));
        Some(Fusion::register_material_effect(
            duel,
            c,
            Fusion::condition_mix_rep::<M>,
            Fusion::operation_mix_rep::<M>,
        ))
    }

    fn condition_mix<M: FusionMix>(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        let tp = c.get_controler(duel);
        let Some(g) = a.materials else {
            let mustg = aux::get_must_be_material_group(duel, tp, None, tp, Some(c), None, REASON_FUSION);
            return M::INSF && mustg.is_empty();
        };
        let (s, sub) = Proc::new(duel, a.e, a.chkf, M::SUB);
        let funs = M::MATERIALS;
        let filter = |mc, duel: &Duel| s.condition_filter(duel, mc, sub, M::MATCHECK, funs.iter().copied());
        let mg = g.filter(duel, filter, Some(c.into()));
        let mut mustg = aux::get_must_be_material_group(duel, tp, Some(g), tp, Some(c), Some(mg), REASON_FUSION);
        if s.contact {
            mustg.clear();
        }
        let Some((mg, mustg)) = s.must_materials(duel, mg, mustg, a.must, filter) else {
            return false;
        };
        let sg = CardSet::default();
        mg.is_exists(
            duel,
            |mc, duel| Fusion::select_mix(duel, mc, s, mg, sg, mustg, sub, sub, funs),
            1,
            None,
        )
    }

    fn operation_mix<M: FusionMix>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (s, sub) = Proc::new(duel, a.e, a.chkf, M::SUB);
        let (c, tp, funs) = (s.fc, s.tp, M::MATERIALS);
        let eg = a.eg.unwrap_or_default();
        let mg = eg.filter(
            duel,
            |mc, duel| s.condition_filter(duel, mc, sub, M::MATCHECK, funs.iter().copied()),
            Some(c.into()),
        );
        let mut mustg = aux::get_must_be_material_group(duel, tp, Some(eg), tp, Some(c), Some(mg), REASON_FUSION);
        if s.contact {
            mustg.clear();
        }
        let mut sg = CardSet::default();
        if let Some(gc) = a.must {
            mustg.merge(gc);
        }
        for tc in mustg {
            sg.insert(tc);
            unported!(duel, !s.contact && tc.is_has_effect(duel, EFFECT_FUSION_MAT_RESTRICTION).is_some());
        }
        let p = s.selecting_player(duel);
        let count = funs.len() as u8;
        while sg.len() < funs.len() {
            duel.hint(HINT_SELECTMSG, p, HINTMSG_FMATERIAL);
            let mustg_left = mustg.difference(sg);
            let cg = mg.filter(
                duel,
                |mc, duel| Fusion::select_mix(duel, mc, s, mg, sg, mustg_left, sub, sub, funs),
                Some(sg),
            );
            let Some(tc) = cg.select_unselect(duel, Some(sg), p, false, s.cancelable && sg.is_empty(), count, count)? else {
                break;
            };
            Proc::toggle(&mut sg, mustg, tc);
        }
        duel.set_fusion_material(sg);
        Ok(())
    }

    // Fusion.CheckMix: whether c, then the rest of mg, fill funs in order.
    fn check_mix(duel: &Duel, c: CardId, mg: CardSet, sg: CardSet, s: Proc, sub: bool, sub2: bool, funs: &[FusionMaterial]) -> bool {
        let [fun1, rest @ ..] = funs else { return false };
        if rest.is_empty() {
            return fun1.check(duel, c, s, sub, sub2, Some(sg));
        }
        let sg = sg.union(c.into());
        let next = |sub| {
            mg.is_exists(
                duel,
                |tc, duel| Fusion::check_mix(duel, tc, mg, sg, s, sub, sub2, rest),
                1,
                Some(sg),
            )
        };
        if fun1.check(duel, c, s, false, sub2, Some(sg)) {
            next(sub)
        } else if sub && fun1.check(duel, c, s, true, sub2, Some(sg)) {
            next(false)
        } else {
            false
        }
    }

    fn check_mix_goal(duel: &Duel, sg: CardSet, s: Proc, sub: bool, sub2: bool, funs: &[FusionMaterial]) -> bool {
        let g = CardSet::default();
        sg.is_exists(duel, |c, duel| Fusion::check_mix(duel, c, sg, g, s, sub, sub2, funs), 1, None)
            && s.location_count(duel, sg, true)
            && check_additional(duel, s.tp, sg, s.fc)
    }

    fn select_mix(
        duel: &Duel,
        c: CardId,
        s: Proc,
        mg: CardSet,
        sg: CardSet,
        mustg: CardSet,
        sub: bool,
        sub2: bool,
        funs: &[FusionMaterial],
    ) -> bool {
        let (totalcount, mustgcount) = (funs.len(), mustg.len());
        if mustgcount > totalcount
            || check(duel, keys::FUSION_CHECK_EXACT).is_some_and(|exact| exact != totalcount || mustgcount > exact)
            || check(duel, keys::FUSION_CHECK_MAX).is_some_and(|max| max < totalcount || mustgcount > max)
            || check(duel, keys::FUSION_CHECK_MIN).is_some_and(|min| min > totalcount)
        {
            return false;
        }
        s.restrictions(duel, c, sg);
        let sg = sg.union(c.into());
        if sg.len() < totalcount {
            let mustg = mustg.difference(sg);
            mg.is_exists(
                duel,
                |tc, duel| Fusion::select_mix(duel, tc, s, mg, sg, mustg, sub, sub2, funs),
                1,
                Some(sg),
            )
        } else {
            Fusion::check_mix_goal(duel, sg, s, sub, sub2, funs) && sg.includes(mustg)
        }
    }
}

// The bits of FusionMixRep::MATERIALS still to fill: the Lua's varargs after table.remove, in their order.
type Rest = u8;

fn rest_iter<M: FusionMixRep>(rest: Rest) -> impl Iterator<Item = (usize, FusionMaterial)> {
    M::MATERIALS.iter().copied().enumerate().filter(move |&(i, _)| rest & (1 << i) != 0)
}

// The groups the MixRep selection checks share.
#[derive(Clone, Copy)]
struct Search {
    proc: Proc,
    mg: CardSet,
    sg: CardSet,
    mustg: CardSet,
}

// The state the MixRep recursion carries: substitutes still allowed, fun1 counts still possible, fixed materials left.
#[derive(Clone, Copy)]
struct Rep {
    sub: bool,
    sub2: bool,
    minc: i32,
    maxc: i32,
    rest: Rest,
}

impl Rep {
    fn fixed(self) -> i32 {
        self.rest.count_ones() as i32
    }

    // A card given to fixed material i, or to fun1 when i is None; sub stays only if the card also passes without it.
    fn after(self, duel: &Duel, s: Proc, c: CardId, i: Option<usize>, f: FusionMaterial, g: CardSet) -> Self {
        let sub = self.sub && f.check(duel, c, s, false, self.sub2, Some(g));
        match i {
            Some(i) => Self {
                sub,
                rest: self.rest & !(1 << i),
                ..self
            },
            None => Self {
                sub,
                minc: self.minc - 1,
                maxc: self.maxc - 1,
                ..self
            },
        }
    }

    fn passes(self, duel: &Duel, s: Proc, c: CardId, f: FusionMaterial, g: CardSet) -> bool {
        f.check(duel, c, s, self.sub, self.sub2, Some(g))
    }
}

impl Fusion {
    fn rep_funs<M: FusionMixRep>() -> impl Iterator<Item = FusionMaterial> {
        std::iter::once(M::FUN1).chain(M::MATERIALS.iter().copied())
    }

    fn rep<M: FusionMixRep>(sub: bool) -> Rep {
        Rep {
            sub,
            sub2: sub,
            minc: M::MINC,
            maxc: M::MAXC,
            rest: ((1u16 << M::MATERIALS.len()) - 1) as Rest,
        }
    }

    fn condition_mix_rep<M: FusionMixRep>(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        let tp = c.get_controler(duel);
        let Some(g) = a.materials else {
            return M::INSF && aux::get_must_be_material_group(duel, tp, None, tp, Some(c), None, REASON_FUSION).is_empty();
        };
        let (s, sub) = Proc::new(duel, a.e, a.chkf, M::SUB);
        // The Lua passes the globals eg and mg here, which are nil.
        let mut mustg = aux::get_must_be_material_group(duel, tp, None, tp, Some(c), None, REASON_FUSION);
        if s.contact {
            mustg.clear();
        }
        let filter = |mc, duel: &Duel| s.condition_filter(duel, mc, sub, None, Fusion::rep_funs::<M>());
        let mg = g.filter(duel, filter, Some(c.into()));
        let Some((mg, mustg)) = s.must_materials(duel, mg, mustg, a.must, filter) else {
            return false;
        };
        let search = Search {
            proc: s,
            mg,
            sg: CardSet::default(),
            mustg,
        };
        mg.is_exists(
            duel,
            |mc, duel| Fusion::select_mix_rep::<M>(duel, mc, search, Fusion::rep::<M>(sub)),
            1,
            None,
        )
    }

    fn operation_mix_rep<M: FusionMixRep>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (s, sub) = Proc::new(duel, a.e, a.chkf, M::SUB);
        let (c, tp) = (s.fc, s.tp);
        let mut sg = CardSet::default();
        let eg = a.eg.unwrap_or_default();
        let mg = eg.filter(
            duel,
            |mc, duel| s.condition_filter(duel, mc, sub, None, Fusion::rep_funs::<M>()),
            Some(c.into()),
        );
        let mut mustg = aux::get_must_be_material_group(duel, tp, Some(eg), tp, Some(c), Some(mg), REASON_FUSION);
        if s.contact {
            mustg.clear();
        }
        if !mg.includes(mustg) || s.cannot_be_material(duel, mustg) {
            return Ok(());
        }
        if let Some(gc) = a.must {
            mustg.merge(gc);
        }
        sg.merge(mustg);
        let p = s.selecting_player(duel);
        let rep = Fusion::rep::<M>(sub);
        while sg.len() < M::MAXC as usize + M::MATERIALS.len() {
            let search = Search { proc: s, mg, sg, mustg };
            let cg = mg.filter(duel, |mc, duel| Fusion::select_mix_rep::<M>(duel, mc, search, rep), Some(sg));
            if cg.is_empty() {
                break;
            }
            let finish = Fusion::check_mix_rep_goal::<M>(duel, sg, s, rep)
                && check(duel, keys::FUSION_CHECK_EXACT).is_none()
                && !check(duel, keys::FUSION_CHECK_MIN).is_some_and(|min| sg.len() < min);
            let cancel = s.cancelable && sg.is_empty();
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_FMATERIAL);
            let Some(tc) = cg.select_unselect(duel, Some(sg), p, finish, cancel, 1, 1)? else {
                break;
            };
            Proc::toggle(&mut sg, mustg, tc);
        }
        duel.set_fusion_material(sg);
        Ok(())
    }

    // Fusion.CheckMixRep; g holds the cards given to fixed materials so far.
    fn check_mix_rep<M: FusionMixRep>(duel: &Duel, sg: CardSet, g: CardSet, s: Proc, rep: Rep) -> bool {
        if let Some((i, fun2)) = rest_iter::<M>(rep.rest).next() {
            let filter = |c: CardId, duel: &Duel| {
                let g = g.union(c.into());
                rep.passes(duel, s, c, fun2, g) && Fusion::check_mix_rep::<M>(duel, sg, g, s, rep.after(duel, s, c, Some(i), fun2, g))
            };
            return sg.is_exists(duel, filter, 1, Some(g));
        }
        let ct1 = sg.filter_count(duel, |c, duel| rep.passes(duel, s, c, M::FUN1, sg), Some(g));
        let ct2 = sg.filter_count(duel, |c, duel| Rep { sub: false, ..rep }.passes(duel, s, c, M::FUN1, sg), Some(g));
        ct1 + g.len() == sg.len() && ct1 <= ct2 + 1
    }

    fn check_mix_rep_goal<M: FusionMixRep>(duel: &Duel, sg: CardSet, s: Proc, rep: Rep) -> bool {
        let count = sg.len() as i32;
        if count < rep.minc + rep.fixed() || count > rep.maxc + rep.fixed() {
            return false;
        }
        Fusion::check_mix_rep::<M>(duel, sg, CardSet::default(), s, rep)
            && s.location_count(duel, sg, false)
            && check_additional(duel, s.tp, sg, s.fc)
    }

    // Fusion.CheckMixRepTemplate with cond CheckMixRepSelectedCond when `selected`, else CheckSelectMixRep.
    fn check_mix_rep_template<M: FusionMixRep>(duel: &Duel, c: CardId, selected: bool, search: Search, g: CardSet, rep: Rep) -> bool {
        let s = search.proc;
        let search = Search {
            mustg: search.mustg.difference(c.into()),
            ..search
        };
        let g = g.union(c.into());
        let cond = |rep| match selected {
            true => Fusion::check_mix_rep_selected_cond::<M>(duel, search, g, rep),
            false => Fusion::check_select_mix_rep::<M>(duel, search, g, rep),
        };
        let fixed = rest_iter::<M>(rep.rest).any(|(i, f)| rep.passes(duel, s, c, f, g) && cond(rep.after(duel, s, c, Some(i), f, g)));
        fixed || (rep.maxc > 0 && rep.passes(duel, s, c, M::FUN1, g) && cond(rep.after(duel, s, c, None, M::FUN1, g)))
    }

    fn check_mix_rep_selected_cond<M: FusionMixRep>(duel: &Duel, search: Search, g: CardSet, rep: Rep) -> bool {
        if g.len() < search.sg.len() {
            let selected = |c, duel: &Duel| Fusion::check_mix_rep_template::<M>(duel, c, true, search, g, rep);
            search.sg.is_exists(duel, selected, 1, Some(g))
        } else {
            Fusion::check_select_mix_rep::<M>(duel, search, g, rep)
        }
    }

    fn check_select_mix_rep<M: FusionMixRep>(duel: &Duel, search: Search, g: CardSet, rep: Rep) -> bool {
        let s = search.proc;
        if !check_additional(duel, s.tp, g, s.fc) {
            return false;
        }
        if s.location_count(duel, g, false) {
            if rep.minc <= 0 && rep.rest == 0 && g.includes(search.mustg) {
                return true;
            }
            let all = |c, duel: &Duel| Fusion::check_select_mix_rep_all::<M>(duel, c, search, g, rep);
            search.mg.is_exists(duel, all, 1, Some(g))
        } else {
            let in_mzone = |c: CardId, duel: &Duel| {
                c.is_controler(duel, s.tp)
                    && c.is_location(duel, LOCATION_MZONE)
                    && Fusion::check_mix_rep_template::<M>(duel, c, false, search, g, rep)
            };
            search.mg.is_exists(duel, in_mzone, 1, Some(g))
        }
    }

    fn check_select_mix_rep_all<M: FusionMixRep>(duel: &Duel, c: CardId, search: Search, g: CardSet, rep: Rep) -> bool {
        let s = search.proc;
        let g = g.union(c.into());
        if let Some((i, fun2)) = rest_iter::<M>(rep.rest).next() {
            return rep.passes(duel, s, c, fun2, g)
                && Fusion::check_select_mix_rep::<M>(duel, search, g, rep.after(duel, s, c, Some(i), fun2, g));
        }
        rep.maxc > 0
            && rep.passes(duel, s, c, M::FUN1, g)
            && Fusion::check_select_mix_rep::<M>(duel, search, g, rep.after(duel, s, c, None, M::FUN1, g))
    }

    fn select_mix_rep<M: FusionMixRep>(duel: &Duel, c: CardId, search: Search, rep: Rep) -> bool {
        let (s, mustgcount) = (search.proc, search.mustg.len() as i32);
        let (totalcount, mut rep) = (rep.fixed(), rep);
        if let Some(exact) = check(duel, keys::FUSION_CHECK_EXACT).map(|exact| exact as i32) {
            if exact < rep.minc + totalcount || mustgcount > exact {
                return false;
            }
            (rep.minc, rep.maxc) = (exact - totalcount, exact - totalcount);
        }
        if let Some(max) = check(duel, keys::FUSION_CHECK_MAX).map(|max| max as i32) {
            if max < rep.minc + totalcount || mustgcount > max {
                return false;
            }
            rep.maxc = rep.maxc.min(max - totalcount);
        }
        if let Some(min) = check(duel, keys::FUSION_CHECK_MIN).map(|min| min as i32) {
            if min > rep.maxc + totalcount {
                return false;
            }
            rep.minc = rep.minc.max(min - totalcount);
        }
        s.restrictions(duel, c, search.sg);
        let sg = search.sg.union(c.into());
        if !check_additional(duel, s.tp, sg, s.fc) {
            return false;
        }
        if Fusion::check_mix_rep_goal::<M>(duel, sg, s, rep) {
            return true;
        }
        let (search, g) = (Search { sg, ..search }, CardSet::default());
        sg.is_exists(
            duel,
            |tc, duel| Fusion::check_mix_rep_template::<M>(duel, tc, true, search, g, rep),
            1,
            None,
        )
    }
}

// Fusion.AddContactProc.
impl Fusion {
    pub fn add_contact_proc<P: ContactFusion>(duel: &mut Duel, c: CardId) {
        if c.is_status(duel, STATUS_COPYING_EFFECT) {
            return;
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_description(duel, P::DESC);
        e1.set_code(duel, EFFECT_SPSUMMON_PROC);
        e1.set_range(duel, LOCATION_EXTRA);
        e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
        if let Some(sumtype) = P::SUMTYPE {
            e1.set_value(duel, sumtype as i32);
        }
        e1.set_condition(duel, Fusion::contact_con::<P>);
        e1.set_target(duel, Fusion::contact_tg::<P>);
        e1.set_operation(duel, Fusion::contact_op::<P>);
        let e2 = P::SUMCON.map(|sumcon| {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
            e2.set_code(duel, EFFECT_SPSUMMON_CONDITION);
            if let Value::SpSummonCondition(_) = sumcon {
                e2.set_value(duel, sumcon);
            }
            e2
        });
        if P::CANNOT_BE_LIZARD {
            aux::add_lizard_check(duel, c);
        }
        c.register_effect(duel, e1);
        if let Some(e2) = e2 {
            c.register_effect(duel, e2);
        }
    }

    fn contact_con<P: ContactFusion>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        let tp = a.e.get_handler_player(duel);
        let m = P::group(duel, tp);
        let chkf = u64::from(c.get_controler(duel)) | FUSPROC_CONTACTFUS;
        c.is_facedown(duel) && c.check_fusion_material(duel, Some(m), None, chkf) && P::CONDITION.is_none_or(|fcon| fcon(duel, tp))
    }

    fn contact_tg<P: ContactFusion>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let m = P::group(duel, tp);
        let chkf = u64::from(tp) | FUSPROC_CONTACTFUS;
        let sg = duel.select_fusion_material(tp, e.get_handler(duel), m, None, chkf)?;
        if sg.is_empty() {
            return Ok(false);
        }
        e.set_label_object_group(duel, Some(sg));
        Ok(true)
    }

    fn contact_op<P: ContactFusion>(duel: &mut Duel, a: &Args) -> Script<()> {
        let Some(c) = a.c else { return Ok(()) };
        let g = a.e.get_label_object_group(duel).unwrap_or_default();
        c.set_material(duel, Some(g));
        P::op(duel, g, a.tp, c)
    }
}

// proc_fusion_spell.lua's replacement of Duel.GetFusionMaterial.
pub trait DuelFusion {
    fn get_fusion_material(&self, tp: u8) -> CardSet;
}

impl DuelFusion for Duel {
    fn get_fusion_material(&self, tp: u8) -> CardSet {
        let mut res = self.get_fusion_material_core(tp);
        let g = self.get_matching_group(
            |c, duel| c.is_has_effect(duel, EFFECT_EXTRA_FUSION_MATERIAL).is_some(),
            tp,
            LOCATION_EXTRA,
            0,
            None,
        );
        res.merge(g);
        res
    }
}

// proc_fusion_spell.lua's global EFFECT_EXTRA_FUSION_MATERIAL, registered when the library loads.
pub fn load(duel: &mut Duel) {
    let geff = duel.global_effect();
    geff.set_type(duel, EFFECT_TYPE_FIELD);
    geff.set_code(duel, EFFECT_EXTRA_FUSION_MATERIAL);
    geff.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    geff.set_target_range(duel, 0xff, 0xff);
    geff.set_target_card(duel, |duel, _, c| {
        duel.get_script_group(keys::FUSION_EXTRA_GROUP).is_some_and(|g| g.is_contains(c))
    });
    geff.set_value(duel, true);
    duel.register_effect_core(geff, 0);
    let id = geff.get_field_id(duel);
    duel.set_script_state(keys::FUSION_GLOBAL_EFFECT, id.into());
}

fn set_extra_group(duel: &mut Duel, group: Option<CardSet>) {
    duel.set_script_group(keys::FUSION_EXTRA_GROUP, group);
}

// GetExtraMatEff: the first EFFECT_EXTRA_FUSION_MATERIAL effect on c but the global one whose value accepts summon_card.
fn get_extra_mat_eff(duel: &Duel, c: CardId, summon_card: Option<CardId>) -> Option<EffectId> {
    let geff = duel.get_script_state(keys::FUSION_GLOBAL_EFFECT);
    c.is_has_effects(duel, EFFECT_EXTRA_FUSION_MATERIAL).into_iter().find(|&eff| {
        i64::from(eff.get_field_id(duel)) != geff
            && summon_card.is_none_or(|sc| match eff.get_value(duel) {
                Value::Card(val) => val(duel, eff, sc) != 0,
                Value::Int(val) => val == 1,
                _ => false,
            })
    })
}

fn extra_mat_eff(c: CardId, duel: &Duel) -> bool {
    get_extra_mat_eff(duel, c, None).is_some()
}

// ExtraMatOPTCheck: once the extra material effect's count is used up, its materials leave mg1 and efmg.
fn extra_mat_opt_check<P: FusionSummonParams>(duel: &Duel, e: EffectId, tp: u8, mg1: &mut CardSet, efmg: &mut CardSet) {
    let extra_feff_mg = mg1.filter(duel, extra_mat_eff, None);
    let Some(first) = extra_feff_mg.get_first() else {
        if let Some(first) = efmg.get_first()
            && get_extra_mat_eff(duel, first, None).is_some_and(|eff| !eff.check_count_limit(duel, tp))
        {
            efmg.clear();
        }
        return;
    };
    let Some(extra_feff) = get_extra_mat_eff(duel, first, None).filter(|eff| !eff.check_count_limit(duel, tp)) else {
        return;
    };
    let (extra_feff_loc, _) = extra_feff.get_target_range(duel);
    let in_range = |c: CardId, duel: &Duel| c.is_location(duel, extra_feff_loc);
    let (extrafil_g, _) = P::extrafil(duel, e, tp, *mg1);
    if extrafil_g.is_none_or(|g| !g.is_empty() && !g.is_exists(duel, in_range, 1, None)) {
        *mg1 = mg1.difference(extra_feff_mg.filter(duel, in_range, None));
        efmg.clear();
    }
}

impl Fusion {
    pub fn create_summon_eff<P: FusionSummonParams>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, P::DESC);
        e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
        e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
        e1.set_code(duel, EVENT_FREE_CHAIN);
        e1.set_target(duel, Fusion::summon_eff_tg::<P>);
        e1.set_operation(duel, Fusion::summon_eff_op::<P>);
        e1
    }

    pub fn register_summon_eff<P: FusionSummonParams>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = Fusion::create_summon_eff::<P>(duel, c);
        c.register_effect(duel, e1);
        e1
    }

    // Fusion.OnFieldMat as a matfilter: the library calls it with the card first, which takes its `type(filter) == "Card"` branch.
    pub fn on_field_mat(c: CardId, duel: &Duel) -> bool {
        c.is_on_field(duel) && c.is_able_to_grave(duel)
    }

    // Fusion.InHandMat as a matfilter, which takes the same `type(filter) == "Card"` branch.
    pub fn in_hand_mat(c: CardId, duel: &Duel) -> bool {
        c.is_location(duel, LOCATION_HAND) && c.is_able_to_grave(duel)
    }

    // A card that calls Fusion.SummonEffTG or SummonEffOP itself builds a new closure each time, whose chkf starts as nil.
    pub fn new_summon_eff_closure(duel: &mut Duel, e: EffectId, op: bool) {
        let key = keys::fusion_chkf(e.get_owner(duel), op);
        duel.set_script_state(key, 0);
    }

    // The SummonEffTG/OP closures keep chkf|tp of every caller; Effect.Clone shares the closure, so it is kept per creator.
    fn upvalue_chkf<P: FusionSummonParams>(duel: &mut Duel, e: EffectId, tp: u8, op: bool, add: u64) -> u64 {
        let key = keys::fusion_chkf(e.get_owner(duel), op);
        let stored = duel.get_script_state(key);
        let chkf = match stored & SET {
            0 => P::CHKF,
            _ => Some((stored & !SET) as u64),
        };
        let chkf = match chkf {
            Some(chkf) if chkf & u64::from(PLAYER_NONE) == u64::from(PLAYER_NONE) => chkf,
            Some(chkf) => chkf | u64::from(tp),
            None => u64::from(tp),
        } | add;
        duel.set_script_state(key, chkf as i64 | SET);
        chkf
    }

    // Fusion.SummonEffFilter: efmg materials accepting c join mg for good, as the Lua merges them into the shared group.
    fn summon_eff_filter<P: FusionSummonParams>(
        c: CardId,
        duel: &Duel,
        e: EffectId,
        tp: u8,
        (mg, efmg): (&Cell<CardSet>, CardSet),
        gc: Option<CardSet>,
        chkf: u64,
        value: u32,
        sumlimit: bool,
    ) -> bool {
        mg.set(
            mg.get()
                .union(efmg.filter(duel, |m, duel| get_extra_mat_eff(duel, m, Some(c)).is_some(), None)),
        );
        c.is_type(duel, TYPE_FUSION)
            && P::fusfilter(c, duel)
            && (P::NOSUMMONCHECK || c.is_can_be_special_summoned_full(duel, e, value, tp, sumlimit, false, P::SUMPOS, tp, 0xff))
            && c.check_fusion_material(duel, Some(mg.get()), gc, chkf)
    }

    // The materials the target and the operation start from; Fusion.ExtraGroup becomes what `extra_group` keeps.
    fn summon_materials<P: FusionSummonParams>(
        duel: &mut Duel,
        e: EffectId,
        tp: u8,
        extra_group: impl Fn(&Duel, CardSet) -> CardSet,
    ) -> (CardSet, CardSet, Option<CardSet>, Option<FusionCheck>) {
        let fmg_all = duel.get_fusion_material(tp);
        let mut mg1 = fmg_all.filter(duel, |c, duel| P::matfilter(c, duel, e, tp), None);
        let efmg = fmg_all.filter(duel, extra_mat_eff, None);
        // An effect whose label object replaces extrafil (c86758746) is not ported.
        let extra_feff = efmg.get_first().and_then(|c| get_extra_mat_eff(duel, c, None));
        unported!(duel, extra_feff.is_some_and(|eff| eff.get_label_object(duel).is_some()));
        let (extra, check_addition) = P::extrafil(duel, e, tp, mg1);
        if let Some(extra) = extra {
            let group = extra_group(duel, extra);
            set_extra_group(duel, Some(group));
            mg1.merge(extra);
        }
        (mg1, efmg, extra, check_addition)
    }

    pub fn summon_eff_tg<P: FusionSummonParams>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let chkf = Fusion::upvalue_chkf::<P>(duel, e, tp, false, 0);
        let sumlimit = chkf & (FUSPROC_NOTFUSION | FUSPROC_NOLIMIT) != 0;
        let notfusion = chkf & FUSPROC_NOTFUSION != 0;
        let value = match notfusion {
            true => P::VALUE | MATERIAL_FUSION,
            false => P::VALUE | MATERIAL_FUSION | u64::from(SUMMON_TYPE_FUSION),
        };
        let gc = P::gc(duel, e, tp);
        if chk == 0 {
            let can_be_material = |c: CardId, duel: &Duel| c.is_can_be_fusion_material(duel, None, value);
            let not_immune = |c: CardId, duel: &Duel| !c.is_immune_to_effect(duel, e);
            let extra_group = |duel: &Duel, g: CardSet| g.filter(duel, can_be_material, None).filter(duel, not_immune, None);
            let (mut mg1, mut efmg, _, check_addition) = Fusion::summon_materials::<P>(duel, e, tp, extra_group);
            if gc.is_some_and(|gc| !mg1.includes(gc)) {
                set_extra_group(duel, None);
                return Ok(false);
            }
            duel.set_fusion_check_additional(check_addition);
            mg1.keep_matching(duel, can_be_material, None);
            mg1.keep_matching(duel, not_immune, None);
            set_checks(duel, [P::EXACTCOUNT, P::MINCOUNT, P::MAXCOUNT]);
            extra_mat_opt_check::<P>(duel, e, tp, &mut mg1, &mut efmg);
            let mg = Cell::new(mg1);
            let filter = |c, duel: &Duel| Fusion::summon_eff_filter::<P>(c, duel, e, tp, (&mg, efmg), gc, chkf, value as u32, sumlimit);
            let res = duel.is_existing_matching_card(filter, tp, P::LOCATION, 0, 1, None);
            duel.set_fusion_check_additional(None);
            set_extra_group(duel, None);
            if !res && !notfusion {
                unported!(duel, !duel.get_player_effect(tp, EFFECT_CHAIN_MATERIAL).is_empty());
            }
            set_checks(duel, [None; 3]);
            return Ok(res);
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, P::LOCATION as i32);
        P::extratg(duel, a, chk)?;
        Ok(false)
    }

    pub fn summon_eff_op<P: FusionSummonParams>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let cancelable = if P::PRESELECT.is_none() { FUSPROC_CANCELABLE } else { 0 };
        let chkf = Fusion::upvalue_chkf::<P>(duel, e, tp, true, cancelable);
        let sumlimit = chkf & (FUSPROC_NOTFUSION | FUSPROC_NOLIMIT) != 0;
        let notfusion = chkf & FUSPROC_NOTFUSION != 0;
        let value = match notfusion {
            true => P::VALUE,
            false => P::VALUE | u64::from(SUMMON_TYPE_FUSION) | MATERIAL_FUSION,
        };
        let gc = P::gc(duel, e, tp);
        let material_filter = |c: CardId, duel: &Duel| {
            c.is_can_be_fusion_material(duel, None, value) && !c.is_immune_to_effect(duel, e) && aux::nvfilter(c, duel)
        };
        let extra_group = |duel: &Duel, g: CardSet| g.filter(duel, material_filter, None);
        let (mut mg1, mut efmg, extragroup, check_addition) = Fusion::summon_materials::<P>(duel, e, tp, extra_group);
        mg1.keep_matching(duel, material_filter, None);
        let forced_mat_validity = |c: CardId, duel: &Duel| match c == e.get_handler(duel) {
            true => !c.is_relate_to_effect(duel, e),
            false => c.is_immune_to_effect(duel, e),
        };
        if gc.is_some_and(|gc| !mg1.includes(gc) || gc.is_exists(duel, forced_mat_validity, 1, None)) {
            set_extra_group(duel, None);
            return Ok(());
        }
        set_checks(duel, [P::EXACTCOUNT, P::MINCOUNT, P::MAXCOUNT]);
        duel.set_fusion_check_additional(check_addition);
        extra_mat_opt_check::<P>(duel, e, tp, &mut mg1, &mut efmg);
        let mg = Cell::new(mg1);
        let filter = |c, duel: &Duel| Fusion::summon_eff_filter::<P>(c, duel, e, tp, (&mg, efmg), gc, chkf, value as u32, sumlimit);
        let sg1 = duel.get_matching_group(filter, tp, P::LOCATION, 0, None);
        let mg1 = mg.get();
        set_extra_group(duel, None);
        duel.set_fusion_check_additional(None);
        if !notfusion {
            unported!(duel, !duel.get_player_effect(tp, EFFECT_CHAIN_MATERIAL).is_empty());
        }
        if !sg1.is_empty() {
            let (tc, mut mat1) = loop {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let Some(tc) = sg1.select(duel, tp, 1, 1, None)?.get_first() else {
                    return Ok(());
                };
                if let Some(preselect) = P::PRESELECT
                    && !preselect(duel, e, tc)?
                {
                    return Ok(());
                }
                duel.set_fusion_check_additional(check_addition);
                set_extra_group(duel, extragroup);
                let mat1 = duel.select_fusion_material(tp, tc, mg1, gc, chkf)?;
                if !mat1.is_empty() {
                    break (tc, mat1);
                }
            };
            set_extra_group(duel, None);
            let backupmat = mat1;
            if !notfusion {
                tc.set_material(duel, Some(mat1));
            }
            let extra_mat_eff_for = |c: CardId, duel: &Duel| get_extra_mat_eff(duel, c, Some(tc)).is_some();
            let extra_feff_mg = mat1.filter(duel, extra_mat_eff_for, None);
            if let Some(extraop) = P::EXTRAOP
                && let Some(extra_feff) = extra_feff_mg.get_first().and_then(|c| get_extra_mat_eff(duel, c, Some(tc)))
                && let Some(Operation::FusionMaterial(extra_feff_op)) = extra_feff.get_operation(duel)
                && !std::ptr::fn_addr_eq(extraop, extra_feff_op)
                && extra_feff.check_count_limit(duel, tp)
            {
                let (extra_feff_loc, _) = extra_feff.get_target_range(duel);
                let (extrafil_g, _) = P::extrafil(duel, e, tp, mg1);
                // No extrafil forces the effect, but the Lua errors on one returning nil; the two are not told apart.
                unported!(duel, extrafil_g.is_none());
                let mut applied = None;
                if extrafil_g.is_none_or(|g| !g.is_exists(duel, |c, duel| c.is_location(duel, extra_feff_loc), 1, None)) {
                    applied = Some(extra_feff_mg);
                } else if duel.select_effect_yes_no(tp, extra_feff.get_handler(duel))? {
                    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RESOLVECARD);
                    let max = u8::try_from(extra_feff_mg.len()).unwrap_or(u8::MAX);
                    applied = Some(extra_feff_mg.select(duel, tp, 1, max, None)?).filter(|g| !g.is_empty());
                }
                if let Some(mut g) = applied {
                    mat1 = mat1.difference(g);
                    extra_feff_op(duel, e, tc, tp, &mut g)?;
                    if extra_feff.check_count_limit(duel, tp) {
                        extra_feff.use_count_limit(duel, tp);
                    }
                }
            }
            if let Some(extraop) = P::EXTRAOP {
                extraop(duel, e, tc, tp, &mut mat1)?;
            }
            if !mat1.is_empty() {
                let (mut extra_feff_mg, normal_mg) = mat1.split(duel, extra_mat_eff_for, None);
                let extra_feff = extra_feff_mg.get_first().and_then(|c| get_extra_mat_eff(duel, c, Some(tc)));
                // Group.AddMaximumCheck adds only Maximum monsters (proc_maximum.lua, out of scope).
                if !normal_mg.is_empty() && extra_feff.is_none_or(|eff| eff.get_label(duel) != 160018042) {
                    duel.send_to_grave(normal_mg, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
                }
                if let Some(extra_feff) = extra_feff {
                    match extra_feff.get_operation(duel) {
                        Some(Operation::FusionMaterial(extra_feff_op)) => extra_feff_op(duel, e, tc, tp, &mut extra_feff_mg)?,
                        _ => {
                            duel.send_to_grave(extra_feff_mg, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
                        }
                    }
                    if extra_feff.check_count_limit(duel, tp) {
                        extra_feff.use_count_limit(duel, tp);
                    }
                }
            }
            duel.break_effect()?;
            duel.special_summon_step(tc, value as u32, tp, tp, sumlimit, false, P::SUMPOS)?;
            P::stage2(duel, e, Some(tc), tp, Some(backupmat), 0)?;
            duel.special_summon_complete()?;
            P::stage2(duel, e, Some(tc), tp, Some(backupmat), 3)?;
            if chkf & FUSPROC_NOTFUSION == 0 {
                tc.complete_procedure(duel);
            }
            P::stage2(duel, e, Some(tc), tp, Some(backupmat), 1)?;
        }
        P::stage2(duel, e, None, tp, None, 2)?;
        set_check(duel, keys::FUSION_CHECK_MIN, None);
        set_check(duel, keys::FUSION_CHECK_MAX, None);
        set_check(duel, keys::FUSION_CHECK_EXACT, None);
        duel.set_fusion_check_additional(None);
        Ok(())
    }

    pub fn banish_material(duel: &mut Duel, _e: EffectId, _tc: CardId, _tp: u8, sg: &mut CardSet) -> Script<()> {
        duel.remove(*sg, POS_FACEUP, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
        sg.clear();
        Ok(())
    }

    pub fn shuffle_material(duel: &mut Duel, _e: EffectId, _tc: CardId, tp: u8, sg: &mut CardSet) -> Script<()> {
        let rg = sg.filter(duel, CardId::is_facedown, None);
        let hg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED), None);
        if !rg.is_empty() {
            duel.confirm_cards(1 - tp, rg)?;
        }
        if !hg.is_empty() {
            duel.hint_selection(hg);
        }
        // Group.AddMaximumCheck adds only Maximum monsters (proc_maximum.lua, out of scope).
        duel.send_to_deck(*sg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
        sg.clear();
        Ok(())
    }

    // Fusion.IsMonsterFilter(f): `filter` is f.
    pub fn is_monster_filter(c: CardId, duel: &Duel, filter: impl Fn(CardId, &Duel) -> bool) -> bool {
        c.is_monster(duel) && filter(c, duel)
    }

    // Fusion.CheckWithHandler(fun)(c, e, ...): `fun` is fun with its arguments.
    pub fn check_with_handler(c: CardId, duel: &Duel, e: EffectId, fun: impl Fn(CardId, &Duel) -> bool) -> bool {
        c == e.get_handler(duel) || fun(c, duel)
    }

    pub fn forced_handler(duel: &Duel, e: EffectId) -> CardSet {
        e.get_handler(duel).into()
    }
}
