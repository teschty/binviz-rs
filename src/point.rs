#[derive(Copy, Clone)]
pub struct Point {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub count: u32,
}

implement_vertex!(Point, pos, color);

