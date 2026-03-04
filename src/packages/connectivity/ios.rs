use super::ConnectivityStatus;
use std::ffi::CStr;

/// Check connectivity using SCNetworkReachability (available without extra frameworks).
pub fn check_connectivity() -> ConnectivityStatus {
    unsafe {
        // Create a reachability reference for 0.0.0.0 (general internet connectivity)
        let mut zero_addr: sockaddr_in = std::mem::zeroed();
        zero_addr.sin_len = std::mem::size_of::<sockaddr_in>() as u8;
        zero_addr.sin_family = 2; // AF_INET

        let reachability = SCNetworkReachabilityCreateWithAddress(
            std::ptr::null(),
            &zero_addr as *const sockaddr_in as *const sockaddr,
        );
        if reachability.is_null() {
            return ConnectivityStatus::None;
        }

        let mut flags: u32 = 0;
        let success = SCNetworkReachabilityGetFlags(reachability, &mut flags);
        CFRelease(reachability as *const _);

        if !success {
            return ConnectivityStatus::None;
        }

        let reachable = flags & kSCNetworkReachabilityFlagsReachable != 0;
        let needs_connection = flags & kSCNetworkReachabilityFlagsConnectionRequired != 0;
        let is_wwan = flags & kSCNetworkReachabilityFlagsIsWWAN != 0;

        if !reachable || needs_connection {
            ConnectivityStatus::None
        } else if is_wwan {
            ConnectivityStatus::Cellular
        } else {
            ConnectivityStatus::Wifi
        }
    }
}

// SCNetworkReachability constants
const kSCNetworkReachabilityFlagsReachable: u32 = 1 << 1;
const kSCNetworkReachabilityFlagsConnectionRequired: u32 = 1 << 2;
const kSCNetworkReachabilityFlagsIsWWAN: u32 = 1 << 18;

// Minimal C type definitions
#[repr(C)]
struct sockaddr {
    sa_len: u8,
    sa_family: u8,
    sa_data: [i8; 14],
}

#[repr(C)]
struct sockaddr_in {
    sin_len: u8,
    sin_family: u8,
    sin_port: u16,
    sin_addr: u32,
    sin_zero: [i8; 8],
}

// SCNetworkReachability is an opaque type
type SCNetworkReachabilityRef = *const std::ffi::c_void;

extern "C" {
    fn SCNetworkReachabilityCreateWithAddress(
        allocator: *const std::ffi::c_void,
        address: *const sockaddr,
    ) -> SCNetworkReachabilityRef;

    fn SCNetworkReachabilityGetFlags(
        target: SCNetworkReachabilityRef,
        flags: *mut u32,
    ) -> bool;

    fn CFRelease(cf: *const std::ffi::c_void);
}
