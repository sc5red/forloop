//! Integration tests for sandbox module.
//!
//! These tests verify the process isolation and syscall filtering.
//! WARNING: These tests may fail on non-Linux systems or without privileges.

/// Test that sandbox can be created with default policy.
#[test]
#[cfg(target_os = "linux")]
fn test_sandbox_creation() {
    use forloop_sandbox::{Sandbox, SandboxPolicy};
    
    let policy = SandboxPolicy::content_process();
    let sandbox = Sandbox::new(policy);
    
    assert!(sandbox.is_ok(), "Sandbox creation should succeed");
}

/// Test that policy rejects dangerous syscalls.
#[test]
#[cfg(target_os = "linux")]
fn test_policy_blocks_dangerous_syscalls() {
    use forloop_sandbox::SandboxPolicy;
    
    let policy = SandboxPolicy::content_process();
    
    // These syscalls should be blocked
    let dangerous_syscalls = [
        "ptrace",      // Process debugging
        "process_vm_readv", // Read other process memory
        "process_vm_writev", // Write other process memory
        "mount",       // Mount filesystems
        "umount2",     // Unmount filesystems
        "kexec_load",  // Load new kernel
        "init_module", // Load kernel module
        "delete_module", // Unload kernel module
    ];
    
    for syscall in &dangerous_syscalls {
        assert!(
            !policy.is_syscall_allowed(syscall),
            "Syscall {} should be blocked",
            syscall
        );
    }
}

/// Test that policy allows necessary syscalls.
#[test]
#[cfg(target_os = "linux")]
fn test_policy_allows_necessary_syscalls() {
    use forloop_sandbox::SandboxPolicy;
    
    let policy = SandboxPolicy::content_process();
    
    // These syscalls are needed for basic operation
    let necessary_syscalls = [
        "read",
        "write",
        "close",
        "mmap",
        "munmap",
        "brk",
        "exit",
        "exit_group",
    ];
    
    for syscall in &necessary_syscalls {
        assert!(
            policy.is_syscall_allowed(syscall),
            "Syscall {} should be allowed",
            syscall
        );
    }
}

/// Test landlock filesystem restrictions.
#[test]
#[cfg(target_os = "linux")]
fn test_landlock_restrictions() {
    use forloop_sandbox::filesystem::LandlockPolicy;
    
    let policy = LandlockPolicy::content_process();
    
    // Should allow reading from /usr/lib (libraries)
    assert!(
        policy.can_read("/usr/lib"),
        "Should be able to read /usr/lib"
    );
    
    // Should not allow writing to /usr
    assert!(
        !policy.can_write("/usr/lib"),
        "Should not be able to write to /usr/lib"
    );
    
    // Should not allow any access to /etc/shadow
    assert!(
        !policy.can_read("/etc/shadow"),
        "Should not be able to read /etc/shadow"
    );
}

/// Test network restrictions.
#[test]
#[cfg(target_os = "linux")]
fn test_network_restrictions() {
    use forloop_sandbox::network::NetworkPolicy;
    
    let policy = NetworkPolicy::content_process();
    
    // Content process should only be able to use UNIX sockets
    // to communicate with network process
    assert!(
        policy.allows_unix_sockets(),
        "Should allow UNIX sockets"
    );
    
    // Should not allow direct network access
    assert!(
        !policy.allows_tcp(),
        "Should not allow direct TCP"
    );
    
    assert!(
        !policy.allows_udp(),
        "Should not allow direct UDP"
    );
}

/// Test that sandbox survives across fork.
#[test]
#[cfg(target_os = "linux")]
fn test_sandbox_survives_fork() {
    use forloop_sandbox::{Sandbox, SandboxPolicy};
    use std::process::Command;
    
    // This is a simplified test - real test would use fork()
    let policy = SandboxPolicy::content_process();
    
    // Child processes should inherit sandbox restrictions
    assert!(
        policy.inheritable(),
        "Sandbox policy should be inheritable"
    );
}

/// Test resource limits.
#[test]
#[cfg(target_os = "linux")]
fn test_resource_limits() {
    use forloop_sandbox::resources::ResourceLimits;
    
    let limits = ResourceLimits::content_process();
    
    // Memory limit should be reasonable for content process
    assert!(
        limits.memory_limit_mb() <= 4096,
        "Memory limit should be <= 4GB, got {}MB",
        limits.memory_limit_mb()
    );
    
    // CPU time should be limited
    assert!(
        limits.cpu_time_limit_secs() > 0,
        "CPU time should have a limit"
    );
    
    // File descriptor limit should be reasonable
    assert!(
        limits.max_open_files() <= 1024,
        "File descriptor limit should be <= 1024, got {}",
        limits.max_open_files()
    );
}

/// Test process capabilities are dropped.
#[test]
#[cfg(target_os = "linux")]
fn test_capabilities_dropped() {
    use forloop_sandbox::capabilities::CapabilitySet;
    
    let caps = CapabilitySet::content_process();
    
    // All dangerous capabilities should be dropped
    let dangerous_caps = [
        "CAP_SYS_ADMIN",
        "CAP_SYS_PTRACE",
        "CAP_NET_ADMIN",
        "CAP_NET_RAW",
        "CAP_DAC_OVERRIDE",
        "CAP_FOWNER",
        "CAP_SETUID",
        "CAP_SETGID",
    ];
    
    for cap in &dangerous_caps {
        assert!(
            !caps.has(cap),
            "Capability {} should be dropped",
            cap
        );
    }
}
