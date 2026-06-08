#![no_std]
#![no_main]

use aya_ebpf::{
    macros::{kprobe, map},
    maps::{LruHashMap, RingBuf},
    programs::ProbeContext,
};
use netflow_common::{FlowEvent, FlowEventType, FlowKey, FlowStats};

#[map]
static FLOW_STATS: LruHashMap<FlowKey, FlowStats> = LruHashMap::with_max_entries(65536, 0);

#[map]
static FLOW_EVENTS: RingBuf = RingBuf::with_byte_size(256 * 1024, 0);

#[kprobe]
pub fn netflow_tcp_set_state(ctx: ProbeContext) -> u32 {
    match try_netflow_tcp_set_state(ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

fn try_netflow_tcp_set_state(ctx: ProbeContext) -> Result<u32, u32> {
    let sk: *mut core::ffi::c_void = ctx.arg(0).ok_or(1u32)?;
    let new_state: i32 = ctx.arg(1).ok_or(1u32)?;

    let key = extract_tcp_5tuple(sk)?;
    let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };

    const TCP_ESTABLISHED: i32 = 1;
    const TCP_CLOSE: i32 = 7;
    const TCP_TIME_WAIT: i32 = 6;

    match new_state {
        TCP_ESTABLISHED => {
            let stats = FlowStats {
                ts_start_ns: now,
                ts_last_ns: now,
                ..Default::default()
            };
            let _ = FLOW_STATS.insert(&key, &stats, 0);
            push_event(FlowEventType::New, key, stats)?;
        }
        TCP_CLOSE | TCP_TIME_WAIT => {
            if let Some(stats) = unsafe { FLOW_STATS.get(&key) } {
                let stats = *stats;
                let _ = FLOW_STATS.remove(&key);
                push_event(FlowEventType::Close, key, stats)?;
            }
        }
        _ => {}
    }

    Ok(0)
}

fn extract_tcp_5tuple(sk: *mut core::ffi::c_void) -> Result<FlowKey, u32> {
    let sk = sk as *const u8;
    let src_ip = unsafe { core::ptr::read_unaligned(sk.add(4) as *const u32) };
    let dst_ip = unsafe { core::ptr::read_unaligned(sk.add(8) as *const u32) };
    let src_port = unsafe { core::ptr::read_unaligned(sk.add(14) as *const u16) };
    let dst_port = unsafe { core::ptr::read_unaligned(sk.add(16) as *const u16) };

    Ok(FlowKey {
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        protocol: 6,
    })
}

fn push_event(ty: FlowEventType, key: FlowKey, stats: FlowStats) -> Result<(), u32> {
    if let Some(mut entry) = FLOW_EVENTS.reserve::<FlowEvent>(0) {
        entry.write(FlowEvent { ty, key, stats });
        entry.submit(0);
        Ok(())
    } else {
        Err(1)
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
