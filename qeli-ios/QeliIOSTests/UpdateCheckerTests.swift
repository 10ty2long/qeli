import XCTest
@testable import Qeli

final class UpdateCheckerTests: XCTestCase {
    func testVersionNormalizationAndNumericComparison() throws {
        XCTAssertEqual(try UpdateChecker.normalize(" v0.7.12-beta+5 "), "0.7.12")
        XCTAssertTrue(try UpdateChecker.isNewer("0.10.0", than: "0.9.9"))
        XCTAssertFalse(try UpdateChecker.isNewer("v0.7.12", than: "0.7.12+715"))
        XCTAssertFalse(try UpdateChecker.isNewer("0.7.11", than: "0.7.12"))
    }

    func testPrivatePathRejectsEitherFamilyLeakAndExcludedRoutes() throws {
        let base = """
        [qeli]
        server = vpn.example.com:443
        user = alice
        pass = secret
        """
        XCTAssertTrue((try VPNConfig(parsing: base)).hasPrivateUpdatePath())

        for narrowing in [
            "gateway = false",
            "allow_ipv4_leak = true",
            "allow_ipv6_leak = true",
            "allow_lan = true",
            "exclude = 203.0.113.0/24",
        ] {
            let config = try VPNConfig(parsing: base + "\n" + narrowing)
            XCTAssertFalse(config.hasPrivateUpdatePath(), narrowing)
        }
        XCTAssertFalse(
            (try VPNConfig(parsing: base)).hasPrivateUpdatePath(globalAllowLAN: true)
        )
    }
}
