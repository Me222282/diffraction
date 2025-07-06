use core::f64;

use zene_structs::{Line2, Vector, Vector2};

use crate::SL;
use super::{Scene, SceneSlit, SceneUIRef, Wall};

impl Scene
{
    pub fn set_wall_point(&mut self, i: usize, ab: bool, p: Vector2<f64>)
    {
        let max = self.walls.len() - 1;
        match (i, ab)
        {
            (0, false) => self.walls[0].set_a(p),
            (val, true) if val == max => self.walls[max].set_b(p),
            _ =>
            {
                let i = if ab { i + 1 }
                else { i };
                let j = i - 1;
                self.walls[i].set_a(p);
                self.walls[j].set_b(p);
            }
        }
    }
    
    pub fn mouse_point(&self, wp: Vector2<f64>, zoom: f32) -> SceneUIRef
    {
        let md = (SL / zoom) as f64;
        let md = md * md;
        
        // let screen_wall = Wall::new(self.env.screen.0, self.env.screen.1);
        // if wall_square_dist(wp, &screen_wall) < md
        // {
        //     return SceneUIRef::Screen(())
        // }
        
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
            close = a_dist;
            ui_ref = SceneUIRef::Screen(false);
        }
        let b_dist = wp.squared_distance(self.env.screen.1);
        if b_dist < close
        {
            // close = b_dist;
            ui_ref = SceneUIRef::Screen(true);
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
                        x2 = p.squared_distance(w.a);
                    }
                }
            }
        }
        
        return (index, x2.sqrt());
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