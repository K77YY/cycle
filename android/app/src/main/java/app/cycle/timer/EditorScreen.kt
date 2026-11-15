@file:OptIn(ExperimentalMaterial3Api::class)

package app.cycle.timer

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

/** シーケンス編集。戻る(←)で内容を反映して自動保存する。 */
@Composable
fun EditorScreen(draft: Plan, onChange: (Plan) -> Unit, onClose: () -> Unit) {
    BackHandler(onBack = onClose)
    var editing by remember { mutableStateOf<Int?>(null) } // -1 = 新規追加

    fun move(i: Int, to: Int) {
        val l = draft.stages.toMutableList()
        l.add(to, l.removeAt(i))
        onChange(draft.copy(stages = l))
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("シーケンス編集") },
                navigationIcon = {
                    IconButton(onClick = onClose) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "戻る")
                    }
                },
            )
        },
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = { editing = -1 },
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("ステージを追加") },
            )
        },
    ) { pad ->
        LazyColumn(
            Modifier.padding(pad),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 96.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            item {
                OutlinedTextField(
                    value = draft.name,
                    onValueChange = { onChange(draft.copy(name = it.take(40))) },
                    label = { Text("シーケンス名") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            item {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Column(Modifier.weight(1f)) {
                        Text("ループ")
                        Text(
                            "最後のステージの後、最初に戻って繰り返す",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Switch(checked = draft.looping, onCheckedChange = { onChange(draft.copy(looping = it)) })
                }
            }
            item {
                Text(
                    "ステージ ・ 合計 ${fmtHuman(draft.totalSecs)}",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 8.dp),
                )
            }
            itemsIndexed(draft.stages) { i, st ->
                val c = kindColor(st.kind)
                Card(
                    onClick = { editing = i },
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)),
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(Modifier.width(6.dp).height(68.dp).background(c))
                        Column(Modifier.weight(1f).padding(horizontal = 12.dp)) {
                            Text(st.name, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Text("${fmtHuman(st.durationSecs)} ・ ${st.kind.label}", style = MaterialTheme.typography.bodySmall, color = c)
                        }
                        IconButton(onClick = { move(i, i - 1) }, enabled = i > 0) {
                            Icon(Icons.Default.KeyboardArrowUp, contentDescription = "上へ")
                        }
                        IconButton(onClick = { move(i, i + 1) }, enabled = i < draft.stages.size - 1) {
                            Icon(Icons.Default.KeyboardArrowDown, contentDescription = "下へ")
                        }
                        IconButton(
                            onClick = { onChange(draft.copy(stages = draft.stages.filterIndexed { j, _ -> j != i })) },
                            enabled = draft.stages.size > 1,
                        ) { Icon(Icons.Default.Delete, contentDescription = "削除") }
                    }
                }
            }
        }
    }

    editing?.let { idx ->
        StageDialog(
            initial = draft.stages.getOrNull(idx),
            onDismiss = { editing = null },
            onSave = { st ->
                val l = draft.stages.toMutableList()
                if (idx in l.indices) l[idx] = st else l.add(st)
                onChange(draft.copy(stages = l))
                editing = null
            },
        )
    }
}

@Composable
private fun StageDialog(initial: Stage?, onDismiss: () -> Unit, onSave: (Stage) -> Unit) {
    val base = initial?.durationSecs ?: 1500L
    var name by remember { mutableStateOf(initial?.name ?: "") }
    var mins by remember { mutableStateOf((base / 60).toString()) }
    var secs by remember { mutableStateOf((base % 60).toString()) }
    var kind by remember { mutableStateOf(initial?.kind ?: Kind.Focus) }
    var error by remember { mutableStateOf<String?>(null) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(if (initial == null) "ステージを追加" else "ステージを編集") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(name, { name = it.take(30) }, label = { Text("名前 (空なら種類名)") }, singleLine = true)
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedTextField(
                        mins, { mins = it.filter(Char::isDigit).take(3) },
                        label = { Text("分") }, singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        modifier = Modifier.weight(1f),
                    )
                    OutlinedTextField(
                        secs, { secs = it.filter(Char::isDigit).take(2) },
                        label = { Text("秒") }, singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        modifier = Modifier.weight(1f),
                    )
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Kind.entries.forEach { k ->
                        FilterChip(selected = kind == k, onClick = { kind = k }, label = { Text(k.label) })
                    }
                }
                error?.let { Text(it, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall) }
            }
        },
        confirmButton = {
            TextButton(onClick = {
                val m = mins.toLongOrNull() ?: 0L
                val sc = secs.toLongOrNull() ?: 0L
                val total = m * 60 + sc
                when {
                    sc >= 60 -> error = "秒は 0〜59 で入力してください"
                    total < 1 -> error = "1秒以上にしてください"
                    total > MAX_SECS -> error = "${MAX_SECS / 60}分以内にしてください"
                    else -> onSave(Stage(name.trim().ifBlank { kind.label }, total, kind))
                }
            }) { Text("決定") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("キャンセル") } },
    )
}
