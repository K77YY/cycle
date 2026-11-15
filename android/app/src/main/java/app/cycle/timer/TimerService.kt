package app.cycle.timer

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch

/**
 * タイマー実行中だけ動くフォアグラウンドサービス。
 * 画面オフ・アプリ非表示でもプロセスを生かし、ステージ終了音を確実に鳴らす。
 * (Android の仕様上、常駐通知が必須です)
 */
class TimerService : Service() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private var job: Job? = null
    private var wake: PowerManager.WakeLock? = null
    private var lastKey = ""

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        getSystemService(NotificationManager::class.java)
            .createNotificationChannel(
                NotificationChannel(CHANNEL, "タイマー", NotificationManager.IMPORTANCE_LOW)
            )
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_TOGGLE) Engine.toggle()
        ServiceCompat.startForeground(
            this, ID, build(Engine.snap.value),
            if (Build.VERSION.SDK_INT >= 34) ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE else 0
        )
        if (job == null) job = scope.launch { Engine.snap.collect { update(it) } }
        return START_NOT_STICKY
    }

    private fun update(s: Snapshot) {
        if (s.state != RunState.Running && s.state != RunState.Paused) {
            stopForeground(STOP_FOREGROUND_REMOVE)
            stopSelf()
            return
        }
        setWake(s.state == RunState.Running)
        val key = "${s.state}-${s.index}-${s.cycle}-${s.remainingSecs}"
        if (key != lastKey) {
            lastKey = key
            getSystemService(NotificationManager::class.java).notify(ID, build(s))
        }
    }

    private fun setWake(on: Boolean) {
        if (on) {
            if (wake == null) {
                wake = getSystemService(PowerManager::class.java)
                    .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "cycle:timer")
                    .apply { setReferenceCounted(false) }
            }
            if (wake?.isHeld != true) wake?.acquire(12 * 60 * 60 * 1000L)
        } else if (wake?.isHeld == true) {
            wake?.release()
        }
    }

    private fun build(s: Snapshot): Notification {
        val running = s.state == RunState.Running
        val open = PendingIntent.getActivity(
            this, 0, Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        val toggle = PendingIntent.getService(
            this, 1, Intent(this, TimerService::class.java).setAction(ACTION_TOGGLE),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        return NotificationCompat.Builder(this, CHANNEL)
            .setSmallIcon(R.drawable.ic_stat)
            .setContentTitle("${s.stage.kind.label} ・ ${s.stage.name}")
            .setContentText("${fmtClock(s.remainingSecs)}  ・  サイクル ${s.cycle}")
            .setProgress(1000, (s.progress * 1000).toInt(), false)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setSilent(true)
            .setShowWhen(false)
            .setContentIntent(open)
            .addAction(0, if (running) "一時停止" else "再開", toggle)
            .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
            .build()
    }

    override fun onDestroy() {
        if (wake?.isHeld == true) wake?.release()
        scope.cancel()
        super.onDestroy()
    }

    companion object {
        private const val CHANNEL = "timer"
        private const val ID = 1
        private const val ACTION_TOGGLE = "app.cycle.timer.TOGGLE"

        fun start(ctx: Context) {
            ContextCompat.startForegroundService(ctx, Intent(ctx, TimerService::class.java))
        }
    }
}
