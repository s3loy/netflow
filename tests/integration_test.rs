// Integration tests require Linux + root
// Run inside Lima VM

#[test]
#[ignore = "requires Linux kernel with eBPF support"]
fn test_tcp_flow_detection() {
    // TODO: Implement when running in Linux VM
    // 1. Load eBPF programs
    // 2. Create TCP connection
    // 3. Verify FlowEvent::New received
    // 4. Close connection
    // 5. Verify FlowEvent::Close received
}
