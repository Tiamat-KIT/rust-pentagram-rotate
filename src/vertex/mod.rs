use std::f32::consts::PI;

#[allow(unused)]
pub fn create_star_vertices(
    outer_radius: Option<f32>,
    inner_radius: Option<f32>,
) -> (Vec<[f32; 2]>, Vec<u16>) {
    // いったん配列の限界を作らない
    let mut vertices = Vec::new();
    const NUM_VERTICES: usize = 5;
    const CENTER : [f32; 2] = [0.0, 0.0];

    for i in 0..NUM_VERTICES * 2 {
        if let Some(inner_radius) = inner_radius {
            if let Some(outer_radius) = outer_radius {
                let radius = if i % 2 == 0 { outer_radius } else { inner_radius };
                let angle = i as f32 * PI / NUM_VERTICES as f32;
                vertices.push([
                    CENTER[0] + angle.cos() * radius,
                    CENTER[1] + angle.sin() * radius,
                ]);
            }
        } else {
            let radius = if i % 2 == 0 { 1.0 } else { 0.38 };
            let angle = i as f32 * PI / NUM_VERTICES as f32;
            vertices.push([
                CENTER[0] + angle.cos() * radius,
                CENTER[1] + angle.sin() * radius,
            ]);
        }
    }

    let mut indices = Vec::new();
    for i in 0..NUM_VERTICES  * 2 {
        indices.push(i as u16);
        indices.push((i + NUM_VERTICES) as u16);
        indices.push((i + 1) as u16 % (NUM_VERTICES as u16 * 2));
        indices.push(NUM_VERTICES as u16 * 2 as u16);
    }
    vertices.push(CENTER);

    (vertices, indices)
}