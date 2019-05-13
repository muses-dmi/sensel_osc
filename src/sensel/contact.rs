use super::bindings::*;

pub use super::bindings::SenselContactState as State;

const CONTACT_INVALID: u32 = State::CONTACT_INVALID as u32;
const CONTACT_START: u32 = State::CONTACT_START as u32;
const CONTACT_MOVE: u32 = State::CONTACT_MOVE as u32;
const CONTACT_END: u32 = State::CONTACT_END as u32;

impl Into<State> for u32 {
    fn into(self) -> State {
        match self {
            CONTACT_INVALID => State::CONTACT_INVALID,
            CONTACT_START => State::CONTACT_START,
            CONTACT_MOVE => State::CONTACT_MOVE,
            CONTACT_END => State::CONTACT_END,
            _ => State::CONTACT_INVALID // maybe make this unreachable?
        }
    }
}

bitflags! {
    pub struct Mask: u8 {
        const ELLIPSE = CONTACT_MASK_ELLIPSE as u8;
        const DELTAS = CONTACT_MASK_DELTAS as u8;
        const BOUNDING_BOX = CONTACT_MASK_BOUNDING_BOX as u8;
        const PEAK = CONTACT_MASK_PEAK as u8;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ellipse {
    pub orientation: f32,
    pub major_axis: f32,
    pub minor_axis: f32
}

#[derive(Clone, Copy, Debug)]
pub struct Delta {
    pub x: f32,
    pub y: f32,
    pub force: f32,
    pub area: f32
}

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32
}

#[derive(Clone, Copy, Debug)]
pub struct Peak {
    pub x: f32,
    pub y: f32,
    pub force: f32
}

#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub id: u8,
    pub state: State,
    pub x: f32,
    pub y: f32,
    pub total_force: f32,
    pub area: f32,
    pub ellipse: Option<Ellipse>,
    pub delta: Option<Delta>,
    pub bounding_box: Option<BoundingBox>,
    pub peak: Option<Peak>
}

impl From<SenselContact> for Contact {
    fn from (sensel_contact: SenselContact) -> Self {
        let SenselContact {
            content_bit_mask,
            id,
            state,
            x_pos,
            y_pos,
            total_force,
            area,
            orientation,
            major_axis,
            minor_axis,
            delta_x,
            delta_y,
            delta_force,
            delta_area,
            min_x,
            min_y,
            max_x,
            max_y,
            peak_x,
            peak_y,
            peak_force
        } = sensel_contact;

        let mask = Mask::from_bits_truncate(content_bit_mask);

        let ellipse = if mask.contains(Mask::ELLIPSE) {
            Some(Ellipse {
                orientation,
                major_axis,
                minor_axis
            })
        } else {
            None
        };

        let delta = if mask.contains(Mask::DELTAS) {
            Some(Delta {
                x: delta_x,
                y: delta_y,
                force: delta_force,
                area: delta_area
            })
        } else {
            None
        };

        let bounding_box = if mask.contains(Mask::BOUNDING_BOX) {
            Some(BoundingBox {
                min_x,
                min_y,
                max_x,
                max_y
            })
        } else {
            None
        };

        let peak = if mask.contains(Mask::PEAK) {
            Some(Peak {
                x: peak_x,
                y: peak_y,
                force: peak_force
            })
        } else {
            None
        };

        Contact {
            id,
            state: state.into(),
            x: x_pos,
            y: y_pos,
            total_force,
            area,
            ellipse,
            delta,
            bounding_box,
            peak
        }
    }
}
