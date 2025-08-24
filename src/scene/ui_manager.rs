use core::f64;

use backend::UIWall;
use num::Zero;
use zene_structs::{Line2, Vector, Vector2};

use crate::SL;
use super::{Scene, SceneSlit, SceneUIRef, Wall};

pub struct SceneWallRef<'a>
{
    scene: &'a mut Scene,
    pre: Option<usize>,
    this: usize,
    post: Option<usize>
}
impl<'a> UIWall for SceneWallRef<'a>
{
    fn set_a(&mut self, a: Vector2<f64>)
    {
        if let Some(o) = self.pre
        {
            self.scene.walls[o].set_b(a);
        }
        
        self.scene.walls[self.this].set_a(a);
    }

    fn set_b(&mut self, b: Vector2<f64>)
    {
        if let Some(o) = self.post
        {
            self.scene.walls[o].set_a(b);
        }
        
        self.scene.walls[self.this].set_b(b);
    }
    
    fn set_a_b(&mut self, a: Vector2<f64>, b: Vector2<f64>)
    {
        if let Some(o) = self.pre
        {
            self.scene.walls[o].set_b(a);
        }
        if let Some(o) = self.post
        {
            self.scene.walls[o].set_a(b);
        }
        
        self.scene.walls[self.this].set_a_b(a, b);
    }
    
    #[inline]
    fn get_a(&self) -> Vector2<f64>
    {
        return self.scene.walls[self.this].a;
    }
    #[inline]
    fn get_b(&self) -> Vector2<f64>
    {
        return self.scene.walls[self.this].b;
    }
    #[inline]
    fn get_a_b(&self) -> (Vector2<f64>, Vector2<f64>)
    {
        let w = &self.scene.walls[self.this];
        return (w.a, w.b);
    }
}

impl Scene
{
    pub fn get_ui_wall(&mut self, i: usize) -> SceneWallRef<'_>
    {
        let pre = if i == 0 { None }
            else { Some(i - 1) };
        let post = if i == (self.walls.len() - 1) { None }
            else { Some(i + 1) };
        
        return SceneWallRef {
            scene: self,
            pre,
            this: i,
            post
        };
    }
    
    pub fn set_slit_pos(&mut self, i: usize, j: usize, wp: Vector2<f64>)
    {
        let w = &mut self.walls[i];
        let hw = w.slits[j].width * 0.5;
        
        let min = match j
        {
            0 => hw,
            _ => w.slits[j - 1].get_right() + hw
        };
        let end = w.slits.len() - 1;
        let max = match j
        {
            x if x == end => w.len() - hw,
            _ => w.slits[j + 1].get_left() - hw
        };
        
        // normalised direction
        let dist = (wp - w.a).dot(w.dir);
        w.slits[j].position = dist.clamp(min, max);
    }
    
    pub fn mouse_point(&self, wp: Vector2<f64>, zoom: f32) -> SceneUIRef
    {
        let md = (SL / zoom) as f64;
        let md = md * md;
        
        let mut close = md;
        let mut ui_ref = SceneUIRef::None;
        
        let max = self.walls.len() - 1;
        for (i, w) in self.walls.iter().enumerate()
        {
            for (j, s) in w.slits.iter().enumerate()
            {
                let d = dist_to_slit(wp, s, w);
                if d < close
                {
                    close = d;
                    ui_ref = SceneUIRef::Slit(i, j);
                }
            }
            
            // prioritise slits
            if let SceneUIRef::Slit(_, _) = ui_ref { continue; }
            
            let d = wp.squared_distance(w.a);
            if d < close
            {
                close = d;
                ui_ref = SceneUIRef::Point(i, false);
                continue;
            }
            if i == max
            {
                let d = wp.squared_distance(w.b);
                if d < close
                {
                    close = d;
                    ui_ref = SceneUIRef::Point(i, true);
                    continue;
                }
            }
            let d = wall_square_dist(wp, w);
            if d < close
            {
                close = d;
                ui_ref = SceneUIRef::Wall(i);
            }
        }
        
        let a_dist = wp.squared_distance(self.env.screen.0);
        if a_dist < close
        {
            // close = a_dist;
            return SceneUIRef::ScreenPoint(false);
        }
        let b_dist = wp.squared_distance(self.env.screen.1);
        if b_dist < close
        {
            // close = b_dist;
            return SceneUIRef::ScreenPoint(true);
        }
        let screen_wall = Wall::new(self.env.screen.0, self.env.screen.1);
        let s_d = wall_square_dist(wp, &screen_wall);
        if s_d < close
        {
            // close = s_d;
            ui_ref = SceneUIRef::Screen;
        }
        
        return ui_ref;
    }
    pub fn wall_pos(&self, wp: Vector2<f64>) -> (usize, f64)
    {
        let mut close = f64::MAX;
        let mut index = 0;
        let mut x2 = 0.0_f64;
        
        for (i, w) in self.walls.iter().enumerate()
        {
            match get_segment(wp, w)
            {
                Segment::ASide =>
                {
                    let d = wp.squared_distance(w.a);
                    if d < close
                    {
                        close = d;
                        index = i;
                        x2 = 0.0;
                    }
                },
                Segment::BSide =>
                {
                    let d = wp.squared_distance(w.b);
                    if d < close
                    {
                        close = d;
                        index = i;
                        x2 = w.a.squared_distance(w.b);
                    }
                },
                Segment::Between =>
                {
                    // direction is normalised
                    let t = (wp - w.a).dot(w.dir);
                    let p = w.a + (w.dir * t);
                    let d = wp.squared_distance(p);
                    if d < close
                    {
                        close = d;
                        index = i;
                        x2 = t * t;
                    }
                }
            }
        }
        
        return (index, x2.sqrt());
    }
    
    pub fn get_ref_pos(&self, sur: SceneUIRef) -> Vector2<f64>
    {
        return match sur
        {
            SceneUIRef::None => Vector2::zero(),
            SceneUIRef::Slit(i, j) =>
            {
                let w = &self.walls[i];
                return w.slits[j].get_position(&w);
            },
            SceneUIRef::Wall(i) => self.walls[i].a, 
            SceneUIRef::Point(i, ab) =>
            {
                return if ab { self.walls[i].b }
                    else     { self.walls[i].a };
            },
            SceneUIRef::ScreenPoint(lr) =>
            {
                return if lr { self.env.screen.1 }
                    else     { self.env.screen.0 };
            },
            SceneUIRef::Screen => self.env.screen.0
        };
    }
}

fn wall_square_dist(p: Vector2<f64>, w: &Wall) -> f64
{
    if !in_segment(p, w)
    {
        // return p.squared_distance(w.a).min(p.squared_distance(w.b));
        return f64::MAX;
    }
    
    return Line2::new(w.dir, w.a).squared_distance_from_point(p);
}

fn in_segment(p: Vector2<f64>, w: &Wall) -> bool
{
    let dot_a = w.dir.dot(p - w.a);
    let dot_b = w.dir.dot(p - w.b);
    
    return dot_a.is_sign_positive() ^ dot_b.is_sign_positive();
}

fn dist_to_slit(wp: Vector2<f64>, s: &SceneSlit, w: &Wall) -> f64
{
    let p = s.get_position(w);
    let off = w.dir * (s.width * 0.5);
    let a = p - off;
    let b = p + off;
    
    let dot_a = w.dir.dot(wp - a);
    let dot_b = w.dir.dot(wp - b);
    
    if !(dot_a.is_sign_positive() ^ dot_b.is_sign_positive())
    {
        return p.squared_distance(wp);
    }
    
    return Line2::new(w.dir, w.a).squared_distance_from_point(wp);
}

enum Segment
{
    ASide,
    BSide,
    Between
}
fn get_segment(p: Vector2<f64>, w: &Wall) -> Segment
{
    let dot_a = w.dir.dot(p - w.a);
    let dot_b = w.dir.dot(p - w.b);
    
    return match (dot_a.is_sign_positive(), dot_b.is_sign_positive())
    {
        (true, true) => Segment::BSide,
        (false, false) => Segment::ASide,
        (true, false) => Segment::Between,
        (false, true) => Segment::Between
    };
}