use core::f64;

use backend::{Colour, EMEnv, Slit, Wave};
use num::Zero;
use zene_structs::{Vector2, Vector3, Vector};

use crate::wave_data::WaveData;

#[derive(Debug, Clone, Default, Copy)]
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
    pub fn new(a: Vector2<f64>, b: Vector2<f64>) -> Self
    {
        return Self {
            a,
            b,
            dir: (b - a).normalised(),
            slits: Vec::new()
        };
    }
    
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
    
    pub fn split(&mut self, pos: f64) -> Wall
    {
        let index = self.slits.iter().position(|s| s.position > pos);
        
        let b = self.b;
        self.b = self.a + (self.dir * pos);
        
        return match index
        {
            Some(at) =>
            {
                let v = self.slits.split_off(at);
                return Wall { a: self.b, b, dir: self.dir, slits: v };
            },
            None => Wall { a: self.b, b, dir: self.dir, slits: Vec::new() }
        };
    }
    
    pub fn insert_slit(&mut self, width: f64, position: f64) -> usize
    {
        let index = self.slits.iter().position(|s| s.position > position);
        let slit = SceneSlit { width, position };
        match index
        {
            Some(i) =>
            {
                self.slits.insert(i, slit);
                return i;
            },
            // if none was found, slit is at the end
            None =>
            {
                let r = self.slits.len();
                self.slits.push(slit);
                return r;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scene
{
    env: EMEnv<f64>,
    waves: Box<[Wave<f64>]>,
    walls: Vec<Wall>
}

impl Default for Scene
{
    fn default() -> Self
    {
        return Self {
            env: EMEnv::new(
                Vector2::new(-2e9, 2e9),
                Vector2::new(2e9, 2e9)),
            waves: Default::default(),
            walls: vec![Wall {
                a: Vector2::new(-1e9, 0.0),
                b: Vector2::new(1e9, 0.0),
                dir: Vector2::new(1.0, 0.0),
                slits: vec![SceneSlit { width: 1560.0, position: 1e9}]
            }]
        };
    }
}

impl Scene
{
    pub fn compute_waves(&mut self, wd: &WaveData)
    {
        // skip waves with no amplitude
        let waves = wd.spectrum.iter().zip(wd.wave_map.iter()).filter(|(a, _)|
        {
            return a[0] > 0.0;
        }).map(|(a, w)|
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
    pub lines: Vec<(Vector2<f32>, Colour)>
}

fn as_32(_64: Vector2<f64>) -> Vector2<f32>
{
    return Vector2::new(_64.x as f32, _64.y as f32);
}

impl SceneUIData
{
    pub fn generate_lines(&mut self, scene: &Scene, sl: f32)
    {
        let mut data = Vec::<(Vector2<f32>, Colour)>::with_capacity(scene.walls.len() * 4);
        
        let ghost = self.ghost.unwrap_or(SceneSlit { width: f64::NAN, position: f64::NAN });
        
        for (i, w) in scene.walls.iter().enumerate()
        {
            let d = w.dir;
            let n = w.dir.rotated_90() * (sl as f64);
            
            let g_here = match self.hover
            {
                SceneUIRef::Wall(a) => a == i,
                _ => false
            };
            
            let cw = get_colour(i, None, self.selection, self.hover);
            let mut last = w.a;
            
            let mut insert = |slit: &SceneSlit, c: Colour|
            {
                let p = slit.get_position(w);
                let x_off = d * (slit.width * 0.5);
                
                let a = p - x_off;
                let b = p + x_off;
                
                // wall between slits
                data.push((as_32(last), cw));
                data.push((as_32(a), cw));
                last = b;
                
                data.push((as_32(a + n), c));
                data.push((as_32(a - n), c));
                data.push((as_32(b + n), c));
                data.push((as_32(b - n), c));
            };
            
            // assume slits sorted by position
            for (j, s) in w.slits.iter().enumerate()
            {
                // insert before - always false if ghost is NAN
                if g_here && ghost.position < s.position
                {
                    insert(&ghost, GHOST);
                }
                
                let cs = get_colour(i, Some(j), self.selection, self.hover);
                insert(s, cs);
            }
            
            data.push((as_32(last), cw));
            data.push((as_32(w.b), cw));
        }
        
        self.lines = data;
    }
}

const NORM: Colour = Colour::rgb(1.0, 0.63529411764, 0.0);
const SELECT: Colour = Colour::rgb(1.0, 0.80784313725, 0.47058823529);
const HOVER: Colour = Colour::rgb(1.0, 0.83529411764, 0.0);
const GHOST: Colour = Colour::new(1.0, 0.83529411764, 0.0, 0.5);
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