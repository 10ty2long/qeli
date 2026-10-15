package com.qeli

import android.content.Context
import android.view.View
import android.view.ViewGroup
import android.widget.CheckBox
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/** Actual settings/confirmation windows, not a duplicate model of the UI event handlers. */
@RunWith(AndroidJUnit4::class)
class DnsCompatibilitySettingsInstrumentedTest {
    private val context = ApplicationProvider.getApplicationContext<Context>()
    private val prefs get() = context.getSharedPreferences(MainActivity.PREFS_STATE, Context.MODE_PRIVATE)
    private var scenario: ActivityScenario<MainActivity>? = null

    @Before fun prepare() {
        assertTrue(prefs.edit().remove(AndroidDnsCompatibilityPolicy.PREFERENCE)
            .putBoolean("battery_opt_requested", true)
            .putBoolean(MainActivity.PREF_AUTO_CONNECT_LAUNCH, false)
            .putBoolean(MainActivity.PREF_AUTO_PROBE, false)
            .putBoolean(MainActivity.PREF_TRUSTED_WIFI_ENABLED, false)
            .putBoolean(MainActivity.PREF_CONNECTION_DESIRED, false).commit())
        VpnServiceImpl.liveStatus = VpnServiceImpl.STATUS_DISCONNECTED
        scenario = ActivityScenario.launch(MainActivity::class.java)
    }

    @After fun cleanup() {
        scenario?.close()
        assertTrue(prefs.edit().remove(AndroidDnsCompatibilityPolicy.PREFERENCE).commit())
    }

    private fun children(v: View): List<View> = listOf(v) + if (v is ViewGroup)
        (0 until v.childCount).flatMap { children(v.getChildAt(it)) } else emptyList()

    private fun topDialog(): View {
        val type = Class.forName("android.view.WindowManagerGlobal")
        val manager = type.getDeclaredMethod("getInstance").invoke(null)
        @Suppress("UNCHECKED_CAST")
        val roots = type.getDeclaredField("mViews").apply { isAccessible = true }.get(manager) as List<View>
        return roots.last { it.findViewById<View>(android.R.id.button1) != null }
    }

    private fun openSettings(owner: MainActivity): Pair<View, CheckBox> {
        MainActivity::class.java.getDeclaredMethod("showSettingsDialog").apply { isAccessible = true }.invoke(owner)
        val dialog = topDialog()
        return dialog to children(dialog).filterIsInstance<CheckBox>().single {
            it.text.toString() == owner.getString(R.string.dns_compatibility)
        }
    }

    @Test fun absentPreferenceShowsSecureDefaultAndWarning() {
        scenario!!.onActivity { owner ->
            val (dialog, toggle) = openSettings(owner)
            assertFalse(toggle.isChecked)
            assertTrue(children(dialog).filterIsInstance<android.widget.TextView>().any {
                it.text.toString() == owner.getString(R.string.dns_compatibility_warning)
            })
            dialog.findViewById<View>(android.R.id.button2).performClick()
            assertFalse(prefs.contains(AndroidDnsCompatibilityPolicy.PREFERENCE))
        }
    }

    @Test fun cancellingWarningNeverPersistsBypass() {
        scenario!!.onActivity { owner ->
            val (settings, toggle) = openSettings(owner)
            toggle.isChecked = true
            assertFalse(toggle.isChecked)
            assertFalse(prefs.getBoolean(AndroidDnsCompatibilityPolicy.PREFERENCE, false))
            topDialog().findViewById<View>(android.R.id.button2).performClick()
            settings.findViewById<View>(android.R.id.button1).performClick()
            assertFalse(prefs.getBoolean(AndroidDnsCompatibilityPolicy.PREFERENCE, false))
        }
    }

    @Test fun cancellingSettingsAfterConsentNeverEnablesBypass() {
        scenario!!.onActivity { owner ->
            val (settings, toggle) = openSettings(owner)
            toggle.isChecked = true
            topDialog().findViewById<View>(android.R.id.button1).performClick()
            assertTrue(toggle.isChecked)
            settings.findViewById<View>(android.R.id.button2).performClick()
            assertFalse(prefs.getBoolean(AndroidDnsCompatibilityPolicy.PREFERENCE, false))
        }
    }

    @Test fun explicitConsentAndSavePersistsAcrossRecreationAndCanBeDisabled() {
        scenario!!.onActivity { owner ->
            val (settings, toggle) = openSettings(owner)
            toggle.isChecked = true
            topDialog().findViewById<View>(android.R.id.button1).performClick()
            settings.findViewById<View>(android.R.id.button1).performClick()
            assertTrue(prefs.getBoolean(AndroidDnsCompatibilityPolicy.PREFERENCE, false))
        }
        scenario!!.recreate()
        scenario!!.onActivity { owner ->
            val (settings, toggle) = openSettings(owner)
            assertTrue(toggle.isChecked)
            toggle.isChecked = false
            settings.findViewById<View>(android.R.id.button1).performClick()
            assertFalse(prefs.getBoolean(AndroidDnsCompatibilityPolicy.PREFERENCE, false))
        }
    }
}
