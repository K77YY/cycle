package app.cycle.timer

enum class RunState { Ready, Running, Paused, Finished }

/**
 * タイマーのロジック (UI・Android 非依存)。時刻は呼び出し側が渡す(単調時計)。
 * デスクトップ版 timer.rs の移植。
 */
class TimerCore(initial: Plan) {
    var plan: Plan = if (initial.stages.isEmpty()) Plan.default() else initial
        private set
    var index = 0
        private set
    var cycle = 1
        private set
    var elapsedMs = 0L
        private set
    var state = RunState.Ready
        private set
    private var last = 0L

    val stage: Stage get() = plan.stages[index]

    fun setPlan(p: Plan, now: Long) {
        plan = if (p.stages.isEmpty()) Plan.default() else p
        restart(now)
    }

    /** 進行状況を保ったまま名前とループ設定だけ変える */
    fun setMeta(name: String, looping: Boolean) {
        plan = plan.copy(name = name, looping = looping)
    }

    private fun restart(now: Long) {
        index = 0
        cycle = 1
        elapsedMs = 0
        state = RunState.Ready
        last = now
    }

    fun toggle(now: Long) {
        when (state) {
            RunState.Ready, RunState.Paused -> {
                last = now
                state = RunState.Running
            }
            RunState.Running -> {
                tick(now)
                if (state == RunState.Running) state = RunState.Paused
            }
            RunState.Finished -> {
                restart(now)
                state = RunState.Running
            }
        }
    }

    /** 時間を進める。ステージが切り替わった/終了したら true。 */
    fun tick(now: Long): Boolean {
        if (state != RunState.Running) return false
        elapsedMs += (now - last).coerceAtLeast(0)
        last = now
        var ended = false
        while (true) {
            val d = stage.durationMs
            if (elapsedMs < d) break
            ended = true
            elapsedMs -= d
            if (!advance()) {
                state = RunState.Finished
                elapsedMs = stage.durationMs
                break
            }
        }
        return ended
    }

    private fun advance(): Boolean {
        return if (index + 1 < plan.stages.size) {
            index++
            true
        } else if (plan.looping) {
            index = 0
            cycle++
            true
        } else false
    }

    fun next(now: Long) {
        if (state == RunState.Finished) return
        last = now
        elapsedMs = 0
        if (!advance()) {
            state = RunState.Finished
            elapsedMs = stage.durationMs
        }
    }

    fun prev(now: Long) {
        last = now
        elapsedMs = 0
        if (state == RunState.Finished) {
            state = RunState.Paused
            return
        }
        if (index > 0) index--
        else if (plan.looping && cycle > 1) {
            index = plan.stages.size - 1
            cycle--
        }
    }

    /** セッション全体をリセット (サイクル1・ステージ1の待機状態) */
    fun reset(now: Long) = restart(now)
}

/** UI に渡す不変スナップショット */
data class Snapshot(
    val plan: Plan,
    val index: Int,
    val cycle: Int,
    val elapsedMs: Long,
    val state: RunState,
) {
    val stage: Stage get() = plan.stages[index]
    val remainingMs: Long get() = (stage.durationMs - elapsedMs).coerceAtLeast(0)
    val progress: Float get() = (elapsedMs.toFloat() / stage.durationMs).coerceIn(0f, 1f)
    val remainingTotalMs: Long
        get() = if (state == RunState.Finished) 0
        else remainingMs + plan.stages.drop(index + 1).sumOf { it.durationMs }

    /** 表示用の残り秒 (切り上げ) */
    val remainingSecs: Long get() = (remainingMs + 999) / 1000
}
