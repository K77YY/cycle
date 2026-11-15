@file:OptIn(ExperimentalMaterial3Api::class)

package app.cycle.timer

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.pm.PackageManager
import android.content.res.Configuration
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.FilledTonalIconButton
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

fun Context.findActivity(): Activity? {
    var c = this
    while (c is ContextWrapper) {
        if (c is Activity) return c
        c = c.baseContext
    }
    return null
}

@Composable
fun TimerScreen(onEdit: () -> Unit, onNew: () -> Unit) {
    val s by Engine.snap.collectAsState()
    val set by Engine.settings.collectAsState()
    val color = kindColor(s.stage.kind)
    val running = s.state == RunState.Running
    val ctx = LocalContext.current
    val view = LocalView.current
    val landscape = LocalConfiguration.current.orientation == Configuration.ORIENTATION_LANDSCAPE
    var sheet by remember { mutableStateOf(false) }

    // 通知権限 (Android 13+): 初回の再生時に一度だけ尋ねる。拒否してもタイマーは動く。
    var asked by rememberSaveable { mutableStateOf(false) }
    val permLauncher = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {}
    val playPause = {
        if (Build.VERSION.SDK_INT >= 33 && !asked &&
            ContextCompat.checkSelfPermission(ctx, Manifest.permission.POST_NOTIFICATIONS)
            != PackageManager.PERMISSION_GRANTED
        ) {
            asked = true
            permLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
        Engine.toggle()
    }

    // 実行中は画面を点けたままにする
    DisposableEffect(running) {
        view.keepScreenOn = running
        onDispose { view.keepScreenOn = false }
    }
    // メニュー非表示中はステータスバー等も隠す (端をスワイプで一時表示)
    DisposableEffect(set.showMenu) {
        val window = ctx.findActivity()?.window
        val c = window?.let { WindowCompat.getInsetsController(it, view) }
        if (!set.showMenu) {
            c?.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            c?.hide(WindowInsetsCompat.Type.systemBars())
        }
        onDispose { c?.show(WindowInsetsCompat.Type.systemBars()) }
    }

    if (!set.showMenu) {
        ZenView(s, color, onTap = playPause, onShow = { Engine.setMenu(true) })
    } else {
        Column(Modifier.fillMaxSize().safeDrawingPadding().padding(horizontal = 16.dp)) {
            Row(Modifier.fillMaxWidth().padding(top = 4.dp), verticalAlignment = Alignment.CenterVertically) {
                TextButton(onClick = { sheet = true }, modifier = Modifier.weight(1f, fill = false)) {
                    Text(s.plan.name, maxLines = 1, overflow = TextOverflow.Ellipsis, style = MaterialTheme.typography.titleMedium)
                    Icon(Icons.Default.KeyboardArrowDown, contentDescription = "シーケンスを選ぶ")
                }
                Spacer(Modifier.weight(1f))
                FilterChip(
                    selected = set.sound,
                    onClick = Engine::toggleSound,
                    label = { Text(if (set.sound) "音 オン" else "音 オフ") },
                )
                IconButton(onClick = onEdit) { Icon(Icons.Default.Edit, contentDescription = "編集") }
                TextButton(onClick = { Engine.setMenu(false) }) { Text("隠す") }
            }
            if (landscape) {
                Row(Modifier.weight(1f).fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    CenterContent(s, color, 64.sp, Modifier.weight(1f))
                    Controls(s, color, vertical = true, onPlay = playPause)
                }
            } else {
                CenterContent(s, color, 96.sp, Modifier.weight(1f).fillMaxWidth())
                Controls(s, color, vertical = false, onPlay = playPause)
            }
        }
    }

    if (sheet) {
        PlanSheet(
            current = s.plan.name,
            onDismiss = { sheet = false },
            onNew = { sheet = false; onNew() },
        )
    }
}

// ---------- メニュー非表示 (カウントダウン + 進捗バーのみ) ----------

@Composable
private fun ZenView(s: Snapshot, color: Color, onTap: () -> Unit, onShow: () -> Unit) {
    val haptic = LocalHapticFeedback.current
    val tap by rememberUpdatedState(onTap)
    val show by rememberUpdatedState(onShow)
    var hint by remember { mutableStateOf(true) }
    LaunchedEffect(Unit) {
        delay(2500)
        hint = false
    }
    Box(
        Modifier.fillMaxSize().pointerInput(Unit) {
            detectTapGestures(
                onTap = { tap() },
                onLongPress = {
                    haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                    show()
                },
            )
        },
        contentAlignment = Alignment.Center,
    ) {
        Column(Modifier.padding(horizontal = 28.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            BigClock(fmtClock(s.remainingSecs), color, s.state == RunState.Paused || s.state == RunState.Ready, 96.sp)
            Spacer(Modifier.height(28.dp))
            ProgressBar(s.progress, color)
        }
        if (hint) {
            Text(
                "タップで一時停止/再開 ・ 長押しでメニュー表示",
                Modifier.align(Alignment.BottomCenter).padding(bottom = 48.dp),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

// ---------- 中央 ----------

@Composable
private fun CenterContent(s: Snapshot, color: Color, clockSize: TextUnit, modifier: Modifier) {
    val stage = s.stage
    val dim = s.state == RunState.Paused || s.state == RunState.Ready
    val muted = MaterialTheme.colorScheme.onSurfaceVariant
    Column(modifier.padding(vertical = 8.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(10.dp).clip(CircleShape).background(color))
            Spacer(Modifier.width(8.dp))
            Text(stage.kind.label, color = color, fontWeight = FontWeight.Bold, style = MaterialTheme.typography.titleMedium)
            Text("  ・  ${stage.name}", style = MaterialTheme.typography.titleMedium)
        }
        Spacer(Modifier.height(12.dp))
        BigClock(fmtClock(s.remainingSecs), color, dim, clockSize)
        Spacer(Modifier.height(16.dp))
        ProgressBar(s.progress, color)
        Spacer(Modifier.height(24.dp))
        Text(
            "サイクル ${s.cycle}  ・  ステージ ${s.index + 1}/${s.plan.stages.size}",
            color = muted, style = MaterialTheme.typography.bodyMedium,
        )
        Text(
            (if (s.plan.looping) "サイクル残り " else "全体の残り ") + fmtClock(s.remainingTotalMs / 1000),
            fontWeight = FontWeight.Medium, style = MaterialTheme.typography.bodyMedium,
        )
        val status = when (s.state) {
            RunState.Ready -> "再生ボタンで開始"
            RunState.Paused -> "一時停止中"
            RunState.Finished -> "完了しました"
            RunState.Running -> ""
        }
        Text(status, color = color, style = MaterialTheme.typography.labelLarge, modifier = Modifier.padding(top = 4.dp).height(20.dp))
        Spacer(Modifier.height(8.dp))
        StageStrip(s)
    }
}

@Composable
private fun BigClock(clock: String, color: Color, dim: Boolean, size: TextUnit) {
    val fs = if (clock.length > 5) size * 0.72f else size
    Text(
        clock,
        color = if (dim) color.copy(alpha = 0.55f) else color,
        maxLines = 1,
        softWrap = false,
        style = MaterialTheme.typography.displayLarge.copy(
            fontSize = fs,
            lineHeight = fs,
            fontWeight = FontWeight.Light,
            fontFeatureSettings = "tnum",
            letterSpacing = (-1).sp,
        ),
    )
}

@Composable
private fun ProgressBar(p: Float, color: Color) {
    Row(Modifier.widthIn(max = 480.dp).fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Box(Modifier.weight(1f).height(8.dp).clip(RoundedCornerShape(4.dp)).background(color.copy(alpha = 0.18f))) {
            Box(Modifier.fillMaxHeight().fillMaxWidth(p).background(color))
        }
        Text(
            "${(p * 100).toInt()}%",
            Modifier.width(48.dp),
            textAlign = TextAlign.End,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun StageStrip(s: Snapshot) {
    val state = rememberLazyListState()
    LaunchedEffect(s.index) { state.animateScrollToItem(s.index) }
    LazyRow(
        Modifier.fillMaxWidth(),
        state = state,
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
    ) {
        itemsIndexed(s.plan.stages) { i, st ->
            val cur = i == s.index
            val c = kindColor(st.kind)
            val shape = RoundedCornerShape(50)
            Box(
                Modifier.clip(shape)
                    .background(if (cur) c.copy(alpha = 0.16f) else Color.Transparent)
                    .border(1.dp, if (cur) c else MaterialTheme.colorScheme.outlineVariant, shape)
                    .padding(horizontal = 12.dp, vertical = 6.dp)
            ) {
                Text(
                    "${fmtHuman(st.durationSecs)} ${st.kind.label}",
                    style = MaterialTheme.typography.labelMedium,
                    color = if (cur) c else MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        if (s.plan.looping) {
            item {
                Text("↻", Modifier.padding(horizontal = 6.dp, vertical = 4.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

// ---------- 操作ボタン ----------

@Composable
private fun Controls(s: Snapshot, color: Color, vertical: Boolean, onPlay: () -> Unit) {
    val running = s.state == RunState.Running
    val buttons: @Composable () -> Unit = {
        FilledTonalIconButton(onClick = Engine::reset, modifier = Modifier.size(52.dp)) {
            Icon(Icons.Default.Refresh, contentDescription = "リセット")
        }
        FilledTonalIconButton(onClick = Engine::prev, modifier = Modifier.size(52.dp)) {
            Glyph(G.Prev, LocalContentColor.current)
        }
        FilledIconButton(
            onClick = onPlay,
            modifier = Modifier.size(76.dp),
            colors = IconButtonDefaults.filledIconButtonColors(
                containerColor = color,
                contentColor = MaterialTheme.colorScheme.background,
            ),
        ) {
            Glyph(if (running) G.Pause else G.Play, LocalContentColor.current, 34.dp)
        }
        FilledTonalIconButton(onClick = Engine::next, modifier = Modifier.size(52.dp)) {
            Glyph(G.Next, LocalContentColor.current)
        }
    }
    if (vertical) {
        Column(Modifier.padding(start = 16.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(14.dp)) { buttons() }
    } else {
        Row(
            Modifier.fillMaxWidth().padding(bottom = 20.dp, top = 8.dp),
            horizontalArrangement = Arrangement.SpaceEvenly,
            verticalAlignment = Alignment.CenterVertically,
        ) { buttons() }
    }
}

// ---------- シーケンス選択シート ----------

@Composable
private fun PlanSheet(current: String, onDismiss: () -> Unit, onNew: () -> Unit) {
    val state = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()
    var plans by remember { mutableStateOf<List<Plan>>(emptyList()) }
    var saveDialog by remember { mutableStateOf(false) }
    var deleteTarget by remember { mutableStateOf<Plan?>(null) }

    fun reload() {
        scope.launch { plans = withContext(Dispatchers.IO) { Engine.listPlans() } }
    }
    LaunchedEffect(Unit) {
        Engine.persist() // 現在のシーケンスも一覧に出す
        reload()
    }

    ModalBottomSheet(onDismissRequest = onDismiss, sheetState = state) {
        Column(Modifier.padding(horizontal = 20.dp).padding(bottom = 24.dp)) {
            Text("シーケンス", style = MaterialTheme.typography.titleLarge)
            Spacer(Modifier.height(8.dp))
            LazyColumn(Modifier.heightIn(max = 360.dp)) {
                items(plans, key = { it.name }) { p ->
                    Row(
                        Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp))
                            .clickable { Engine.loadPlan(p); onDismiss() }
                            .padding(start = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(Modifier.weight(1f).padding(vertical = 10.dp)) {
                            Text(p.name, fontWeight = if (p.name == current) FontWeight.Bold else FontWeight.Normal)
                            Text(
                                "${p.stages.size}ステージ ・ ${fmtHuman(p.totalSecs)}",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        if (p.name == current) {
                            Text("使用中", color = MaterialTheme.colorScheme.primary, style = MaterialTheme.typography.labelMedium)
                        }
                        IconButton(onClick = { deleteTarget = p }) {
                            Icon(Icons.Default.Delete, contentDescription = "削除")
                        }
                    }
                }
            }
            Spacer(Modifier.height(12.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedButton(onClick = onNew) { Text("新規作成") }
                OutlinedButton(onClick = { saveDialog = true }) { Text("名前を付けて保存") }
            }
        }
    }

    if (saveDialog) {
        var name by remember { mutableStateOf(current) }
        AlertDialog(
            onDismissRequest = { saveDialog = false },
            title = { Text("名前を付けて保存") },
            text = { OutlinedTextField(name, { name = it.take(40) }, singleLine = true, label = { Text("名前") }) },
            confirmButton = {
                TextButton(
                    enabled = name.isNotBlank(),
                    onClick = { Engine.saveAs(name); saveDialog = false; reload() },
                ) { Text("保存") }
            },
            dismissButton = { TextButton(onClick = { saveDialog = false }) { Text("キャンセル") } },
        )
    }
    deleteTarget?.let { t ->
        AlertDialog(
            onDismissRequest = { deleteTarget = null },
            title = { Text("削除しますか?") },
            text = { Text("「${t.name}」を削除します。") },
            confirmButton = {
                TextButton(onClick = { Engine.deletePlan(t.name); deleteTarget = null; reload() }) { Text("削除") }
            },
            dismissButton = { TextButton(onClick = { deleteTarget = null }) { Text("キャンセル") } },
        )
    }
}
