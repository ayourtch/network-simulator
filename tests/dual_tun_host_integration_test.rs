// Integration test for dual TUN host behind simulator
// This test requires root privileges (CAP_NET_ADMIN) and is therefore ignored in normal CI runs.
// It serves as documentation of the required steps and can be manually executed.

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Requires CAP_NET_ADMIN to create TUN devices and namespaces"]
    fn dual_tun_host_integration() {
        // Steps (manual or via a privileged CI runner):
        // 1. Create network namespace `ns_host` and TUN devices `tunA` and `tunB`.
        // 2. Configure addresses as per `config.toml`.
        // 3. Run the simulator binary with the appropriate configuration.
        // 4. From within the namespace, ping the host side address and verify replies.
        // 5. Clean up namespace and interfaces.
        // This placeholder test ensures the test suite includes the integration test file.
        // Actual implementation is omitted due to required privileges.
    }
}
