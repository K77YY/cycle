package app.cycle.timer

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext

/** システムのライト/ダークと壁紙色 (Android 12+) に追従する。 */
@Composable
fun CycleTheme(content: @Composable () -> Unit) {
    val dark = isSystemInDarkTheme()
    val ctx = LocalContext.current
    val scheme = when {
        Build.VERSION.SDK_INT >= 31 ->
            if (dark) dynamicDarkColorScheme(ctx) else dynamicLightColorScheme(ctx)
        dark -> darkColorScheme()
        else -> lightColorScheme()
    }
    MaterialTheme(colorScheme = scheme, content = content)
}

/** 集中=赤、休憩=緑。ライト/ダークそれぞれで読みやすい色。 */
@Composable
fun kindColor(kind: Kind): Color {
    val dark = isSystemInDarkTheme()
    return when (kind) {
        Kind.Focus -> if (dark) Color(0xFFFF8A80) else Color(0xFFC62828)
        Kind.Break -> if (dark) Color(0xFF81C784) else Color(0xFF2E7D32)
    }
}
