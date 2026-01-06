problem: writing to TUN apparently fails with various odd errors.

Solution: switched to the `tokio-tun` crate, removing rawfd hacks. This resolves the write errors and stabilizes TUN handling.