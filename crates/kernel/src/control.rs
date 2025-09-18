//! Control plane message parsing and graph wiring (V0 binary framing).
//! Frame header: magic 'C'(0x43), ver u8(0), cmd u8, flags u8, len u32 LE, payload[len].
//! Commands:
//!  0x01 CreateGraph {}
//!  0x02 AddChannel { capacity_le_u16 }
//!  0x03 AddOperator { op_id_le_u32, in_ch_le_u16(0xFFFF=none), out_ch_le_u16(0xFFFF=none), priority_u8, stage_u8 }
//!  0x04 StartGraph { steps_le_u32 }
//!  0x05 AddOperatorTyped { op_id_le_u32, in_ch_le_u16(0xFFFF=none), out_ch_le_u16(0xFFFF=none), priority_u8, stage_u8, in_schema_le_u32, out_schema_le_u32 }
//!  0x06 EnableDeterministic { wcet_ns_le_u64, period_ns_le_u64, deadline_ns_le_u64 }

use crate::graph::{GraphApi, OperatorSpec, Stage};
use crate::trace::metric_kv;

static mut CTRL_GRAPH: Option<GraphApi> = None;

pub enum CtrlError {
    BadFrame,
    Unsupported,
    NoGraph,
}

pub fn handle_frame(frame: &[u8]) -> Result<(), CtrlError> {
    if frame.len() < 8 { return Err(CtrlError::BadFrame); }
    if frame[0] != 0x43 { return Err(CtrlError::BadFrame); } // 'C'
    let ver = frame[1];
    let cmd = frame[2];
    let _flags = frame[3];
    let len = u32::from_le_bytes([frame[4], frame[5], frame[6], frame[7]]) as usize;
    if ver != 0 { return Err(CtrlError::Unsupported); }
    if frame.len() < 8 + len { return Err(CtrlError::BadFrame); }
    let payload = &frame[8..8+len];

    match cmd {
        0x01 => { // CreateGraph
            unsafe { CTRL_GRAPH = Some(GraphApi::create()); }
            ctrl_print(b"CTRL: graph created\n");
            // Emit basic graph stats metrics (ops/channels)
            if let Some((ops, chans)) = current_graph_counts() {
                metric_kv("graph_stats_ops", ops);
                metric_kv("graph_stats_channels", chans);
            }
            Ok(())
        }
        0x02 => { // AddChannel
            if payload.len() < 2 { return Err(CtrlError::BadFrame); }
            let cap = u16::from_le_bytes([payload[0], payload[1]]) as usize;
            unsafe {
                if let Some(ref mut g) = CTRL_GRAPH {
                    let _ = g.add_channel(crate::graph::ChannelSpec { capacity: cap });
                    ctrl_print(b"CTRL: channel added\n");
                    if let Some((ops, chans)) = current_graph_counts() {
                        metric_kv("graph_stats_ops", ops);
                        metric_kv("graph_stats_channels", chans);
                    }
                    Ok(())
                } else { Err(CtrlError::NoGraph) }
            }
        }
        0x03 => { // AddOperator
            ctrl_print(b"CTRL: begin add-operator\n");
            if payload.len() < 4+2+2+1+1 { ctrl_print(b"CTRL: bad frame len\n"); return Err(CtrlError::BadFrame); }
            let op_id = u32::from_le_bytes([payload[0],payload[1],payload[2],payload[3]]);
            let in_ch = u16::from_le_bytes([payload[4],payload[5]]);
            let out_ch = u16::from_le_bytes([payload[6],payload[7]]);
            let prio = payload[8];
            let stage_u8 = payload[9];
            let stage = match stage_u8 { 0=>Some(Stage::AcquireData),1=>Some(Stage::CleanData),2=>Some(Stage::ExploreData),3=>Some(Stage::ModelData),4=>Some(Stage::ExplainResults), _=>None };
            unsafe {
                if let Some(ref mut g) = CTRL_GRAPH {
                    // Defensive index checks
                    if in_ch != 0xFFFF && (in_ch as usize) >= g.counts().1 { ctrl_print(b"CTRL: in_ch OOR\n"); }
                    if out_ch != 0xFFFF && (out_ch as usize) >= g.counts().1 { ctrl_print(b"CTRL: out_ch OOR\n"); }
                    let spec = OperatorSpec {
                        id: op_id,
                        func: crate::graph::op_a_run,
                        in_ch: if in_ch==0xFFFF { None } else { Some(in_ch as usize) },
                        out_ch: if out_ch==0xFFFF { None } else { Some(out_ch as usize) },
                        priority: prio,
                        stage,
                        in_schema: None,
                        out_schema: None,
                    };
                    ctrl_print(b"CTRL: inserting operator\n");
                    let _idx = g.add_operator(spec);
                    ctrl_print(b"CTRL: operator added\n");
                    if let Some((ops, chans)) = current_graph_counts() {
                        metric_kv("graph_stats_ops", ops);
                        metric_kv("graph_stats_channels", chans);
                    }
                    Ok(())
                } else { Err(CtrlError::NoGraph) }
            }
        }
        0x04 => { // StartGraph (run steps)
            if payload.len() < 4 { return Err(CtrlError::BadFrame); }
            let steps = u32::from_le_bytes([payload[0],payload[1],payload[2],payload[3]]) as usize;
            unsafe {
                if let Some(ref mut g) = CTRL_GRAPH {
                    let t0 = crate::graph::now_cycles();
                    g.run_steps(steps);
                    let t1 = crate::graph::now_cycles();
                    let ns = crate::graph::cycles_to_ns(t1.saturating_sub(t0));
                    metric_kv("scheduler_run_us", (ns / 1000) as usize);
                    ctrl_print(b"CTRL: ran steps\n");
                    Ok(())
                } else { Err(CtrlError::NoGraph) }
            }
        }
        0x05 => { // AddOperatorTyped
            ctrl_print(b"CTRL: begin add-operator (typed)\n");
            if payload.len() < (4+2+2+1+1+4+4) { return Err(CtrlError::BadFrame); }
            let op_id = u32::from_le_bytes([payload[0],payload[1],payload[2],payload[3]]);
            let in_ch = u16::from_le_bytes([payload[4],payload[5]]);
            let out_ch = u16::from_le_bytes([payload[6],payload[7]]);
            let prio = payload[8];
            let stage_u8 = payload[9];
            let in_schema = u32::from_le_bytes([payload[10],payload[11],payload[12],payload[13]]);
            let out_schema = u32::from_le_bytes([payload[14],payload[15],payload[16],payload[17]]);
            let stage = match stage_u8 { 0=>Some(Stage::AcquireData),1=>Some(Stage::CleanData),2=>Some(Stage::ExploreData),3=>Some(Stage::ModelData),4=>Some(Stage::ExplainResults), _=>None };
            unsafe {
                if let Some(ref mut g) = CTRL_GRAPH {
                    // Defensive: ensure channel indices are in range when present
                    if in_ch != 0xFFFF && (in_ch as usize) >= g.counts().1 { ctrl_print(b"CTRL: typed in_ch OOR\n"); }
                    if out_ch != 0xFFFF && (out_ch as usize) >= g.counts().1 { ctrl_print(b"CTRL: typed out_ch OOR\n"); }
                    let spec = OperatorSpec {
                        id: op_id,
                        func: crate::graph::op_a_run,
                        in_ch: if in_ch==0xFFFF { None } else { Some(in_ch as usize) },
                        out_ch: if out_ch==0xFFFF { None } else { Some(out_ch as usize) },
                        priority: prio,
                        stage,
                        in_schema: if in_schema == 0 { None } else { Some(in_schema) },
                        out_schema: if out_schema == 0 { None } else { Some(out_schema) },
                    };
                    ctrl_print(b"CTRL: inserting operator (typed)\n");
                    let _idx = g.add_operator(spec);
                    ctrl_print(b"CTRL: operator added (typed)\n");
                    if let Some((ops, chans)) = current_graph_counts() {
                        metric_kv("graph_stats_ops", ops);
                        metric_kv("graph_stats_channels", chans);
                    }
                    Ok(())
                } else { Err(CtrlError::NoGraph) }
            }
        }
        0x06 => { // EnableDeterministic (graph-level)
            if payload.len() < (8+8+8) { return Err(CtrlError::BadFrame); }
            let wcet = u64::from_le_bytes([payload[0],payload[1],payload[2],payload[3],payload[4],payload[5],payload[6],payload[7]]);
            let period = u64::from_le_bytes([payload[8],payload[9],payload[10],payload[11],payload[12],payload[13],payload[14],payload[15]]);
            let deadline = u64::from_le_bytes([payload[16],payload[17],payload[18],payload[19],payload[20],payload[21],payload[22],payload[23]]);
            unsafe {
                if let Some(ref mut g) = CTRL_GRAPH {
                    #[cfg(feature = "deterministic")]
                    {
                        let ok = g.enable_deterministic(wcet, period, deadline);
                        if ok { ctrl_print(b"CTRL: deterministic enabled\n"); } else { ctrl_print(b"CTRL: deterministic admit rejected\n"); }
                        return Ok(());
                    }
                    #[cfg(not(feature = "deterministic"))]
                    {
                        ctrl_print(b"CTRL: deterministic feature not enabled\n");
                        return Ok(());
                    }
                } else { Err(CtrlError::NoGraph) }
            }
        }
        _ => Err(CtrlError::Unsupported),
    }
}

fn ctrl_print(msg: &[u8]) { unsafe { crate::uart_print(msg); } }

/// Expose current graph counts for diagnostics (ops, channels)
pub fn current_graph_counts() -> Option<(usize, usize)> {
    unsafe {
        if let Some(ref g) = CTRL_GRAPH {
            Some(g.counts())
        } else {
            None
        }
    }
}
