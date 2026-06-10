#![no_std]

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlowKey {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub _pad: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FlowStats {
    pub packets_sent: u64,
    pub packets_recv: u64,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub ts_start_ns: u64,
    pub ts_last_ns: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum FlowEventType {
    New = 1,
    Close = 2,
    Timeout = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FlowEvent {
    pub ty: FlowEventType,
    pub key: FlowKey,
    pub stats: FlowStats,
}
