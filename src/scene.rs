use backend::{Colour, EMEnv, Slit, Wave};
use num::Zero;
use zene_structs::{Vector2, Vector3, Vector};

#[derive(Debug, Clone, Default)]
pub struct SceneSlit
{
    pub width: f64,
    pub position: f64
}
#[derive(Debug, Clone)]
pub struct Wall
{
    a: Vector2<f64>,
    b: Vector2<f64>,
    dir: Vector2<f64>,
    slits: Vec<SceneSlit>
}

impl Default for Wall
{
    fn default() -> Self
    {
        return Self {
            a: Vector2::zero(),
            b: Vector2::zero(),
            dir: Vector2::zero(),
            slits: Default::default()
        };
    }
}

impl Wall
{
    pub fn set_a(&mut self, a: Vector2<f64>)
    {
        self.a = a;
        self.dir = (self.b - a).normalised();
    }
    pub fn set_b(&mut self, b: Vector2<f64>)
    {
        self.b = b;
        self.dir = (b - self.a).normalised();
    }
    pub fn get_a_b(&self) -> (Vector2<f64>, Vector2<f64>)
    {
        return (self.a, self.b);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scene
{
    env: EMEnv<f64>,
    waves: Box<[Wave<f64>]>,
    walls: Vec<Wall>
}

impl Scene
{
    pub fn compute_waves(&mut self, spec: &[[f32; 4]], wave_map: &[(f64, Vector3<f32>)])
    {
        let waves = spec.iter().zip(wave_map.iter()).map(|(a, w)|
        {
            return Wave::<f64>::new(w.0, a[0] as f64);
        });
        
        self.waves = waves.collect::<Box<[Wave<f64>]>>();
    }
    pub fn simulate<S>(&self, wave_map: &[(f64, Vector3<f32>)], samples: &mut [S])
        where S: From<Vector3<f32>>
    {
        let sim_slits = self.get_slits();
        self.env.generate_pattern(&sim_slits, wave_map, samples);
    }
    fn get_slits(&self) -> Vec<Slit<f64>>
    {
        let mut sim_slits = Vec::<Slit<f64>>::with_capacity(self.walls.len() * 2);
        for w in &self.walls
        {
            for s in &w.slits
            {
                sim_slits.push(s.get_slit(w, &self.waves));
            }
        }
        return sim_slits;
    }
}

impl SceneSlit
{
    pub fn get_position(&self, wall: &Wall) -> Vector2<f64>
    {
        return wall.a + (wall.dir * self.position);
    }
    pub fn get_slit<'a>(&self, wall: &Wall, waves: &'a [Wave<f64>]) -> Slit<'a, f64>
    {
        return Slit::new(self.width, self.get_position(&wall), wall.dir.rotated_90(), waves);
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub enum SceneUIRef
{
    #[default]
    None,
    Slit(usize, usize),
    Wall(usize),
    Point(usize, bool),
    Screen(bool)
}

#[derive(Debug, Clone, Default)]
pub struct SceneUIData
{
    pub selection: SceneUIRef,
    pub hover: SceneUIRef,
    pub ghost: Option<SceneSlit>,
    pub lines: Box<[(Vector2<f32>, Colour)]>
}

impl SceneUIData
{
    pub fn generate_lines(&mut self, scene: &Scene, sl: f32)
    {
        let mut data = Vec::<(Vector2<f32>, Colour)>::with_capacity(scene.walls.len() * 4);
        
        for (i, w) in scene.walls.iter().enumerate()
        {
            let n = w.dir.rotated_90();
            
            let cw = get_colour(i, None, self.selection, self.hover);
            
            for (j, s) in w.slits.iter().enumerate()
            {
                let cs = get_colour(i, Some(j), self.selection, self.hover);
            }
        }
    }
}

const NORM: Colour = Colour::rgb(1.0, 0.63529411764, 0.0);
const SELECT: Colour = Colour::rgb(1.0, 0.80784313725, 0.47058823529);
const HOVER: Colour = Colour::rgb(1.0, 0.83529411764, 0.0);
fn get_colour(i: usize, j: Option<usize>, select: SceneUIRef, hover: SceneUIRef) -> Colour
{
    match (select, j)
    {
        (SceneUIRef::Slit(a, b), Some(j)) =>
        {
            if a == i && b == j
            {
                return SELECT;
            }
        },
        (SceneUIRef::Wall(a), None) =>
        {
            if a == i
            {
                return SELECT;
            }
        },
        _ => {}
    }
    
    match (hover, j)
    {
        (SceneUIRef::Slit(a, b), Some(j)) =>
        {
            if a == i && b == j
            {
                return HOVER;
            }
        },
        (SceneUIRef::Wall(a), None) =>
        {
            if a == i
            {
                return HOVER;
            }
        },
        _ => {}
    }
    
    return NORM;
}