#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn to_vec2(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn rotate_y(self, angle: f32) -> Vec3 {
        let cos = libm::cosf(angle);
        let sin = libm::sinf(angle);

        Vec3 {
            x: self.x * cos + self.z * sin,
            y: self.y,
            z: -self.x * sin + self.z * cos,
        }
    }

    pub fn rotate_z(self, angle: f32) -> Vec3 {
        let cos = libm::cosf(angle);
        let sin = libm::sinf(angle);

        Vec3 {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
            z: self.z,
        }
    }
}

impl core::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn extend(self, z: f32) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z,
        }
    }

    #[inline]
    pub const fn cross(self, other: Self) -> f32 {
        (self.x * other.y) - (self.y * other.x)
    }
}

impl core::ops::Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

macro_rules! impl_math_ops {
    ($ty:path, $($field:ident),*) => {
        impl core::ops::Add for $ty {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field + rhs.$field,)*
                }
            }
        }

        impl core::ops::Sub for $ty {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field - rhs.$field,)*
                }
            }
        }

        impl core::ops::Mul for $ty {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field * rhs.$field,)*
                }
            }
        }

        impl core::ops::Div for $ty {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field / rhs.$field,)*
                }
            }
        }

        impl core::ops::AddAssign for $ty {
            fn add_assign(&mut self, rhs: Self) {
                $(self.$field += rhs.$field;)*
            }
        }

        impl core::ops::SubAssign for $ty {
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$field -= rhs.$field;)*
            }
        }

        impl core::ops::MulAssign for $ty {
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$field *= rhs.$field;)*
            }
        }

        impl core::ops::DivAssign for $ty {
            fn div_assign(&mut self, rhs: Self) {
                $(self.$field /= rhs.$field;)*
            }
        }

        impl core::ops::Add<f32> for $ty {
            type Output = Self;

            fn add(self, rhs: f32) -> Self::Output {
                Self {
                    $($field: self.$field + rhs,)*
                }
            }
        }

        impl core::ops::Sub<f32> for $ty {
            type Output = Self;

            fn sub(self, rhs: f32) -> Self::Output {
                Self {
                    $($field: self.$field - rhs,)*
                }
            }
        }

        impl core::ops::Mul<f32> for $ty {
            type Output = Self;

            fn mul(self, rhs: f32) -> Self::Output {
                Self {
                    $($field: self.$field * rhs,)*
                }
            }
        }

        impl core::ops::Div<f32> for $ty {
            type Output = Self;

            fn div(self, rhs: f32) -> Self::Output {
                Self {
                    $($field: self.$field / rhs,)*
                }
            }
        }

        impl core::ops::AddAssign<f32> for $ty {
            fn add_assign(&mut self, rhs: f32) {
                $(self.$field += rhs;)*
            }
        }

        impl core::ops::SubAssign<f32> for $ty {
            fn sub_assign(&mut self, rhs: f32) {
                $(self.$field -= rhs;)*
            }
        }

        impl core::ops::MulAssign<f32> for $ty {
            fn mul_assign(&mut self, rhs: f32) {
                $(self.$field *= rhs;)*
            }
        }

        impl core::ops::DivAssign<f32> for $ty {
            fn div_assign(&mut self, rhs: f32) {
                $(self.$field /= rhs;)*
            }
        }
    };
}

impl_math_ops!(Vec2, x, y);
impl_math_ops!(Vec3, x, y, z);
impl_math_ops!(crate::color::LinearRgb, r, g, b, a);
