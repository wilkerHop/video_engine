use criterion::{black_box, criterion_group, criterion_main, Criterion};
use interstellar_triangulum::renderer::RenderEngine;
use interstellar_triangulum::script::{Layer, Metadata, Resolution, Scene, VideoScript};
use interstellar_triangulum::AssetLoader;
use std::path::PathBuf;

fn create_test_script() -> VideoScript {
    VideoScript {
        metadata: Metadata {
            title: "Benchmark".into(),
            resolution: Resolution::Named("1920x1080".into()),
            fps: 30,
            duration: 1.0,
            description: None,
            citations: vec![],
        },
        scenes: vec![Scene {
            id: "bench".into(),
            duration: 1.0,
            scene_type: Default::default(),
            layers: vec![Layer::Text {
                content: "Benchmark".into(),
                font: "assets/font.ttf".into(),
                font_size: 48.0,
                color: interstellar_triangulum::script::Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
                position: Default::default(),
                effects: vec![],
            }],
            transition: None,
        }],
        audio: None,
    }
}

fn bench_render_frame(c: &mut Criterion) {
    let script = create_test_script();
    let mut engine = RenderEngine::new(script);
    let mut loader = AssetLoader::new(PathBuf::from("."));

    c.bench_function("render_frame_1080p", |b| {
        b.iter(|| {
            engine.render_frame(black_box(0), &mut loader).unwrap();
        })
    });
}

criterion_group!(benches, bench_render_frame);
criterion_main!(benches);
