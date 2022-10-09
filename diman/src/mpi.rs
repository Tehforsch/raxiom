use glam::DVec2;
use mpi::datatype::DatatypeRef;
use mpi::datatype::SystemDatatype;
use mpi::datatype::UserDatatype;
use mpi::ffi;
use mpi::traits::Equivalence;
use mpi::traits::FromRaw;
use once_cell::sync::Lazy;

use super::dimension::Dimension;
use super::quantity::Quantity;

unsafe impl<const D: Dimension> Equivalence for Quantity<f64, D> {
    type Out = SystemDatatype;

    fn equivalent_datatype() -> Self::Out {
        unsafe { DatatypeRef::from_raw(ffi::RSMPI_DOUBLE) }
    }
}

unsafe impl<const D: Dimension> Equivalence for Quantity<DVec2, D> {
    type Out = DatatypeRef<'static>;

    fn equivalent_datatype() -> Self::Out {
        static DATATYPE: Lazy<::mpi::datatype::UserDatatype> =
            Lazy::new(|| UserDatatype::contiguous(2, &f64::equivalent_datatype()));
        DATATYPE.as_ref()
    }
}

#[cfg(test)]
#[cfg(feature = "mpi")]
mod tests {
    use mpi::traits::Communicator;

    use crate::communication::MPI_UNIVERSE;
    use crate::units::Length;
    use crate::units::VecLength;

    #[test]
    fn pack_unpack_f64_quantity() {
        let world = MPI_UNIVERSE.world();

        let q1 = Length::meters(1.0);
        let mut q2 = Length::meters(2.0);
        let a = world.pack(&q1);
        unsafe {
            world.unpack_into(&a, &mut q2, 0);
        }
        assert_eq!(q1, q2);
    }

    #[test]
    fn pack_unpack_vec_quantity() {
        let world = MPI_UNIVERSE.world();
        let q1 = VecLength::meters(1.0, 2.0);
        let mut q2 = VecLength::meters(3.0, 4.0);
        let a = world.pack(&q1);
        unsafe {
            world.unpack_into(&a, &mut q2, 0);
        }
        assert_eq!(q1, q2);
    }
}