
use tracer::primitives::{HasColor, HasCenter, Intersectable, HasNormal};
use tracer::primitives::Primitive;
use tracer::primitives::BoundingBox;
use tracer::utils::ray::Ray;
use tracer::utils::color::Color;

use nalgebra::{distance, Point3, Vector3};
use nalgebra::core::Unit;

use std::f32;
use std::usize;
use std::cmp::Ordering;

struct BVHNode
{
   pub bbox: BoundingBox,
   pub primitive: Option<Primitive>,
   pub left: Option<Box<BVHNode>>,
   pub right: Option<Box<BVHNode>>
}

impl BVHNode
{
   pub fn new_leaf(primitive: Primitive) -> BVHNode
   {
      let n = BVHNode {
         bbox: BoundingBox::new(&primitive),
         primitive: Some(primitive),
         left: None,
         right: None
      };

      return n;
   }

   pub fn new(left: Box<BVHNode>, right: Box<BVHNode>) -> BVHNode
   {

      let n = BVHNode {
         bbox: BoundingBox::new_from(&left.bbox, &right.bbox),
         primitive: None,
         left: Some(left),
         right: Some(right)
      };

      return n;
   }

   pub fn intersect(&self, ray: &Ray) -> Option<HitInfo>
   {
      let bbox_inter = self.bbox.intersect(ray);
      match bbox_inter
      {
         Some(_) => {
            match self.primitive
            {
               Some(ref p) => 
               {
                  let inter_dist = p.intersect(ray);
                  match inter_dist
                  {
                     Some(x) => {
                        if x < 1.0 //hit ourselves
                        {
                           return None;
                        }

                        let p_hit = ray.origin + x * ray.direction.as_ref();
                        let h_info = HitInfo {
                           color: p.get_color(),
                           normal: p.get_normal(p_hit),
                           p_hit: p_hit,
                           distance: x
                        };

                        return Some(h_info);
                     },
                     None => {
                        return None;
                     }
                  }
               },
               None => 
               {
                  let mut left_inter = None;
                  let mut right_inter = None;
                  match self.left
                  {
                     Some(ref left) => {
                        left_inter = left.intersect(ray);

                     }
                     None => {

                     }
                  }
                  match self.right
                  {
                     Some(ref right) => {
                        right_inter = right.intersect(ray);

                     }
                     None => {

                     }
                  }

                  if left_inter.is_none() && right_inter.is_none()
                  {
                     return None;
                  }
                  else if left_inter.is_none()
                  {
                     return right_inter;
                  }
                  else if right_inter.is_none()
                  {
                     return left_inter;
                  }
                  else
                  {
                     if left_inter.partial_cmp(&right_inter) == Some(Ordering::Less)
                     {
                        return left_inter;
                     }
                     else
                     {
                        return right_inter;
                     }

                  }
               }
            }
         },
         None => {
         }
      }
      
         
      return None;
   }
}

pub struct BoundingVolumeHierarchy
{
   root: Box<BVHNode>
}

pub struct HitInfo
{
   pub color: Color,
   pub normal: Unit<Vector3<f32>>,
   pub p_hit: Point3<f32>,
   pub distance: f32
}

impl PartialEq for HitInfo
{
   fn eq(&self, other: &HitInfo) -> bool {
        self.distance == other.distance
    }
}
impl PartialOrd for HitInfo
{
   fn partial_cmp(&self, other: &HitInfo) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl BoundingVolumeHierarchy
{
   pub fn new(primitives: Vec<Primitive>) -> BoundingVolumeHierarchy
   {
      // 1. create 1 node per primitive
      // 2. group node together (closest)
      // 3. When there is only 1 node, set as root
      let mut nodes: Vec<Box<BVHNode>> = Vec::with_capacity(primitives.len());
      for p in primitives
      {
         let node = Box::new(BVHNode::new_leaf(p));
         nodes.push(node);
      }

      while nodes.len() > 1
      {
         let mut merged_nodes = Vec::new();

         // take all nodes and merge them to the closest other node
         while nodes.len() > 1
         {
            let mut min_dist = f32::MAX;
            let mut min_dist_idx = usize::MAX;
            let last = nodes.pop().unwrap();
            let last_center = last.bbox.get_center();
            for i in 0..nodes.len()
            {
               //find closest
               let d = distance(&last_center, &nodes[i].bbox.get_center());
               if d < min_dist
               {
                 min_dist = d;
                 min_dist_idx = i;
               }
            }

            let closest = nodes.swap_remove(min_dist_idx);
            let t = Box::new(BVHNode::new(last, closest));
            merged_nodes.push(t);
         }

         if nodes.len() == 1
         {
            merged_nodes.push(nodes.pop().unwrap());
         }

         nodes = merged_nodes;
      }

      assert!(nodes.len() == 1);
      let m = BoundingVolumeHierarchy {
         root: nodes.pop().unwrap()
      };

      return m;
   }

   pub fn intersect(&self, ray: &Ray) -> Option<HitInfo>
   {
      return self.root.intersect(ray);
   }
}
