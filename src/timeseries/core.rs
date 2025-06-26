use std::ops::{Add, Div, Mul, Sub};

use crate::detector::channel::Channel;
use crate::types::series::{Series, SeriesBuilder};
use astronomy::time::Time;
use astronomy::units::{HERTZ, Quantity, QuantityError, SECOND, Unit, UnitProduct};
use ndarray::{Array1, array};

#[derive(Debug, Clone, PartialEq)]
pub struct TimeSeriesBase {
    // TimeSeriesBase is a base class for time series data structures.
    // It has a Series, which in turn has a GWArray.
    series_data: Series,
    // No new direct fields are strictly needed here,
    // as it re-interprets Series's x-axis fields.
    // The `epoch` and `sample_rate` are computed properties.
}

/// Builder for TimeSeriesBase
///
/// This builder allows for the construction of a TimeSeriesBase instance
/// with a Series, which is the core data structure for time series data.
/// It handles the specific time-domain arguments (`to`, `dt`, `sample_rate`, `times`) and
/// maps them to the underlying `SeriesBuilder`'s `x0`, `dx`, `xindex` fields.
pub struct TimeSeriesBaseBuilder {
    value: Option<Array1<f64>>,
    unit: Option<Unit>,
    name: Option<String>,
    channel: Option<Channel>,
    // Time-domain specific fields
    t0: Option<Time>,              // GPS epoch, directly using astronomy::Time
    dt: Option<Quantity>,          // time between samples
    sample_rate: Option<Quantity>, // samples per second
    times: Option<Quantity>,       //explicit array of times

    // Intermediate/fallback epoch/t0 (aliases)
    _raw_t0_float: Option<f64>, // For direct float t0 input
}

impl TimeSeriesBaseBuilder {
    pub fn new() -> Self {
        TimeSeriesBaseBuilder {
            value: None,
            unit: None,
            name: None,
            channel: None,
            t0: None,
            dt: None,
            sample_rate: None,
            times: None,
            _raw_t0_float: None, // For direct float t0 input
        }
    }

    // Builder setters for the TimeSeriesBaseBuilder
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
    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }
    /// Sets the GPS epoch time (t0) for the time series.
    pub fn epoch(mut self, epoch: Time) -> Self {
        self.t0 = Some(epoch);
        self
    }
    /// Sets the GPS epoch for these data as a raw `f64` GPS second value.
    pub fn t0(mut self, t0: f64) -> Self {
        self._raw_t0_float = Some(t0);
        self
    }
    /// Sets the time between successive samples (dt) as a `Quantity`
    pub fn dt(mut self, dt: Quantity) -> Self {
        self.dt = Some(dt);
        self
    }
    /// Sets the rate of samples per second (sample_rate) as a `Quantity`
    pub fn sample_rate(mut self, sample_rate: Quantity) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }
    /// Sets the complete array of GPS times accompanying the data as a `Quantity`.
    pub fn times(mut self, times: Quantity) -> Self {
        self.times = Some(times);
        self
    }
    /// Builds the `TimeSeriesBase` instance from the builder.
    /// This method translates the `TimesSeriesBase` specific arguments into the underlying `SeriesBuilder`'s `x0`, `dx`, and `xindex` fields.
    pub fn build(self) -> Result<TimeSeriesBase, QuantityError> {
        // Ensure we have the required value
        let value = self.value.ok_or_else(|| {
            QuantityError::InvalidQuantity("Value is required to build TimeSeriesBase".to_string())
        })?;
        let mut series_builder = SeriesBuilder::new().value(value).unit(
            self.unit
                .unwrap_or_else(|| Unit::new("", 1.0, UnitProduct::zero())),
        );
        if let Some(name) = self.name {
            series_builder = series_builder.name(name);
        }
        if let Some(channel) = self.channel {
            series_builder = series_builder.channel(channel);
        }
        if let Some(times_quantity) = self.times {
            // If times are provided, use them directly
            series_builder = series_builder.xindex(times_quantity);
        } else {
            // Handle `t0` or `epoch`
            let resolved_t0_quantity = if let Some(epoch_time) = self.t0 {
                // If epoch is provided, convert it to a Quantity
                Some(Quantity::new(
                    array![epoch_time.as_gps_seconds_f64()],
                    SECOND,
                ))
            } else {
                self._raw_t0_float
                    .map(|raw_t0| Quantity::new(array![raw_t0], SECOND))
            };
            if let Some(t0_quantity) = resolved_t0_quantity {
                series_builder = series_builder.x0(t0_quantity);
            }

            // Handle `dt` or `sample_rate`
            let resolved_dt_quantity = if let Some(dt_quantity) = self.dt {
                Some(dt_quantity) // Use provided dt directly
            } else if let Some(sample_rate_quantity) = self.sample_rate {
                // Convert sample_rate (Hz) to dt (seconds)
                // Ensure sample_rate is a scalar quantity
                if sample_rate_quantity.value.len() != 1 {
                    return Err(QuantityError::InvalidQuantity(
                        "Sample rate must be a scalar quantity.".to_string(),
                    ));
                }
                // (1.0 / sample_rate) should give us a Quantity in seconds
                let unit_s = SECOND;
                let one_quantity =
                    Quantity::new(array![1.0], Unit::new("", 1.0, UnitProduct::zero()));
                let dt_converted = (one_quantity / sample_rate_quantity)?.to(&unit_s)?;
                Some(dt_converted)
            } else {
                None
            };
            if let Some(dt_quantity) = resolved_dt_quantity {
                series_builder = series_builder.dx(dt_quantity);
            }
        }
        // Build the underlying Series
        let series_data = series_builder.build()?;
        Ok(TimeSeriesBase::new_internal(series_data))
    }
}

impl Default for TimeSeriesBaseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Private constructor for TimeSeriesBase
/// This constructor is used internally by the builder to create a TimeSeriesBase instance.
impl TimeSeriesBase {
    fn new_internal(series_data: Series) -> Self {
        TimeSeriesBase { series_data }
    }
    // Returns the underlying Series data

    pub fn value(&self) -> &Array1<f64> {
        self.series_data.value()
    }
    pub fn unit(&self) -> &Unit {
        self.series_data.unit()
    }
    pub fn get_name(&self) -> Option<&str> {
        self.series_data.get_name()
    }
    pub fn get_channel(&self) -> Option<&Channel> {
        self.series_data.get_channel()
    }
    pub fn get_t0(&self) -> Option<&Quantity> {
        self.series_data.get_x0()
    }
    pub fn get_dt(&self) -> Option<&Quantity> {
        self.series_data.get_dx()
    }
    pub fn get_times(&self) -> Option<&Quantity> {
        self.series_data.get_xindex()
    }
    pub fn get_epoch(&self) -> Option<Time> {
        self.get_t0()
            // t0_quantity should have a single value in seconds
            .map(|t0_quantity| Time::from_gps_seconds(t0_quantity.value[0]))
    }
    pub fn get_sample_rate(&self) -> Option<Quantity> {
        self.get_dt().map(|dt_quantity| {
            // dt_quantity should have a single value in seconds
            let one_quantity = Quantity::new(array![1.0], Unit::new("", 1.0, UnitProduct::zero()));
            (one_quantity / dt_quantity.clone())
                .expect("Failed to divide Quantity for sample rate.")
                .to(&HERTZ)
                .expect("Failed to convert Quantity to Hertz.")
        })
    }
    pub fn duration(&self) -> Option<Quantity> {
        self.get_times().map(|times_quantity| {
            let values = &times_quantity.value;
            if values.is_empty() {
                Quantity::new(array![0.0], times_quantity.unit.clone())
            } else {
                let start_time = values[0];
                let end_time = values[values.len() - 1];
                let duration_value = end_time - start_time;
                // Use the xunit of the times_quantity for the duration
                Quantity::new(array![duration_value], times_quantity.unit.clone())
            }
        })
    }
}

// -- Arithmetic operations for TimeSeriesBase

impl Add for TimeSeriesBase {
    type Output = Result<Self, QuantityError>;
    fn add(self, rhs: Self) -> Self::Output {
        // Delegate addition to Series's add method
        let result_series = self.series_data.add(rhs.series_data)?;
        Ok(TimeSeriesBase::new_internal(result_series))
    }
}

impl Div for TimeSeriesBase {
    type Output = Result<Self, QuantityError>;
    fn div(self, rhs: Self) -> Self::Output {
        // Delegate division to Series's div method
        let result_series = self.series_data.div(rhs.series_data)?;
        Ok(TimeSeriesBase::new_internal(result_series))
    }
}
impl Mul for TimeSeriesBase {
    type Output = Result<Self, QuantityError>;
    fn mul(self, rhs: Self) -> Self::Output {
        // Delegate multiplication to Series's mul method
        let result_series = self.series_data.mul(rhs.series_data)?;
        Ok(TimeSeriesBase::new_internal(result_series))
    }
}

impl Sub for TimeSeriesBase {
    type Output = Result<Self, QuantityError>;
    fn sub(self, rhs: Self) -> Self::Output {
        // Delegate subtraction to Series's sub method
        let result_series = self.series_data.sub(rhs.series_data)?;
        Ok(TimeSeriesBase::new_internal(result_series))
    }
}

// --- Test Module for TimeSeriesBase ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::channel::Channel;
    use astronomy::units::{HERTZ, JOULE, METRE, SECOND};
    use ndarray::array;

    #[test]
    fn test_timeseriesbase_creation_t0_dt() {
        let t0_time = Time::from_gps_seconds(1126259446.0);
        let dt_quantity = Quantity::new(array![0.000244140625], SECOND.clone());
        let channel = Channel::new("H1:GW-STRAIN", None, None, None, None, None, None).unwrap();

        let ts = TimeSeriesBaseBuilder::new()
            .value(array![1.0, 2.0, 3.0, 4.0])
            .unit(METRE.clone())
            .epoch(t0_time) // Using epoch
            .dt(dt_quantity.clone())
            .name("Strain Data".to_string())
            .channel(channel)
            .build()
            .unwrap();

        assert_eq!(ts.value(), &array![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(ts.unit(), &METRE);
        assert_eq!(ts.get_name(), Some("Strain Data"));
        assert_eq!(ts.get_channel().unwrap().get_name(), "H1:GW-STRAIN");

        // Check delegated/aliased properties
        assert_eq!(
            ts.get_t0().unwrap().value,
            &array![t0_time.as_gps_seconds_f64()]
        );
        assert_eq!(ts.get_t0().unwrap().unit, SECOND);
        assert_eq!(ts.get_dt().unwrap(), &dt_quantity);
        assert_eq!(
            ts.get_times().unwrap().value,
            &array![
                t0_time.as_gps_seconds_f64(),
                t0_time.as_gps_seconds_f64() + dt_quantity.value[0],
                t0_time.as_gps_seconds_f64() + 2.0 * dt_quantity.value[0],
                t0_time.as_gps_seconds_f64() + 3.0 * dt_quantity.value[0],
            ]
        );
        assert_eq!(ts.get_times().unwrap().unit, SECOND);

        // Check computed properties
        assert_eq!(ts.get_epoch().unwrap(), t0_time);
        assert_eq!(
            ts.get_sample_rate().unwrap().value[0],
            1.0 / dt_quantity.value[0]
        );
        assert_eq!(ts.get_sample_rate().unwrap().unit, HERTZ);

        println!("TimeSeriesBase (t0, dt): {:?}", ts);
    }

    #[test]
    fn test_timeseriesbase_creation_t0_sample_rate() {
        let raw_t0 = 123456789.0;
        let sr_quantity = Quantity::new(array![4096.0], HERTZ.clone()); // 4096 Hz

        let ts = TimeSeriesBaseBuilder::new()
            .value(array![1.0, 2.0, 3.0])
            .unit(JOULE.clone())
            .t0(raw_t0) // Using raw f64 t0
            .sample_rate(sr_quantity.clone())
            .name("Energy Reading".to_string())
            .build()
            .unwrap();

        assert_eq!(ts.get_t0().unwrap().value, &array![raw_t0]);
        assert_eq!(ts.get_t0().unwrap().unit, SECOND);
        assert_eq!(ts.get_sample_rate().unwrap(), sr_quantity); // Check the property
        assert_eq!(ts.get_dt().unwrap().value[0], 1.0 / 4096.0); // Check derived dt
        assert_eq!(ts.get_dt().unwrap().unit, SECOND);
        assert_eq!(ts.get_epoch().unwrap(), Time::from_gps_seconds(raw_t0));

        println!("TimeSeriesBase (t0, sample_rate): {:?}", ts);
    }

    #[test]
    fn test_timeseriesbase_creation_times() {
        let explicit_times = Quantity::new(array![100.0, 101.0, 102.0], SECOND.clone());

        let ts = TimeSeriesBaseBuilder::new()
            .value(array![10.0, 11.0, 12.0])
            .unit(METRE.clone())
            .times(explicit_times.clone())
            .name("Known Times".to_string())
            .build()
            .unwrap();

        assert_eq!(ts.get_times().unwrap(), &explicit_times);
        assert_eq!(ts.get_t0(), None); // x0/dt should be None if times is set
        assert_eq!(ts.get_dt(), None);
        assert_eq!(ts.get_epoch(), None); // epoch should be None if t0 is None
        assert!(ts.get_sample_rate().is_none());

        println!("TimeSeriesBase (times): {:?}", ts);
    }

    // #[test]
    // fn test_timeseriesbase_duration() {
    //     let t0_time = Time::from_gps_seconds(0.0);
    //     let dt_quantity = Quantity::new(array![1.0], SECOND.clone()); // 1 second sample interval
    //
    //     let ts = TimeSeriesBaseBuilder::new()
    //         .value(array![1.0, 2.0, 3.0, 4.0, 5.0]) // 5 samples
    //         .unit(METRE.clone())
    //         .epoch(t0_time)
    //         .dt(dt_quantity)
    //         .build()
    //         .unwrap();
    //
    //     // 5 samples means 4 intervals. Duration = 4 * dt
    //     assert_eq!(ts.duration().unwrap().value[0], 4.0);
    //     assert_eq!(ts.duration().unwrap().unit, SECOND);
    //
    //     // Empty series duration
    //     let empty_ts = TimeSeriesBaseBuilder::new()
    //         .value(array![])
    //         .unit(METRE.clone())
    //         .build()
    //         .unwrap();
    //     assert_eq!(empty_ts.duration().unwrap().value[0], 0.0);
    // }

    #[test]
    fn test_timeseriesbase_arithmetic_propagation() {
        let t0_time = Time::from_gps_seconds(100.0);
        let dt_quantity = Quantity::new(array![0.1], SECOND.clone());

        let ts1 = TimeSeriesBaseBuilder::new()
            .value(array![1.0, 2.0])
            .unit(METRE.clone())
            .epoch(t0_time)
            .dt(dt_quantity.clone())
            .name("TS1".to_string())
            .build()
            .unwrap();

        let ts2 = TimeSeriesBaseBuilder::new()
            .value(array![5.0, 6.0])
            .unit(METRE.clone())
            .epoch(Time::from_gps_seconds(100.0)) // Same epoch for simplicity
            .dt(dt_quantity.clone())
            .name("TS2".to_string())
            .build()
            .unwrap();

        let sum_ts = (ts1.clone() + ts2).unwrap();

        assert_eq!(sum_ts.value(), &array![6.0, 8.0]);
        assert_eq!(sum_ts.unit(), &METRE);
        assert_eq!(sum_ts.get_name(), ts1.get_name()); // Name from LHS
        assert_eq!(sum_ts.get_epoch(), ts1.get_epoch()); // Epoch from LHS
        assert_eq!(sum_ts.get_dt().unwrap(), ts1.get_dt().unwrap()); // dt from LHS
    }
}
