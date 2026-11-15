package app.cycle.timer

import android.content.Context
import android.media.AudioManager
import android.media.ToneGenerator
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

/**
 * アプリ全体で1つのタイマー。UI とフォアグラウンドサービスの両方がここを見る。
 * 時刻は elapsedRealtime (スリープ中も進む) なので、画面を消しても正確。
 */
object Engine {
    private lateinit var app: Context
    private lateinit var core: TimerCore
    private lateinit var _snap: MutableStateFlow<Snapshot>
    private val _settings = MutableStateFlow(Settings())
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private var loop: Job? = null
    private var ready = false

    val snap: StateFlow<Snapshot> get() = _snap
    val settings: StateFlow<Settings> get() = _settings

    private fun now() = SystemClock.elapsedRealtime()

    fun setup(ctx: Context) {
        if (ready) return
        ready = true
        app = ctx.applicationContext
        Store.setup(app)
        val st = Store.loadSettings()
        _settings.value = st
        val plan = st.last?.let { Store.loadPlan(it) } ?: Plan.default()
        core = TimerCore(plan)
        _snap = MutableStateFlow(snapshot())
    }

    private fun snapshot() = Snapshot(core.plan, core.index, core.cycle, core.elapsedMs, core.state)

    private fun publish() {
        _snap.value = snapshot()
    }

    /** 状態を反映し、実行中ならティックループとサービスを動かす */
    private fun sync() {
        publish()
        if (core.state == RunState.Running) {
            if (loop?.isActive != true) loop = scope.launch { runLoop() }
            TimerService.start(app)
        } else {
            loop?.cancel()
            loop = null
        }
    }

    private suspend fun runLoop() {
        while (core.state == RunState.Running) {
            if (core.tick(now()) && _settings.value.sound) Beeper.play()
            publish()
            if (core.state != RunState.Running) break
            delay(200)
        }
    }

    // ---- 操作 ----
    fun toggle() { core.toggle(now()); sync() }
    fun next() { core.next(now()); sync() }
    fun prev() { core.prev(now()); sync() }
    fun reset() { core.reset(now()); sync() }

    // ---- 設定 ----
    private fun update(f: (Settings) -> Settings) {
        _settings.value = f(_settings.value)
        Store.saveSettings(_settings.value)
    }

    fun toggleSound() {
        update { it.copy(sound = !it.sound) }
        if (_settings.value.sound) Beeper.play()
    }

    fun setMenu(show: Boolean) = update { it.copy(showMenu = show) }

    // ---- シーケンス ----
    fun persist() {
        Store.savePlan(core.plan)
        update { it.copy(last = core.plan.name) }
    }

    /** 編集内容を反映。ステージが変わればリセット、名前/ループだけなら進行状況を保つ。 */
    fun applyEdit(draft: Plan) {
        val cur = core.plan
        if (draft == cur) return
        if (draft.stages == cur.stages) core.setMeta(draft.name, draft.looping)
        else core.setPlan(draft, now())
        persist()
        sync()
    }

    fun loadPlan(p: Plan) {
        core.setPlan(p, now())
        update { it.copy(last = p.name) }
        sync()
    }

    fun saveAs(name: String) {
        core.setMeta(name.trim(), core.plan.looping)
        persist()
        publish()
    }

    fun listPlans(): List<Plan> = Store.listPlans()
    fun deletePlan(name: String) = Store.deletePlan(name)
}

/** ステージ終了音 (通知音量。マナーモードでは鳴らない)。音声ファイル不要。 */
object Beeper {
    fun play() {
        try {
            val tg = ToneGenerator(AudioManager.STREAM_NOTIFICATION, 90)
            tg.startTone(ToneGenerator.TONE_PROP_BEEP2, 600)
            Handler(Looper.getMainLooper()).postDelayed({ tg.release() }, 1000)
        } catch (e: RuntimeException) {
            // 音が出せない端末では何もしない
        }
    }
}
