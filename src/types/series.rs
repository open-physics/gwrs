use crate::detector::channel::Channel;
use crate::types::array::GWArray;
use astronomy::time::Time;
use astronomy::units::{Quantity, QuantityError, Unit};
use ndarray::Array1;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    array_data: GWArray,
    x0: Option<Quantity>,
    dx: Option<Quantity>,
    _xindex_cache: Option<Quantity>,
}

pub struct SeriesBuilder {
    value: Option<Array1<f64>>,
    unit: Option<Unit>,
    name: Option<String>,
    epoch: Option<Time>,
    channel: Option<Channel>,
    x0: Option<Quantity>,
    dx: Option<Quantity>,
    xindex: Option<Quantity>,
}

impl SeriesBuilder {
    pub fn new() -> Self {
        SeriesBuilder {
            value: None,
            unit: None,
            name: None,
            epoch: None,
            channel: None,
            x0: None,
            dx: None,
            xindex: None,
        }
    }

    pub fn value(mut self, value: Array1<f64>) -> Self {
        self.value = Some(value);
        self
    }

    pub fn unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn epoch(mut self, epoch: Time) -> Self {
        self.epoch = Some(epoch);
        self
    }

    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn x0(mut self, x0: Quantity) -> Self {
        self.x0 = Some(x0);
        self
    }

    pub fn dx(mut self, dx: Quantity) -> Self {
        self.dx = Some(dx);
        self
    }

    pub fn xindex(mut self, xindex: Quantity) -> Self {
        self.xindex = Some(xindex);
        self
    }

    /// Build the Series instance, ensuring all required fields are set
    pub fn build(self) -> Result<Series, QuantityError> {
        let value = self.value.ok_or_else(|| {
            QuantityError::InvalidQuantity("Value is required to build Series".to_string())
        })?;
        let array_data = GWArray::new(value, self.unit, self.name, self.epoch, self.channel);
        let data_len = array_data.value().len();
        let resolved_index = if let Some(index_quantity) = self.xindex {
            // If xindex is explicitly provided, use it directly
            if index_quantity.value.len() != data_len {
                return Err(QuantityError::MismatchError(format!(
                    "Index length ({}) must match value length ({})",
                    index_quantity.value.len(),
                    data_len
                )));
            }
            Some(index_quantity)
        } else if let (Some(start_quantity), Some(step_quantity)) =
            (self.x0.as_ref(), self.dx.as_ref())
        {
            // If x0 and dx are provided, calculate the index
            if start_quantity.value.len() != 1 || step_quantity.value.len() != 1 {
                return Err(QuantityError::MismatchError(
                    "x0 and dx must be single-value quantities".to_string(),
                ));
            }
            if start_quantity.unit.dimensions != step_quantity.unit.dimensions {
                return Err(QuantityError::IncompatibleUnits {
                    from: start_quantity.unit.name.to_string(),
                    to: step_quantity.unit.name.to_string(),
                });
            }
            // Convert dx to the unit of x0
            let converted_dx = step_quantity.to(&start_quantity.unit)?.value[0];
            let start_value = start_quantity.value[0];
            let mut x_values = Array1::zeros(data_len);
            for i in 0..data_len {
                x_values[i] = start_value + i as f64 * converted_dx;
            }
            Some(Quantity::new(x_values, start_quantity.unit.clone()))
        } else {
            None
        };

        Ok(Series::new_internal(
            array_data,
            self.x0,
            self.dx,
            resolved_index,
        ))
    }
}

impl Default for SeriesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Private constructor for Series, only accessible through SeriesBuilder
impl Series {
    fn new_internal(
        array_data: GWArray,
        x0: Option<Quantity>,
        dx: Option<Quantity>,
        _xindex_cache: Option<Quantity>,
    ) -> Self {
        Series {
            array_data,
            x0,
            dx,
            _xindex_cache,
        }
    }

    // Delegated methods to access GWArray functionality
    // Public methods to access the underlying GWArray
    pub fn value(&self) -> &Array1<f64> {
        self.array_data.value()
    }
    pub fn unit(&self) -> &Unit {
        self.array_data.unit()
    }
    pub fn get_name(&self) -> Option<&str> {
        self.array_data.get_name()
    }
    pub fn get_epoch(&self) -> Option<Time> {
        self.array_data.get_epoch()
    }
    pub fn get_channel(&self) -> Option<&Channel> {
        self.array_data.get_channel()
    }
    // Series specific methods
    pub fn get_x0(&self) -> Option<&Quantity> {
        self.x0.as_ref()
    }
    pub fn get_dx(&self) -> Option<&Quantity> {
        self.dx.as_ref()
    }
    pub fn get_xindex(&self) -> Option<&Quantity> {
        self._xindex_cache.as_ref()
    }
    pub fn get_xunit(&self) -> Option<&Unit> {
        if let Some(xindex_quantity) = self.get_xindex() {
            Some(&xindex_quantity.unit)
        } else if let Some(x0_quantity) = self.get_x0() {
            Some(&x0_quantity.unit)
        } else if let Some(dx_quantity) = self.get_dx() {
            Some(&dx_quantity.unit)
        } else {
            None
        }
    }
}

// Helper to propagate metadata for Series after an arithmetic operation
fn propagate_metadata_series(result_quantity: Quantity, lhs: &Series, rhs: &Series) -> Series {
    // Attempt to get metdata from the left-hand side (lhs) Series, falling back to the right-hand side (rhs) if necessary
    let new_name = lhs.array_data.name.clone().or(rhs.array_data.name.clone());
    let new_epoch = lhs.array_data.epoch.or(rhs.array_data.epoch);
    let new_channel = lhs
        .array_data
        .channel
        .clone()
        .or(rhs.array_data.channel.clone());
    let x0_clone = lhs.x0.clone().or(rhs.x0.clone());
    let dx_clone = lhs.dx.clone().or(rhs.dx.clone());
    let xindex_clone = lhs._xindex_cache.clone().or(rhs._xindex_cache.clone());

    let data_len = result_quantity.value.len();
    let re_derived_xindex = if let Some(index_quantity) = xindex_clone.clone() {
        if index_quantity.value.len() == data_len {
            Some(index_quantity)
        } else if let (Some(start_quantity), Some(step_quantity)) =
            (x0_clone.as_ref(), dx_clone.as_ref())
        {
            if start_quantity.value.len() != 1 || step_quantity.value.len() != 1 {
                None
            } else {
                let converted_dx = step_quantity
                    .to(&start_quantity.unit)
                    .expect("Unit conversion error in propagate_metadata_series")
                    .value[0];
                let start_value = start_quantity.value[0];
                let mut x_values = Array1::zeros(data_len);
                for i in 0..data_len {
                    x_values[i] = start_value + i as f64 * converted_dx;
                }
                Some(Quantity::new(x_values, start_quantity.unit.clone()))
            }
        } else {
            None
        }
    } else if let (Some(start_quantity), Some(step_quantity)) =
        (x0_clone.as_ref(), dx_clone.as_ref())
    {
        if start_quantity.value.len() != 1 || step_quantity.value.len() != 1 {
            None
        } else {
            let converted_dx = step_quantity
                .to(&start_quantity.unit)
                .expect("Unit conversion error in propagate_metadata_series")
                .value[0];
            let start_value = start_quantity.value[0];
            let mut x_values = Array1::zeros(data_len);
            for i in 0..data_len {
                x_values[i] = start_value + i as f64 * converted_dx;
            }
            Some(Quantity::new(x_values, start_quantity.unit.clone()))
        }
    } else {
        None
    };

    Series::new_internal(
        GWArray::new(
            result_quantity.value,
            Some(result_quantity.unit),
            new_name,
            new_epoch,
            new_channel,
        ),
        x0_clone,
        dx_clone,
        re_derived_xindex,
    )
}
// --- Implementing Traits for `Series` (Arithmetic Operations) ---
impl Add for Series {
    type Output = Result<Self, QuantityError>;
    fn add(self, rhs: Self) -> Self::Output {
        let added_array = (self.array_data.clone() + rhs.array_data.clone())?;
        Ok(propagate_metadata_series(added_array.quantity, &self, &rhs))
    }
}
impl Div for Series {
    type Output = Result<Self, QuantityError>;
    fn div(self, rhs: Self) -> Self::Output {
        let divided_array = (self.array_data.clone() / rhs.array_data.clone())?;
        Ok(propagate_metadata_series(
            divided_array.quantity,
            &self,
            &rhs,
        ))
    }
}
impl Mul for Series {
    type Output = Result<Self, QuantityError>;
    fn mul(self, rhs: Self) -> Self::Output {
        let multiplied_array = (self.array_data.clone() * rhs.array_data.clone())?;
        Ok(propagate_metadata_series(
            multiplied_array.quantity,
            &self,
            &rhs,
        ))
    }
}
impl Sub for Series {
    type Output = Result<Self, QuantityError>;
    fn sub(self, rhs: Self) -> Self::Output {
        let subtracted_array = (self.array_data.clone() - rhs.array_data.clone())?;
        Ok(propagate_metadata_series(
            subtracted_array.quantity,
            &self,
            &rhs,
        ))
    }
}

// --- Tests for `Series` ---
// --- Test Module ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector;
    use astronomy::time::Time;
    use astronomy::units::{Dimension, QuantityError, Unit, UnitProduct};
    use astronomy::units::{JOULE, METER, SECOND};
    use ndarray::array;

    #[test]
    fn test_series_creation_x0_dx() {
        let unit_w = Unit::new(
            "W",
            1.0,
            UnitProduct::from_components(&[
                (Dimension::Mass, 1),
                (Dimension::Length, 2),
                (Dimension::Time, -3),
            ]),
        );
        let x0_qty = Quantity::new(array![0.0], unit_w.clone());
        let dx_qty = Quantity::new(array![2.0], unit_w.clone());
        let test_channel =
            detector::channel::Channel::new("TEST_CHANNEL", None, None, None, None, None, None)
                .unwrap();

        let data = SeriesBuilder::new()
            .value(array![1.0, 2.0, 3.0, 2.0, 4.0, 3.0])
            .unit(METER.clone())
            .x0(x0_qty.clone())
            .dx(dx_qty.clone())
            .name("Displacement".to_string())
            .epoch(Time::from_gps_seconds(0.0))
            .channel(test_channel)
            .build()
            .unwrap();

        assert_eq!(data.value(), &array![1.0, 2.0, 3.0, 2.0, 4.0, 3.0]);
        assert_eq!(data.unit(), &METER);
        assert_eq!(data.get_name(), Some("Displacement"));
        assert_eq!(data.get_x0(), Some(&x0_qty));
        assert_eq!(data.get_dx(), Some(&dx_qty));
        assert_eq!(
            data.get_xindex().unwrap().value,
            &array![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
        );
        assert_eq!(data.get_xindex().unwrap().unit, unit_w);
        assert_eq!(data.get_xunit().unwrap(), &unit_w);
        assert_eq!(data.get_channel().unwrap().get_name(), "TEST_CHANNEL");

        println!("Series Debug (x0/dx): {:?}", data);
    }

    #[test]
    fn test_series_creation_explicit_xindex() {
        let unit_s = SECOND.clone();
        let xindex_qty = Quantity::new(array![0.0, 1.0, 2.0, 3.0, 4.0], unit_s.clone());
        let index_chan =
            detector::channel::Channel::new("INDEX_CHAN", Some(1.0), None, None, None, None, None)
                .unwrap();

        let data = SeriesBuilder::new()
            .value(array![10.0, 20.0, 30.0, 40.0, 50.0])
            .unit(JOULE.clone())
            .xindex(xindex_qty.clone()) // Explicit xindex
            .name("Energy Series".to_string())
            .epoch(Time::from_gps_seconds(100.0))
            .channel(index_chan)
            .build()
            .unwrap();

        assert_eq!(data.value(), &array![10.0, 20.0, 30.0, 40.0, 50.0]);
        assert_eq!(data.unit(), &JOULE);
        assert_eq!(data.get_name(), Some("Energy Series"));
        assert_eq!(data.get_x0(), None);
        assert_eq!(data.get_dx(), None);
        assert_eq!(
            data.get_xindex().unwrap().value,
            &array![0.0, 1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(data.get_xindex().unwrap().unit, unit_s);
        assert_eq!(data.get_xunit().unwrap(), &unit_s);
        assert_eq!(data.get_channel().unwrap().get_name(), "INDEX_CHAN");

        println!("Series Debug (explicit xindex): {:?}", data);
    }

    #[test]
    fn test_series_add_propagation() {
        let unit_s = SECOND.clone();
        let xindex1_qty = Quantity::new(array![0.0, 1.0, 2.0], unit_s.clone());
        let xindex2_qty = Quantity::new(array![0.0, 1.0, 2.0], unit_s.clone());

        let chan1 =
            detector::channel::Channel::new("CHAN1", None, None, None, None, None, None).unwrap();
        let chan2 =
            detector::channel::Channel::new("CHAN2", None, None, None, None, None, None).unwrap();

        let s1 = SeriesBuilder::new()
            .value(array![1.0, 2.0, 3.0])
            .unit(METER.clone())
            .xindex(xindex1_qty.clone())
            .name("Series1".to_string())
            .epoch(Time::from_gps_seconds(1000.0))
            .channel(chan1)
            .build()
            .unwrap();

        let s2 = SeriesBuilder::new()
            .value(array![10.0, 20.0, 30.0])
            .unit(METER.clone())
            .xindex(xindex2_qty.clone())
            .name("Series2".to_string())
            .epoch(Time::from_gps_seconds(1000.0))
            .channel(chan2)
            .build()
            .unwrap();

        let sum_s = (s1.clone() + s2).unwrap();

        assert_eq!(sum_s.value(), &array![11.0, 22.0, 33.0]);
        assert_eq!(sum_s.unit(), &METER);
        assert_eq!(sum_s.get_name(), Some("Series1")); // From LHS
        assert_eq!(sum_s.get_epoch(), Some(Time::from_gps_seconds(1000.0))); // From LHS
        assert_eq!(sum_s.get_xindex().unwrap(), &xindex1_qty); // From LHS
        assert_eq!(sum_s.get_xunit().unwrap(), &unit_s);
        assert_eq!(sum_s.get_channel().unwrap().get_name(), "CHAN1"); // From LHS
    }

    #[test]
    fn test_series_add_propagation_with_rhs_fallback() {
        // s1 has some metadata as None, s2 has them
        let s1 = SeriesBuilder::new()
            .value(array![1.0])
            .unit(METER.clone())
            .name("LHS_Name".to_string()) // Will be taken from LHS
            // epoch: None
            // channel: None
            .build()
            .unwrap();

        let s2_channel =
            detector::channel::Channel::new("RHS_CHANNEL", None, None, None, None, None, None)
                .unwrap();
        let s2 = SeriesBuilder::new()
            .value(array![2.0])
            .unit(METER.clone())
            // name: None
            .epoch(Time::from_gps_seconds(200.0)) // Will be taken from RHS
            .channel(s2_channel.clone()) // Will be taken from RHS
            .build()
            .unwrap();

        let sum_s = (s1.clone() + s2.clone()).unwrap();

        assert_eq!(sum_s.value(), &array![3.0]);
        assert_eq!(sum_s.get_name(), Some("LHS_Name")); // LHS preferred
        assert_eq!(sum_s.get_epoch(), s2.get_epoch()); // RHS used as fallback
        assert_eq!(
            sum_s.get_channel().unwrap().get_name(),
            s2.get_channel().unwrap().get_name()
        ); // RHS used as fallback

        // Test with both names None
        let s3 = SeriesBuilder::new()
            .value(array![1.0])
            .unit(METER.clone())
            .build()
            .unwrap();
        let s4 = SeriesBuilder::new()
            .value(array![2.0])
            .unit(METER.clone())
            .build()
            .unwrap();
        let sum_s_none_names = (s3 + s4).unwrap();
        assert_eq!(sum_s_none_names.get_name(), None); // Still None
    }

    #[test]
    fn test_series_x0_dx_incompatible_units() {
        let x0_qty = Quantity::new(array![0.0], METER.clone());
        let dx_qty = Quantity::new(array![1.0], SECOND.clone());

        let result = SeriesBuilder::new()
            .value(array![1.0, 2.0])
            .unit(JOULE.clone())
            .x0(x0_qty)
            .dx(dx_qty)
            .build();

        assert!(result.is_err());
        if let QuantityError::IncompatibleUnits { from, to } = result.unwrap_err() {
            assert_eq!(from, "m");
            assert_eq!(to, "s");
        } else {
            panic!("Expected UnitMismatch error");
        }
    }

    #[test]
    fn test_series_length_mismatch() {
        let xindex_qty = Quantity::new(array![0.0, 1.0], SECOND.clone()); // Length 2
        let result = SeriesBuilder::new()
            .value(array![1.0, 2.0, 3.0]) // Length 3
            .unit(JOULE.clone())
            .xindex(xindex_qty)
            .build();

        assert!(result.is_err());
        if let QuantityError::MismatchError(msg) = result.unwrap_err() {
            assert!(msg.contains("Index length (2) must match value length (3)"));
        } else {
            panic!("Expected MismatchError");
        }
    }

    #[test]
    fn test_series_missing_value() {
        let result = SeriesBuilder::new().unit(METER.clone()).build();
        assert!(result.is_err());
        if let QuantityError::InvalidQuantity(msg) = result.unwrap_err() {
            assert_eq!(msg, "Value is required to build Series");
        } else {
            panic!("Expected MissingArgument error");
        }
    }
}
