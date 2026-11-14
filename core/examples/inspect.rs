use std::time::Instant;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bytes = std::fs::read(&args[1]).expect("read");
    let t = Instant::now();
    let w = seedcore::World::parse(&bytes).expect("parse");
    let t1 = t.elapsed();
    let a = seedcore::analyze(&w);
    let t2 = t.elapsed();
    println!("parse {:?} analyze {:?}", t1, t2 - t1);
    println!("{}", serde_json::to_string(&a.header).unwrap());
    println!("{}", serde_json::to_string(&a.summary).unwrap());
    let mut kinds = std::collections::BTreeMap::new();
    for p in &a.pois { *kinds.entry(p.kind).or_insert(0) += 1; }
    println!("{:?}", kinds);
    for l in a.loot.iter().filter(|l| l.tier == 2) { println!("  RARE {} @ {} ({},{})", l.item, l.place, l.x, l.y); }
    let places: std::collections::BTreeMap<_, usize> = a.chests.iter().fold(Default::default(), |mut m, c| { *m.entry(c.place.clone()).or_insert(0) += 1; m });
    println!("chests by place {:?}", places);
    println!("surface: {}", a.surface.iter().map(|s| format!("{}[{}-{}]", s.biome, s.x0, s.x1)).collect::<Vec<_>>().join(" "));
    if let Some(out) = args.get(2) {
        let img = seedcore::render::render(&w, 15);
        let f = std::fs::File::create(out).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), w.header.width as u32, w.header.height as u32);
        e.set_color(png::ColorType::Rgba);
        e.write_header().unwrap().write_image_data(&img).unwrap();
    }
}
