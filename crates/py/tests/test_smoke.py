import os
import pathlib
import time

import numpy as np
import pytest

import ygo_sim

DECK = pathlib.Path(__file__).resolve().parents[2] / "diff" / "elfnote-ycs-montreal-2026.deck"
SIZE = 64


@pytest.fixture(scope="module")
def database():
    return ygo_sim.Database()


@pytest.fixture(scope="module")
def deck(database):
    return database.load_deck(DECK)


def uniform(masks, rng):
    picks = (rng.random(len(masks)) * masks.sum(1)).astype(np.int64)
    return (masks.cumsum(1) > picks[:, None]).argmax(1)


def play(batch, seed, observe=False):
    """Steps the batch until every slot retired under a uniformly random policy, checking each action is legal."""
    rng = np.random.default_rng(seed)
    masks, acting, tensors = batch.masks(), batch.acting(), batch.observe()
    outcomes, stepping = [], 0.0
    while (acting >= 0).any():
        actions = uniform(masks, rng)
        live = acting >= 0
        assert masks[live, actions[live]].all()
        started = time.perf_counter()
        outcomes += batch.step(actions)
        stepping += time.perf_counter() - started
        batch.masks(masks)
        batch.acting(acting)
        if observe:
            batch.observe(tensors)
    return outcomes, stepping


def test_shapes_and_dtypes(database, deck):
    batch = ygo_sim.Batch(database, [(deck, deck)], SIZE, seed=3)
    masks, acting, tensors = batch.masks(), batch.acting(), batch.observe()
    assert (masks.shape, masks.dtype) == ((SIZE, ygo_sim.ACTIONS), np.bool_)
    assert (acting.shape, acting.dtype) == ((SIZE,), np.int32)
    shapes = {
        "global_": (SIZE, len(ygo_sim.GLOBAL_FEATURES)),
        "cards": (SIZE, ygo_sim.MAX_OBS_CARDS, len(ygo_sim.CARD_FEATURES)),
        "chain": (SIZE, ygo_sim.MAX_CHAIN, len(ygo_sim.CHAIN_FEATURES)),
        "actions": (SIZE, ygo_sim.ACTIONS, len(ygo_sim.ACTION_FEATURES)),
    }
    for name, shape in shapes.items():
        array = getattr(tensors, name)
        assert (array.shape, array.dtype) == (shape, np.int32), name
    assert ((tensors.actions[..., 0] >= 0) == masks).all()
    assert (tensors.global_[:, ygo_sim.GLOBAL_FEATURES.index("acting")] == 0).all()
    assert (tensors.cards[..., 0].sum(1) == 110).all()
    with pytest.raises(ygo_sim.Error):
        batch.step(np.zeros(SIZE - 1, dtype=np.int64))
    with pytest.raises(ygo_sim.Error, match="not a legal action"):
        batch.step(np.full(SIZE, ygo_sim.ACTIONS - 1, dtype=np.int64))


def test_every_duel_ends_alike_from_the_same_seed(database, deck):
    batch = ygo_sim.Batch(database, [(deck, deck)], SIZE, seed=11, duels=SIZE)
    outcomes, _ = play(batch, seed=5, observe=True)
    assert sorted(outcome.slot for outcome in outcomes) == list(range(SIZE))
    assert all(outcome.result is not None for outcome in outcomes)
    batch.reset()
    assert play(batch, seed=5)[0] == outcomes


def test_determinize_shows_no_more_than_the_observation(database, deck):
    duel = ygo_sim.Duel(database, (deck, deck), 21)
    pooled = duel.clone()
    rng = np.random.default_rng(21)
    resampled, hidden = 0, 0
    for step in range(600):
        if duel.acting is None:
            break
        if step % 20 == 0:
            player = duel.acting
            seen = duel.observe(player)
            opponent_hand = (seen.cards[0, :, 1] == 1) & (seen.cards[0, :, 2] == 0x2)
            hidden += opponent_hand.sum()
            assert (seen.cards[0, opponent_hand, 5] == 0).all()
            determinized = duel.determinize(player, step, deck)
            shown = determinized.observe(player)
            for name in ("global_", "cards", "chain", "actions"):
                assert (getattr(shown, name) == getattr(seen, name)).all(), name
            truth = duel.observe(1 - player)
            resampled += (determinized.observe(1 - player).cards != truth.cards).any()
        duel.apply(int(rng.choice(duel.legal_actions())))
    assert hidden > 0 and resampled > 0
    replayed = ygo_sim.Duel.from_record(database, duel.record())
    assert (replayed.observe(0).cards == duel.observe(0).cards).all()
    assert duel.clone(out=pooled) is pooled and pooled.record() == duel.record()
    assert ygo_sim.Duel.from_position(database, duel.position(), 1).acting is not None


def test_throughput(database, deck):
    duels = int(os.environ.get("YGO_SIM_DUELS", SIZE))
    threads = os.environ.get("RAYON_NUM_THREADS", "all")
    for observe in (False, True):
        batch = ygo_sim.Batch(database, [(deck, deck)], SIZE, seed=3, duels=duels)
        started = time.perf_counter()
        outcomes, stepping = play(batch, seed=0, observe=observe)
        seconds = time.perf_counter() - started
        actions = sum(outcome.steps + outcome.skipped for outcome in outcomes)
        print(
            f"\n{threads} threads, {duels} duels, batch {SIZE}, observe {observe}: {actions / seconds:,.0f} actions/s"
            f" ({actions / stepping:,.0f} counting step alone)"
        )
