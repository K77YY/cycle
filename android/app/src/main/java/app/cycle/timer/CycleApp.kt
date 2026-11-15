package app.cycle.timer

import android.app.Application

class CycleApp : Application() {
    override fun onCreate() {
        super.onCreate()
        Engine.setup(this)
    }
}
