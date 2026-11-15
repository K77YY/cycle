package app.cycle.timer

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TimerCoreTest {
    private fun plan(loop: Boolean) = Plan(
        "t",
        listOf(Stage("a", 60, Kind.Focus), Stage("b", 60, Kind.Break)),
        loop,
    )

    @Test
    fun advancesAndLoops() {
        val t = TimerCore(plan(true))
        t.toggle(0)
        assertTrue(t.tick(60_000))
        assertEquals(1, t.index)
        assertTrue(t.tick(120_000))
        assertEquals(0, t.index)
        assertEquals(2, t.cycle)
    }

    @Test
    fun finishesWithoutLoop() {
        val t = TimerCore(plan(false))
        t.toggle(0)
        assertTrue(t.tick(120_000))
        assertEquals(RunState.Finished, t.state)
    }

    @Test
    fun resetReturnsToCycleOneStageOne() {
        val t = TimerCore(plan(true))
        t.toggle(0)
        t.tick(90_000)
        assertEquals(1, t.index)
        t.reset(150_000)
        assertEquals(0, t.index)
        assertEquals(1, t.cycle)
        assertEquals(RunState.Ready, t.state)
    }

    @Test
    fun defaultPlanIsStudy() {
        val p = Plan.default()
        assertEquals("勉強", p.name)
        assertEquals(listOf(55L, 5L, 25L, 5L), p.stages.map { it.durationSecs / 60 })
        assertEquals(listOf("読書", "休憩", "漢検", "休憩"), p.stages.map { it.name })
    }
}
