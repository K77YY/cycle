package app.cycle.timer

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

enum class G { Play, Pause, Next, Prev }

/** 再生/一時停止/前後のアイコン (拡張アイコンライブラリ不要で軽量) */
@Composable
fun Glyph(g: G, tint: Color, box: Dp = 24.dp) {
    Canvas(Modifier.size(box)) {
        val w = size.width
        val h = size.height
        fun tri(x0: Float, x1: Float) = Path().apply {
            moveTo(x0 * w, 0.2f * h)
            lineTo(x1 * w, 0.5f * h)
            lineTo(x0 * w, 0.8f * h)
            close()
        }
        when (g) {
            G.Play -> drawPath(tri(0.3f, 0.8f), tint)
            G.Pause -> {
                drawRoundRect(tint, Offset(0.24f * w, 0.2f * h), Size(0.18f * w, 0.6f * h), CornerRadius(0.04f * w))
                drawRoundRect(tint, Offset(0.58f * w, 0.2f * h), Size(0.18f * w, 0.6f * h), CornerRadius(0.04f * w))
            }
            G.Next -> {
                drawPath(tri(0.2f, 0.68f), tint)
                drawRect(tint, Offset(0.72f * w, 0.2f * h), Size(0.1f * w, 0.6f * h))
            }
            G.Prev -> {
                drawPath(tri(0.8f, 0.32f), tint)
                drawRect(tint, Offset(0.18f * w, 0.2f * h), Size(0.1f * w, 0.6f * h))
            }
        }
    }
}
