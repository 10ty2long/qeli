package com.qeli

/** Device-local opt-in: never imported from INI/URI or enabled by a server push. */
internal object AndroidDnsCompatibilityPolicy {
    const val PREFERENCE = "android_dns_compatibility"
    const val DEFAULT_ENABLED = false

    enum class Readiness {
        SECURE_DEFAULT,
        COMPATIBILITY_READY,
        PROFILE_KILL_SWITCH,
        SYSTEM_LOCKDOWN,
        LOCKDOWN_UNOBSERVABLE,
    }

    fun evaluate(requested: Boolean, profileKillSwitch: Boolean, systemLockdown: Boolean?): Readiness = when {
        !requested -> Readiness.SECURE_DEFAULT
        profileKillSwitch -> Readiness.PROFILE_KILL_SWITCH
        systemLockdown == null -> Readiness.LOCKDOWN_UNOBSERVABLE
        systemLockdown -> Readiness.SYSTEM_LOCKDOWN
        else -> Readiness.COMPATIBILITY_READY
    }
}
