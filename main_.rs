use crossterm::terminal;

type Point3D = (i32, i32, i32);
type Point2D = (i32, i32);

pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub cx: i32,
    pub cy: i32,
    pub ax: i32,
    pub ay: i32,
}

impl Canvas {
    pub fn new() -> Self {
        let (w, h) = terminal::size().unwrap();
        let ax = 4;
        let ay = 1;
        let cx = w as i32 / 2;
        let cy = h as i32 / 2;
        let w = w as i32 - ax * 2;
        let h = h as i32 - ay * 2;
        Self {
            ax,
            ay,
            cx,
            cy,
            w,
            h,
        }
    }

    pub fn print(&self) {
        println!("Screen Width : {}", self.w);
        println!("Screen Height: {}", self.h);
        println!("Center X     : {}", self.cx);
        println!("Center Y     : {}", self.cy);
    }
}

pub struct Cube {
    pub points: [Point3D; 8],
    pub edges: [[Point3D; 2]; 12],
    pub faces: [[Point3D; 4]; 6],
}

impl Cube {
    pub fn new(canvas: &Canvas) -> Self {
        let h = canvas.h;
        let s = h / 6;
        let p1 = (-s, -s, -s);
        let p2 = (s, -s, -s);
        let p3 = (s, s, -s);
        let p4 = (-s, s, -s);
        let p5 = (-s, -s, s);
        let p6 = (s, -s, s);
        let p7 = (s, s, s);
        let p8 = (-s, s, s);

        let points = [p1, p2, p3, p4, p5, p6, p7, p8];
        let edges = [
            [p1, p2],
            [p2, p3],
            [p3, p4],
            [p4, p1],
            [p5, p6],
            [p6, p7],
            [p7, p8],
            [p8, p5],
            [p1, p5],
            [p2, p6],
            [p3, p7],
            [p4, p8],
        ];
        let faces = [
            [p1, p2, p3, p4],
            [p5, p6, p7, p8],
            [p1, p2, p6, p5],
            [p3, p4, p8, p7],
            [p1, p5, p8, p4],
            [p2, p3, p7, p6],
        ];

        Self {
            points,
            edges,
            faces,
        }
    }

    pub fn print(&self) {
        println!("Cube Points:");
        for (i, &(x, y, z)) in self.points.iter().enumerate() {
            println!("p{}: ({:.2}, {:.2}, {:.2})", i + 1, x, y, z);
        }
    }
}

pub struct Projector {
    h: i32,
    cx: i32,
    cy: i32,
    x_angle: f32,
    y_angle: f32,
    x_scale: i32,
    y_scale: i32,
    z_offset: i32,
    min_z: i32,
}

impl Projector {
    pub fn new(canvas: &Canvas) -> Self {
        let cx = canvas.cx;
        let cy = canvas.cy;
        let h = canvas.h;
        let x_angle = 0.00;
        let y_angle = 0.00;
        let x_scale = 2;
        let y_scale = 1;
        let z_offset = h * 1.2 as i32;
        let min_z = 0;
        Self {
            cx,
            cy,
            h,
            x_angle,
            y_angle,
            x_scale,
            y_scale,
            z_offset,
            min_z,
        }
    }

    pub fn print(&self) {
        println!("x_angle : {:.2}", self.x_angle);
        println!("y_angle : {:.2}", self.y_angle);
        println!("z_offset: {:.2}", self.z_offset);
    }

    fn rotate_point(&self, p: Point3D) -> Point3D {
        if self.x_angle == 0.0 && self.y_angle == 0.0 {
            return p;
        }

        let x = p.0 as f32;
        let y = p.1 as f32;
        let z = p.2 as f32;
        let sx = self.x_angle.sin();
        let cx = self.x_angle.cos();
        let sy = self.y_angle.sin();
        let cy = self.y_angle.cos();

        let x1 = x;
        let y1 = y * cx - z * sx;
        let z1 = y * sx + z * cx;

        let rx = x1 * cy - z1 * sy;
        let ry = y1;
        let rz = x1 * sy + z1 * cy;

        (rx as i32, ry as i32, rz as i32)
    }

    fn project_point(&self, p: Point3D) -> Point2D {
        let h = self.h as f32;
        let cx = self.cx as f32;
        let cy = self.cy as f32;
        let x_scale = self.x_scale as f32;
        let y_scale = self.y_scale as f32;
        let z_offset = self.z_offset as f32;

        let x = p.0 as f32;
        let y = p.1 as f32;
        let z = p.2 as f32;

        let scale = 1.0 / (z_offset - z);

        let px = cx + x * scale * h * x_scale;
        let py = cy - y * scale * h * y_scale;

        (px as i32, py as i32)
    }

    fn project_edge(&self, p1: Point2D, p2: Point2D) -> Vec<Point2D> {
        let (x1, y1) = p1;
        let (x2, y2) = p2;
        let mut points = Vec::new();

        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1;
        let mut y = y1;
        let mut step = 0;

        loop {
            // if step % 2 == 0 {
            //     points.push((x, y));
            // }
            points.push((x, y));

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }

            step += 1;
        }

        points
    }

    fn project_face(&self, p1: Point2D, p2: Point2D, p3: Point2D, p4: Point2D) -> Vec<Point2D> {
        let mut points = Vec::new();

        points.extend(self.project_edge(p1, p2));
        points.extend(self.project_edge(p2, p3));
        points.extend(self.project_edge(p3, p4));
        points.extend(self.project_edge(p4, p1));

        let edges = [(p1, p2), (p2, p3), (p3, p4), (p4, p1)];

        let min_y = p1.1.min(p2.1).min(p3.1).min(p4.1);
        let max_y = p1.1.max(p2.1).max(p3.1).max(p4.1);

        for y in min_y..=max_y {
            let mut xs = Vec::new();
            for &(a, b) in &edges {
                let y1 = a.1;
                let y2 = b.1;

                if (y1 <= y && y2 >= y) || (y2 <= y && y1 >= y) {
                    let dy = y2 - y1;
                    if dy == 0 {
                        continue;
                    };
                    let t = (y as f32 - y1 as f32) / dy as f32;
                    let x = a.0 as f32 + t * (b.0 as f32 - a.0 as f32);
                    xs.push(x as i32);
                }
            }

            xs.sort_unstable();

            let mut i = 0;
            while i + 1 < xs.len() {
                let x_start = xs[i];
                let x_end = xs[i + 1];
                for x in x_start..=x_end {
                    points.push((x as i32, y as i32));
                }
                i += 2;
            }
        }

        points
    }

    pub fn get_points(&mut self, points: &[Point3D; 8]) -> (Vec<Point2D>, Vec<Point2D>) {
        let mut front_points = Vec::new();
        let mut back_points = Vec::new();
        let min_z = points
            .iter()
            .map(|&p| self.rotate_point(p).2)
            .min()
            .unwrap();
        self.min_z = min_z;

        for &p in points {
            let rp = self.rotate_point(p);
            let pp = self.project_point(rp);

            if rp.2 == min_z {
                back_points.push(pp);
            } else {
                front_points.push(pp);
            }
        }

        (front_points, back_points)
    }

    pub fn get_edges(&self, edges: &[[Point3D; 2]; 12]) -> (Vec<Vec<Point2D>>, Vec<Vec<Point2D>>) {
        let mut front_edges = Vec::new();
        let mut back_edges = Vec::new();
        let min_z = self.min_z;

        for &[p0, p1] in edges {
            let rp0 = self.rotate_point(p0);
            let rp1 = self.rotate_point(p1);
            let pp0 = self.project_point(rp0);
            let pp1 = self.project_point(rp1);

            if rp0.2 == min_z || rp1.2 == min_z {
                back_edges.push(self.project_edge(pp0, pp1));
            } else {
                front_edges.push(self.project_edge(pp0, pp1));
            }
        }
        (front_edges, back_edges)
    }

    pub fn get_faces(&self, faces: &[[Point3D; 4]; 6]) -> (Vec<Vec<Point2D>>, Vec<Vec<Point2D>>) {
        let mut front_faces = Vec::new();
        let mut back_faces = Vec::new();
        let min_z = self.min_z;

        for &[p0, p1, p2, p3] in faces {
            let rp0 = self.rotate_point(p0);
            let rp1 = self.rotate_point(p1);
            let rp2 = self.rotate_point(p2);
            let rp3 = self.rotate_point(p3);

            let pp0 = self.project_point(rp0);
            let pp1 = self.project_point(rp1);
            let pp2 = self.project_point(rp2);
            let pp3 = self.project_point(rp3);

            if rp0.2 == min_z || rp1.2 == min_z || rp2.2 == min_z || rp3.2 == min_z {
                let face = self.project_face(pp0, pp1, pp2, pp3);
                front_faces.push(face);
            } else {
                let face = self.project_face(pp0, pp1, pp2, pp3);
                back_faces.push(face);
            }
        }

        (front_faces, back_faces)
    }

    pub fn flatten_array<T>(&self, array: Vec<Vec<T>>) -> Vec<T> {
        array.into_iter().flatten().collect()
    }
}

pub struct Renderer {
    ax: i32,
    ay: i32,
    w: i32,
    h: i32,
    buffer: Vec<Vec<char>>,
}

impl Renderer {
    pub fn new(canvas: &Canvas) -> Self {
        let ax = canvas.ax;
        let ay = canvas.ay;
        let w = canvas.w;
        let h = canvas.h;
        let buffer = vec![vec![' '; w as usize]; h as usize];
        Self {
            ax,
            ay,
            w,
            h,
            buffer,
        }
    }

    pub fn set_char(&mut self, p: Point2D, c: char) {
        let x = p.0;
        let y = p.1;
        if x >= 0 && x < self.w && y >= 0 && y < self.h {
            self.buffer[y as usize][x as usize] = c;
        }
    }

    pub fn erase_buffer(&mut self) {
        for row in &mut self.buffer {
            row.fill(' ');
        }
    }

    pub fn paint_buffer(&mut self, points: &[Point2D], char: char) {
        for &p in points {
            self.set_char(p, char);
        }
    }

    pub fn show_buffer(&self) {
        print!("\x1B[{};{}H", self.ay, self.ax);
        // print!("{}", "=========");
        for row in &self.buffer {
            let s: String = row.iter().collect();
            println!("{}", s);
        }
    }
}

fn main() {
    let screen = Canvas::new();
    let cube = Cube::new(&screen);
    let mut projector = Projector::new(&screen);
    let mut renderer = Renderer::new(&screen);
    // screen.print();
    // cube.print();
    // projector.print();
    // println!("============================");

    print!("\x1B[?25l");
    loop {
        let (front_points, back_points) = projector.get_points(&cube.points);
        let (front_edges, back_edges) = projector.get_edges(&cube.edges);
        let (front_faces, back_faces) = projector.get_faces(&cube.faces);
        let front_edges = projector.flatten_array(front_edges);
        let back_edges = projector.flatten_array(back_edges);
        let front_faces = projector.flatten_array(front_faces);
        let back_faces = projector.flatten_array(back_faces);

        renderer.erase_buffer();
        // renderer.paint_buffer(&back_faces,   '@');
        // renderer.paint_buffer(&front_faces,  '#');
        renderer.paint_buffer(&back_edges, '+');
        renderer.paint_buffer(&front_edges, '|');
        renderer.paint_buffer(&back_points, '*');
        renderer.paint_buffer(&front_points, '■');
        renderer.show_buffer();

        projector.x_angle += 0.02;
        projector.y_angle += 0.04;
        // std::thread::sleep(std::time::Duration::from_millis(40));
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    // print!("\x1B[?25h");
}
