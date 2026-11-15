package app.cycle.timer

import org.json.JSONArray
import org.json.JSONObject
import java.util.Locale

/** 1ステージの最大長 (999分)。デスクトップ版 cycle と同じ。 */
const val MAX_SECS = 999L * 60

enum class Kind(val label: String, val key: String) {
    Focus("集中", "focus"),
    Break("休憩", "break");

    fun toggled(): Kind = if (this == Focus) Break else Focus

    companion object {
        fun fromKey(k: String?): Kind = if (k == "break") Break else Focus
    }
}

data class Stage(val name: String, val durationSecs: Long, val kind: Kind) {
    val durationMs: Long get() = durationSecs * 1000
}

/** シーケンス。JSON 形式はデスクトップ版 (TUI) と互換。 */
data class Plan(val name: String, val stages: List<Stage>, val looping: Boolean = true) {
    val totalSecs: Long get() = stages.sumOf { it.durationSecs }

    fun toJson(): String {
        val arr = JSONArray()
        stages.forEach {
            arr.put(
                JSONObject()
                    .put("name", it.name)
                    .put("duration_secs", it.durationSecs)
                    .put("kind", it.kind.key)
            )
        }
        return JSONObject().put("name", name).put("looping", looping).put("stages", arr).toString(2)
    }

    companion object {
        /** 既定: 読書55分 → 休憩5分 → 漢検25分 → 休憩5分 (ループ) */
        fun default() = Plan(
            "勉強",
            listOf(
                Stage("読書", 55 * 60L, Kind.Focus),
                Stage("休憩", 5 * 60L, Kind.Break),
                Stage("漢検", 25 * 60L, Kind.Focus),
                Stage("休憩", 5 * 60L, Kind.Break),
            )
        )

        fun blank() = Plan("新しいシーケンス", listOf(Stage("集中", 25 * 60L, Kind.Focus)))

        fun fromJson(text: String): Plan? {
            return try {
                val o = JSONObject(text)
                val arr = o.getJSONArray("stages")
                val stages = (0 until arr.length()).map { i ->
                    val s = arr.getJSONObject(i)
                    val kind = Kind.fromKey(s.optString("kind"))
                    Stage(
                        s.optString("name", "").ifBlank { kind.label },
                        s.getLong("duration_secs").coerceIn(1, MAX_SECS),
                        kind
                    )
                }
                if (stages.isEmpty()) null
                else Plan(o.optString("name", "").ifBlank { "無題" }, stages, o.optBoolean("looping", true))
            } catch (e: Exception) {
                null
            }
        }
    }
}

fun fmtClock(secs: Long): String {
    val h = secs / 3600
    val m = (secs % 3600) / 60
    val s = secs % 60
    return if (h > 0) "%d:%02d:%02d".format(Locale.ROOT, h, m, s)
    else "%02d:%02d".format(Locale.ROOT, m, s)
}

/** 「55分」「1時間30分」「1分30秒」 */
fun fmtHuman(secs: Long): String {
    val h = secs / 3600
    val m = (secs % 3600) / 60
    val s = secs % 60
    val out = buildString {
        if (h > 0) append("${h}時間")
        if (m > 0) append("${m}分")
        if (s > 0) append("${s}秒")
    }
    return out.ifEmpty { "0秒" }
}

fun fileStem(name: String): String {
    val s = name.trim()
        .map { if (it.isLetterOrDigit() || it == '-' || it == '_' || it == ' ') it else '_' }
        .joinToString("")
        .trim()
    return s.ifEmpty { "sequence" }
}
