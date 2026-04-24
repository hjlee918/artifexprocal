use crate::types::{XYZ, WhitePoint};

/// Bradford chromatic adaptation matrix (from CIECAM02)
const BRADFORD_M: [[f64; 3]; 3] = [
    [0.8951, 0.2664, -0.1614],
    [-0.7502, 1.7135, 0.0367],
    [0.0389, -0.0685, 1.0296],
];

const BRADFORD_M_INV: [[f64; 3]; 3] = [
    [0.9869929, -0.1470543, 0.1599627],
    [0.4323053, 0.5183603, 0.0492912],
    [-0.0085287, 0.0400428, 0.9684867],
];

/// Apply Bradford chromatic adaptation from source white point to destination white point
pub fn bradford_adapt(xyz: &XYZ, source: WhitePoint, dest: WhitePoint) -> XYZ {
    if source == dest {
        return *xyz;
    }

    let src_wp = source.to_xyz();
    let dst_wp = dest.to_xyz();

    let src_lms = mat_vec_mul(&BRADFORD_M, &[src_wp.x, src_wp.y, src_wp.z]);
    let dst_lms = mat_vec_mul(&BRADFORD_M, &[dst_wp.x, dst_wp.y, dst_wp.z]);

    let scale = [
        dst_lms[0] / src_lms[0],
        dst_lms[1] / src_lms[1],
        dst_lms[2] / src_lms[2],
    ];

    let lms = mat_vec_mul(&BRADFORD_M, &[xyz.x, xyz.y, xyz.z]);
    let lms_adapted = [lms[0] * scale[0], lms[1] * scale[1], lms[2] * scale[2]];
    let adapted = mat_vec_mul(&BRADFORD_M_INV, &lms_adapted);

    XYZ {
        x: adapted[0],
        y: adapted[1],
        z: adapted[2],
    }
}

fn mat_vec_mul(m: &[[f64; 3]; 3], v: &[f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}
