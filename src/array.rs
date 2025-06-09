use astronomy::units::{Quantity, QuantityError, Unit};
use ndarray::Array1;

#[derive(Debug, Clone, PartialEq)]
pub struct GWArray {
    quantity: Quantity,
    pub name: Option<String>,
}

impl GWArray {
    pub fn new(value: Array1<f64>, unit: Unit, name: Option<String>) -> Self {
        GWArray {
            quantity: Quantity::new(value, unit),
            name,
        }
    }

    pub fn value(&self) -> &Array1<f64> {
        &self.quantity.value
    }

    pub fn unit(&self) -> &Unit {
        &self.quantity.unit
    }

    pub fn to(&self, target_unit: &Unit) -> Result<GWArray, QuantityError> {
        let converted_quantity = self.quantity.to(target_unit)?;
        Ok(GWArray::new(
            converted_quantity.value,
            converted_quantity.unit,
            self.name.clone(),
        ))
    }
}

use std::ops::Add;

impl Add for GWArray {
    type Output = Result<Self, QuantityError>;
    fn add(self, rhs: Self) -> Self::Output {
        let added_quantity = (self.quantity + rhs.quantity)?;
        Ok(GWArray::new(
            added_quantity.value,
            added_quantity.unit,
            self.name.clone(),
        ))
    }
}

// Some tests
#[cfg(test)]
mod tests {
    use super::*;
    use astronomy::units::{CENTIMETER, METER, SECOND};
    use ndarray::array;

    #[test]
    fn test_gwarray_creation() {
        let value = array![1.0, 2.0, 3.0];
        let gw_array = GWArray::new(value, METER, Some("Test Array".to_string()));
        assert_eq!(gw_array.value(), &array![1.0, 2.0, 3.0]);
        assert_eq!(gw_array.unit(), &METER);
        assert_eq!(gw_array.name, Some("Test Array".to_string()));
    }

    #[test]
    fn test_gwarray_creation_and_name() {
        let gw_array = GWArray::new(
            array![1.0, 2.0, 3.0],
            METER.clone(),
            Some("Test Array".to_string()),
        );
        assert_eq!(gw_array.name, Some("Test Array".to_string()));
        assert_eq!(gw_array.unit(), &METER);
        assert_eq!(gw_array.value(), &array![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_gw_array_to_method() {
        let gw_array = GWArray::new(
            array![1.0, 2.0, 3.0],
            METER.clone(),
            Some("Test Array".to_string()),
        );
        let converted_array = gw_array.to(&CENTIMETER).unwrap();

        assert_eq!(converted_array.value(), &array![100.0, 200.0, 300.0]);
        assert_eq!(converted_array.unit(), &CENTIMETER);
        assert_eq!(converted_array.name, Some("Test Array".to_string()));
    }

    #[test]
    fn test_gw_array_addition() {
        let gw_array1 = GWArray::new(array![1.0, 2.0, 3.0], METER.clone(), None);
        let gw_array2 = GWArray::new(array![4.0, 5.0, 6.0], METER.clone(), None);
        let result = gw_array1 + gw_array2;

        assert!(result.is_ok());
        let added_array = result.unwrap();
        assert_eq!(added_array.value(), &array![5.0, 7.0, 9.0]);
        assert_eq!(added_array.unit(), &METER);
    }

    #[test]
    fn test_gw_array_addition_with_different_units_same_dimension() {
        let gw_array1 = GWArray::new(array![1.0, 2.0, 3.0], METER.clone(), None);
        let gw_array2 = GWArray::new(array![100.0, 200.0, 300.0], CENTIMETER.clone(), None);
        let result = gw_array1 + gw_array2;

        assert!(result.is_err());
        if let Err(QuantityError::IncompatibleAddition { lhs, rhs }) = result {
            assert_eq!(lhs, "m");
            assert_eq!(rhs, "cm");
        } else {
            panic!("Expected incompatible addition error");
        }
    }

    #[test]
    fn test_gw_array_addition_with_different_units_different_dimension() {
        let gw_array1 = GWArray::new(array![1.0, 2.0, 3.0], METER.clone(), None);
        let gw_array2 = GWArray::new(array![100.0, 200.0, 300.0], SECOND.clone(), None);
        let result = gw_array1 + gw_array2;

        assert!(result.is_err());
        if let Err(QuantityError::IncompatibleAddition { lhs, rhs }) = result {
            assert_eq!(lhs, "m");
            assert_eq!(rhs, "s");
        } else {
            panic!("Expected incompatible addition error");
        }
    }
}
