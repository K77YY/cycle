package app.cycle.timer

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            CycleTheme {
                Surface(Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
                    App()
                }
            }
        }
    }
}

@Composable
private fun App() {
    var draft by remember { mutableStateOf<Plan?>(null) }
    val d = draft
    if (d == null) {
        TimerScreen(
            onEdit = { draft = Engine.snap.value.plan },
            onNew = { draft = Plan.blank() },
        )
    } else {
        EditorScreen(
            draft = d,
            onChange = { draft = it },
            onClose = {
                Engine.applyEdit(d.copy(name = d.name.trim().ifBlank { "無題" }))
                draft = null
            },
        )
    }
}
