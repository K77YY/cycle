package app.cycle.timer

import android.content.Context
import org.json.JSONObject
import java.io.File

data class Settings(
    val sound: Boolean = true,
    val showMenu: Boolean = true,
    val last: String? = null,
)

/** アプリ内部ストレージ (filesDir) に JSON で保存する。 */
object Store {
    private lateinit var root: File

    fun setup(ctx: Context) {
        root = ctx.filesDir
    }

    private fun seqDir(): File = File(root, "sequences").apply { mkdirs() }

    private fun write(f: File, text: String) {
        f.parentFile?.mkdirs()
        val tmp = File(f.path + ".tmp")
        tmp.writeText(text)
        if (!tmp.renameTo(f)) {
            f.writeText(text)
            tmp.delete()
        }
    }

    fun loadSettings(): Settings = try {
        val o = JSONObject(File(root, "settings.json").readText())
        Settings(
            sound = o.optBoolean("sound", true),
            showMenu = o.optBoolean("show_menu", true),
            last = if (o.has("last_sequence") && !o.isNull("last_sequence")) o.getString("last_sequence") else null,
        )
    } catch (e: Exception) {
        Settings()
    }

    fun saveSettings(s: Settings) {
        try {
            write(
                File(root, "settings.json"),
                JSONObject()
                    .put("sound", s.sound)
                    .put("show_menu", s.showMenu)
                    .put("last_sequence", s.last ?: JSONObject.NULL)
                    .toString(2)
            )
        } catch (e: Exception) {
            // 保存失敗は無視 (次回の操作で再試行される)
        }
    }

    fun savePlan(p: Plan) {
        try {
            write(File(seqDir(), fileStem(p.name) + ".json"), p.toJson())
        } catch (e: Exception) {
        }
    }

    private fun read(f: File): Plan? = try {
        Plan.fromJson(f.readText())
    } catch (e: Exception) {
        null
    }

    fun loadPlan(name: String): Plan? = read(File(seqDir(), fileStem(name) + ".json"))

    fun listPlans(): List<Plan> =
        seqDir().listFiles { f -> f.extension == "json" }
            ?.mapNotNull { read(it) }
            ?.sortedBy { it.name }
            ?: emptyList()

    fun deletePlan(name: String) {
        File(seqDir(), fileStem(name) + ".json").delete()
    }
}
