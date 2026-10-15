package com.qeli

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class AndroidDnsCompatibilityPolicyTest {
    private val secure = AndroidDnsCompatibilityPolicy.Readiness.SECURE_DEFAULT

    @Test fun defaultNeverOptsIntoBypass() {
        assertFalse(AndroidDnsCompatibilityPolicy.DEFAULT_ENABLED)
        for (profile in listOf(false, true)) {
            for (lockdown in listOf(false, true, null)) {
                assertEquals(secure, AndroidDnsCompatibilityPolicy.evaluate(false, profile, lockdown))
            }
        }
    }

    @Test fun explicitOptInRequiresKnownUnlockedSystemAndNoProfileKillSwitch() {
        assertEquals(AndroidDnsCompatibilityPolicy.Readiness.COMPATIBILITY_READY,
            AndroidDnsCompatibilityPolicy.evaluate(true, false, false))
        assertEquals(AndroidDnsCompatibilityPolicy.Readiness.SYSTEM_LOCKDOWN,
            AndroidDnsCompatibilityPolicy.evaluate(true, false, true))
        assertEquals(AndroidDnsCompatibilityPolicy.Readiness.LOCKDOWN_UNOBSERVABLE,
            AndroidDnsCompatibilityPolicy.evaluate(true, false, null))
        for (lockdown in listOf(false, true, null)) {
            assertEquals(AndroidDnsCompatibilityPolicy.Readiness.PROFILE_KILL_SWITCH,
                AndroidDnsCompatibilityPolicy.evaluate(true, true, lockdown))
        }
    }
}
