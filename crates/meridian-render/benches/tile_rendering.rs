use criterion::{black_box, criterion_group, criterion_main, Criterion};
use meridian_render::{
    cache::MemoryCache,
    raster::{RasterRenderer, TileData, TileFormat},
    style::{Color, Layer, LayerType, PaintProperties, PropertyValue, Style},
    tile::TileCoord,
};

fn bench_tile_coordinate_creation(c: &mut Criterion) {
    c.bench_function("tile_coord_new", |b| {
        b.iter(|| {
            let coord = TileCoord::new(black_box(10), black_box(512), black_box(384));
            black_box(coord)
        })
    });
}

fn bench_tile_bounds_calculation(c: &mut Criterion) {
    let coord = TileCoord::new(10, 512, 384).unwrap();
    c.bench_function("tile_bounds", |b| {
        b.iter(|| {
            let bounds = coord.bounds();
            black_box(bounds)
        })
    });
}

fn bench_tile_children(c: &mut Criterion) {
    let coord = TileCoord::new(10, 512, 384).unwrap();
    c.bench_function("tile_children", |b| {
        b.iter(|| {
            let children = coord.children();
            black_box(children)
        })
    });
}

fn bench_cache_operations(c: &mut Criterion) {
    let cache = MemoryCache::new(1000);
    let coord = TileCoord::new(10, 512, 384).unwrap();

    c.bench_function("cache_put_get", |b| {
        b.iter(|| {
            let tile = meridian_render::cache::CachedTile::new(
                coord,
                vec![1, 2, 3, 4, 5],
                "image/png".to_string(),
            );
            cache.put(tile);
            let result = cache.get(&coord);
            black_box(result)
        })
    });
}

fn bench_raster_rendering_simple(c: &mut Criterion) {
    let renderer = RasterRenderer::new();
    let coord = TileCoord::new(10, 512, 384).unwrap();
    let mut style = Style::new("benchmark".to_string());

    // Add a simple background layer
    style.add_layer(Layer {
        id: "background".to_string(),
        layer_type: LayerType::Background,
        source: None,
        source_layer: None,
        minzoom: None,
        maxzoom: None,
        filter: None,
        paint: Some(PaintProperties {
            fill_color: Some(PropertyValue::Constant(Color::rgb(255, 255, 255))),
            fill_opacity: None,
            fill_outline_color: None,
            line_color: None,
            line_width: None,
            line_opacity: None,
            circle_radius: None,
            circle_color: None,
            circle_opacity: None,
            text_color: None,
            text_halo_color: None,
            text_halo_width: None,
        }),
        layout: None,
    });

    let data = TileData::new();

    c.bench_function("raster_render_simple", |b| {
        b.iter(|| {
            let image = renderer.render_tile(coord, &style, &data).unwrap();
            black_box(image)
        })
    });
}

fn bench_raster_rendering_with_geometry(c: &mut Criterion) {
    let renderer = RasterRenderer::new();
    let coord = TileCoord::new(10, 512, 384).unwrap();
    let bounds = coord.bounds();

    let mut style = Style::new("benchmark".to_string());
    style.add_layer(Layer {
        id: "fill".to_string(),
        layer_type: LayerType::Fill,
        source: Some("test".to_string()),
        source_layer: None,
        minzoom: None,
        maxzoom: None,
        filter: None,
        paint: Some(PaintProperties {
            fill_color: Some(PropertyValue::Constant(Color::rgb(100, 150, 200))),
            fill_opacity: Some(PropertyValue::Constant(0.8)),
            fill_outline_color: None,
            line_color: None,
            line_width: None,
            line_opacity: None,
            circle_radius: None,
            circle_color: None,
            circle_opacity: None,
            text_color: None,
            text_halo_color: None,
            text_halo_width: None,
        }),
        layout: None,
    });

    let mut data = TileData::new();
    // Add a simple polygon
    let cx = (bounds.min_x + bounds.max_x) / 2.0;
    let cy = (bounds.min_y + bounds.max_y) / 2.0;
    let size = bounds.width() / 4.0;
    data.add_polygon(vec![
        (cx - size, cy - size),
        (cx + size, cy - size),
        (cx + size, cy + size),
        (cx - size, cy + size),
        (cx - size, cy - size),
    ]);

    c.bench_function("raster_render_polygon", |b| {
        b.iter(|| {
            let image = renderer.render_tile(coord, &style, &data).unwrap();
            black_box(image)
        })
    });
}

fn bench_image_encoding(c: &mut Criterion) {
    let renderer = RasterRenderer::new();
    let coord = TileCoord::new(10, 512, 384).unwrap();
    let style = Style::new("benchmark".to_string());
    let data = TileData::new();

    let image = renderer.render_tile(coord, &style, &data).unwrap();

    c.bench_function("encode_png", |b| {
        b.iter(|| {
            let encoded = renderer.encode(&image, TileFormat::Png).unwrap();
            black_box(encoded)
        })
    });

    c.bench_function("encode_jpeg", |b| {
        b.iter(|| {
            let encoded = renderer.encode(&image, TileFormat::Jpeg).unwrap();
            black_box(encoded)
        })
    });
}

criterion_group!(
    benches,
    bench_tile_coordinate_creation,
    bench_tile_bounds_calculation,
    bench_tile_children,
    bench_cache_operations,
    bench_raster_rendering_simple,
    bench_raster_rendering_with_geometry,
    bench_image_encoding,
);
criterion_main!(benches);
