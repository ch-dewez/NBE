use glam::Vec3;

use crate::collision::detection::ContactManifold;

pub type SATableForEachAxesClosure<'a> = &'a mut dyn FnMut(Axis) -> bool;
pub type SATableForEachVecClosure<'a> = &'a mut dyn FnMut(Vec3) -> bool;
pub trait SATable {
    fn for_all_axes<'a>(&self, other: &dyn SATable, closure: SATableForEachAxesClosure<'a>);
    fn project_onto_axis(&self, axis: Vec3) -> (f32, f32);

    fn for_each_faces_axes<'a>(&self, closure: SATableForEachVecClosure<'a>);
    fn for_each_edges_axes<'a>(&self, closure: SATableForEachVecClosure);
}

#[derive(Default, Clone, Debug)]
pub struct SATResult {
    pub intersect: bool,
    pub mtv: Vec3,
}

#[derive(Default, Clone)]
pub enum AxisType {
    #[default]
    Face,
    Edge,
}
#[derive(Default, Clone)]
pub struct Axis {
    pub axis: Vec3,
    pub axis_type: AxisType,
}

impl From<Axis> for Vec3 {
    fn from(value: Axis) -> Self {
        value.axis
    }
}

pub fn apply_sat_and_clipping<A: SATable, B: SATable>(
    shape_a: A,
    shape_b: B,
) -> Option<ContactManifold> {
    let sat_result = sat(shape_a, shape_b);
    if !sat_result.intersect {
        None
    } else {
        println!("SATResult: {:?}", sat_result);
        //Some(get_contact_manifold_from_sat(sat_result))
        None
    }
}

pub fn sat<A: SATable, B: SATable>(shape_a: A, shape_b: B) -> SATResult {
    let mut result: Option<SATResult> = None;

    let mut overlap = f32::MAX;
    let mut overlap_axis = Vec3::ZERO;

    shape_a.for_all_axes(&shape_b, &mut |axis: Axis| {
        let (min1, max1) = shape_a.project_onto_axis(axis.axis);
        let (min2, max2) = shape_b.project_onto_axis(axis.axis);

        if max1 < min2 || max2 < min1 {
            result = Some(SATResult {
                intersect: false,
                mtv: Vec3::ZERO,
            });

            return true;
        }

        let overlap1 = max1 - min2;
        let overlap2 = max2 - min1;
        let (current_overlap, current_axis) = if overlap1 < overlap2 {
            (overlap1, -axis.axis)
        } else {
            (overlap2, axis.axis)
        };

        if overlap > current_overlap {
            overlap = current_overlap;
            overlap_axis = current_axis;
        }
        return false;
    });

    if result.is_none() {
        result = Some(SATResult {
            intersect: true,
            mtv: overlap_axis * overlap,
        });
    }

    result.unwrap()
}

pub fn get_contact_manifold_from_sat(sat: SATResult) -> ContactManifold {
    unimplemented!()
}
