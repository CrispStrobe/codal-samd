// SPDX-License-Identifier: MIT
//! Actual native game, source-bound render-return boundaries and input samples.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::{
    bus::SystemBus,
    inspect::InspectOpts,
    memory::{ProgramImage, Segment},
    system::cortex_m::configure_cortex_m,
    AdvanceRequest, AdvanceStop, Arch, Machine,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Instant,
};
type M = Machine<labwired_core::cpu::cortex_m::CortexM>;
fn sha(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn rectangle(bytes: &[u8], colour: [u8; 3], size: usize) -> Option<(usize, usize)> {
    let points: Vec<_> = bytes
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, v)| *v == colour)
        .map(|(i, _)| (i % 128, i / 128))
        .collect();
    if points.len() != size * size {
        return None;
    }
    let x = points.iter().map(|p| p.0).min()?;
    let y = points.iter().map(|p| p.1).min()?;
    if points.iter().any(|&(a, b)| a >= x + size || b >= y + size) {
        return None;
    }
    // View (col,row) corresponds to game (159-row,col), MY=1, GM00 row offset 2.
    Some((159 - y - (size - 1), x))
}
fn frame(m: &mut M, render: u32, panic: u32) -> (Vec<u8>, u64) {
    for target in [render, 0] {
        let (address, sp) = if target == render {
            (render, None)
        } else {
            (m.cpu.lr & !1, Some(m.cpu.sp))
        };
        m.breakpoints = [address, panic].into_iter().collect();
        let start = m.total_cycles;
        loop {
            let report = m.advance(AdvanceRequest::run(Some(500_000))).unwrap();
            assert!(
                m.total_cycles - start < 600_000_000,
                "render boundary budget pc={:x}",
                m.cpu.pc
            );
            if let AdvanceStop::Breakpoint(pc) = report.stop {
                assert_ne!(pc, panic, "actual target_panic({})", m.cpu.r0);
                assert_eq!(pc, address);
                if let Some(sp) = sp {
                    assert_eq!(m.cpu.sp, sp, "same render call returned");
                }
                break;
            }
        }
    }
    let a = m
        .bus
        .display_artifact(
            "controller_memory",
            &InspectOpts {
                include_bytes: true,
                peripheral: None,
            },
        )
        .expect("attached controller memory");
    assert_eq!(a.meta["w"], 128);
    assert_eq!(a.meta["h"], 160);
    assert_eq!(a.meta["unknown_pixels"], 0, "complete authored colour view");
    (a.bytes.expect("actual colour pixels"), m.total_cycles)
}
fn snapshot(
    m: &mut M,
    render: u32,
    panic: u32,
    colour: [u8; 3],
    out: &Path,
    index: usize,
) -> serde_json::Value {
    let start = m.total_cycles;
    loop {
        let (bytes, cycles) = frame(m, render, panic);
        assert!(cycles - start < 600_000_000, "game picture budget");
        let Some(player) = rectangle(&bytes, colour, 16) else {
            continue;
        };
        let Some(ball) = rectangle(&bytes, [0, 48, 170], 8) else {
            continue;
        };
        // Eight 8x8 orange targets must actually be rendered.
        if bytes
            .chunks_exact(3)
            .filter(|v| *v == [255, 138, 48])
            .count()
            != 512
        {
            continue;
        }
        std::fs::write(out.join(format!("native-{index}.rgb")), &bytes).unwrap();
        return serde_json::json!({"cycles":cycles,"sha256":sha(&bytes),"player":player,"ball":ball});
    }
}
#[test]
#[ignore = "hosted source-bound actual native game only"]
fn native_game() {
    let folder = PathBuf::from(std::env::var("BW_GAME_PACKAGE").unwrap());
    let engine = std::env::var("BW_GAME_ENGINE").unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let chip = ChipDescriptor::from_file(root.join("configs/chips/samd51-no-asf-simulation.yaml"))
        .unwrap();
    let manifest =
        SystemManifest::from_file(root.join("configs/systems/samd51-no-asf-simulation.yaml"))
            .unwrap();
    let symbols: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("symbols.json")).unwrap()).unwrap();
    let render = symbols["updateScreen"].as_u64().unwrap() as u32;
    let panic = symbols["target_panic"].as_u64().unwrap() as u32;
    let raw = std::fs::read(folder.join("game.bin")).unwrap();
    let mut samples = Vec::new();
    for sample in 0..3 {
        let out = folder.join(format!("native-sample-{sample}"));
        std::fs::create_dir(&out).unwrap();
        let began = Instant::now();
        let blobs = HashMap::from([
            (
                "LABWIRED_SAMD51_SIM_SW0".into(),
                std::fs::read(folder.join("sw0.bin")).unwrap(),
            ),
            (
                "LABWIRED_SAMD51_SIM_SERIAL".into(),
                std::fs::read(folder.join("serial.bin")).unwrap(),
            ),
        ]);
        let mut bus =
            SystemBus::from_config_with_region_images(&chip, &manifest, &[], &blobs).unwrap();
        let (cpu, _) = configure_cortex_m(&mut bus);
        let mut m = Machine::new(cpu, bus);
        let mut image = ProgramImage::new(0, Arch::Arm);
        image.segments.push(Segment {
            start_addr: 0,
            data: raw.clone(),
        });
        m.load_firmware(&image).unwrap();
        let mut pictures = Vec::new();
        let mut times = Vec::new();
        for _ in 0..6 {
            let p = snapshot(&mut m, render, panic, [255, 32, 32], &out, pictures.len());
            pictures.push(p);
            times.push(began.elapsed().as_secs_f64());
        }
        assert_ne!(
            pictures[0]["ball"], pictures[5]["ball"],
            "actual moving game"
        );
        let initial = pictures[5]["player"][0].as_u64().unwrap();
        m.set_input("right", 1.).unwrap();
        m.set_input("a", 1.).unwrap();
        let pressed_at = Instant::now();
        let pressed_cycles = m.total_cycles;
        loop {
            let p = snapshot(&mut m, render, panic, [138, 255, 255], &out, pictures.len());
            let moved = p["player"][0].as_u64().unwrap() >= initial + 8;
            pictures.push(p);
            times.push(began.elapsed().as_secs_f64());
            if moved {
                break;
            }
            assert!(m.total_cycles - pressed_cycles < 600_000_000);
        }
        let press_seconds = pressed_at.elapsed().as_secs_f64();
        let press_cycles = m.total_cycles - pressed_cycles;
        m.set_input("right", 0.).unwrap();
        m.set_input("a", 0.).unwrap();
        let released_at = Instant::now();
        let released_cycles = m.total_cycles;
        for _ in 0..4 {
            let p = snapshot(&mut m, render, panic, [255, 32, 32], &out, pictures.len());
            pictures.push(p);
            times.push(began.elapsed().as_secs_f64());
        }
        let release_seconds = released_at.elapsed().as_secs_f64();
        let release_cycles = m.total_cycles - released_cycles;
        let end = pictures.len();
        assert_eq!(
            pictures[end - 1]["player"],
            pictures[end - 2]["player"],
            "released movement stopped"
        );
        let result = serde_json::json!({"sample":sample,"engine":engine,"image_sha256":sha(&raw),"startup_seconds":times[0],"first_six_render_seconds":times[5]-times[0],"render_intervals":5,"press_seconds":press_seconds,"press_cycles":press_cycles,"release_seconds":release_seconds,"release_cycles":release_cycles,"pictures":pictures,"wall_seconds":began.elapsed().as_secs_f64(),"process_status":std::fs::read_to_string("/proc/self/status").unwrap()});
        std::fs::write(
            out.join("sample.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        samples.push(result);
    }
    std::fs::write(
        folder.join("native-results.json"),
        serde_json::to_vec_pretty(&samples).unwrap(),
    )
    .unwrap();
    println!(
        "SAMD_GAME_NATIVE_PASS engine={engine} image_sha256={} samples=3 physical_capture=false",
        sha(&raw)
    );
}
