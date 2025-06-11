use astronomy::units::{Dimension, Quantity, QuantityError, Unit, UnitProduct};
use ndarray::array;
use thiserror::Error;

/// Errors that can occur when creating or manipulating a `Channel`.
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("Failed to parse unit string: {0}")]
    UnitParseError(String),
    #[error("Invalid quantity error: {0}")]
    QuantityError(#[from] QuantityError), //Allows converting QuantityError into ChannelError
}

#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    // The data for this channel, e.g., gravitational wave strain data

    // The name of the channel, e.g., "H1:GWOSC-4KHZ_RAMP_C00"
    // typically follows the format "Detector:ChannelName"
    pub name: String,

    // In Hz, or with explicit Unit
    pub sample_rate: Option<Quantity>,

    // The unit of the data for this channel, e.g., "m", "s", "Hz"
    pub unit: Option<Unit>,

    // Frequency range in Hz (low, high), e.g. (0.0, 1000.0)
    pub frequency_range: Option<(f64, f64)>,

    // Indicates if the channel is safe to use
    pub safe: Option<bool>,

    // LDAS name for frames that contain this channel.
    // The type of frame this channel belongs to, e.g., "L1_HOFT_C00", "H1_HOFT_C00"
    pub frametype: Option<String>,

    // The model of the detector, e.g., "LIGO", "Virgo"
    pub model: Option<String>,
}
impl Channel {
    /// Creates a new Channel with the given name, sample rate, unit, frequency range, safety status, frame type, and model.
    /// # Parameters
    /// - `name`: The name of the channel, e.g., "H1:GWOSC-4KHZ_RAMP_C00".
    /// - `sample_rate`: The sample rate of the channel, e.g., 4096 Hz. If provided, converted to a `Quantity` with "Hz" unit.
    /// - `unit`: The unit of the data for this channel, e.g., "m", "s", "Hz". For now, an astronomy::units::Unit must be provided.
    /// - `frequency_range`: The frequency range of the channel, e.g., (0.0, 1000.0) Hz.
    /// - `safe`: Indicates if the channel is safe to use, e.g., `true` or `false`.
    /// - `frametype`: The LDAS name for frames that contain this channel, e.g., "L1_HOFT_C00".
    /// - `model`: The model of the detector, e.g., "LIGO", "Virgo".
    /// # Returns
    /// A `Result` containing the new `Channel` instance or an error if the unit is not valid.
    pub fn new(
        name: impl Into<String>, // Allows accepting String or &str
        sample_rate: Option<f64>,
        unit: Option<Unit>,
        frequency_range: Option<(f64, f64)>,
        safe: Option<bool>,
        frametype: Option<String>,
        model: Option<String>,
    ) -> Result<Self, ChannelError> {
        let parsed_sample_rate = sample_rate.map(|sr| {
            Quantity::new(
                array![sr],
                // Hz is the inverse of time
                // We create a basic Unit for Hz using Dimension::Time.inverse()
                Unit::new("Hz", 1.0, UnitProduct::new(Dimension::Time).inverse()),
            )
        });

        Ok(Self {
            name: name.into(),
            sample_rate: parsed_sample_rate,
            unit,
            frequency_range,
            safe,
            frametype,
            model,
        })
    }

    /// Returns the name of the channel.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the sample rate of the channel, if available, as an Option of Quantity.
    pub fn get_sample_rate(&self) -> Option<&Quantity> {
        self.sample_rate.as_ref()
    }

    /// Returns the unit of the channel, if available, as an Option of Unit.
    pub fn get_unit(&self) -> Option<&Unit> {
        self.unit.as_ref()
    }

    /// Returns the frequency range of the channel, if available, as an Option of tuple (low, high).
    pub fn get_frequency_range(&self) -> Option<(f64, f64)> {
        self.frequency_range
    }

    /// Return the safety status of the channel, if available, as an Option of bool.
    pub fn is_safe(&self) -> Option<bool> {
        self.safe
    }

    /// Returns the frame type of the channel, if available, as an Option of &str.
    pub fn get_frametype(&self) -> Option<&str> {
        self.frametype.as_deref()
    }

    /// Returns the model of the detector, if available, as an Option of &str.
    pub fn get_model(&self) -> Option<&str> {
        self.model.as_deref()
    }
}

// -- Display implementation for Channel
impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Channel(name='{}'", self.name)?;
        if let Some(sr) = &self.sample_rate {
            // Display value and unit of Quantity
            write!(f, ", sample_rate={}{}", sr.value[0], sr.unit.name)?;
        }
        if let Some(u) = &self.unit {
            write!(f, ", unit={}", u.name)?;
        }
        if let Some((low, high)) = self.frequency_range {
            write!(f, ", frequency_range=({}, {})", low, high)?;
        }
        if let Some(safe) = self.safe {
            write!(f, ", safe={}", safe)?;
        }
        if let Some(ft) = &self.frametype {
            write!(f, ", frametype='{}'", ft)?;
        }
        if let Some(model) = &self.model {
            write!(f, ", model='{}'", model)?;
        }
        write!(f, ")")
    }
}

// -- Tests for Channel
#[cfg(test)]
mod channel_tests {
    use super::*;
    use astronomy::units::{METER, Unit};

    #[test]
    fn test_channel_creation_minimal() {
        let channel = Channel::new(
            "H1:GWOSC-4KHZ_RAMP_C00",
            Some(4096.0),
            Some(Unit::new("m", 1.0, UnitProduct::new(Dimension::Length))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(channel.get_name(), "H1:GWOSC-4KHZ_RAMP_C00");
        assert_eq!(channel.get_sample_rate().unwrap().value[0], 4096.0);
        assert_eq!(channel.get_unit().unwrap().name, "m");
        assert!(channel.get_frequency_range().is_none());
        assert!(channel.is_safe().is_none());
        assert!(channel.get_frametype().is_none());
        assert!(channel.get_model().is_none());
        println! {"Channel created: {channel}"};
    }

    #[test]
    fn test_channel_creation() {
        let channel = Channel::new(
            "H1:GWOSC-4KHZ_RAMP_C00",
            Some(4096.0),
            Some(METER),
            Some((0.0, 1000.0)),
            Some(true),
            Some("L1_HOFT_C00".to_string()),
            Some("LIGO".to_string()),
        )
        .unwrap();

        assert_eq!(channel.get_name(), "H1:GWOSC-4KHZ_RAMP_C00");
        assert_eq!(channel.get_sample_rate().unwrap().value[0], 4096.0);
        assert_eq!(channel.get_unit().unwrap().name, "m");
        assert_eq!(channel.get_frequency_range(), Some((0.0, 1000.0)));
        assert_eq!(channel.is_safe(), Some(true));
        assert_eq!(channel.get_frametype(), Some("L1_HOFT_C00"));
        assert_eq!(channel.get_model(), Some("LIGO"));
    }

    #[test]
    fn test_channel_creation_with_specific_unit_and_rate() {
        let rate_unit = Unit::new("Hz", 1.0, UnitProduct::new(Dimension::Time).inverse());
        let expected_rate = Quantity::new(array![1024.0], rate_unit);

        // Volts ML2T−3I−1
        let voltage_unit = Unit::new(
            "V",
            1.0,
            UnitProduct::from_components(&[
                (Dimension::Mass, 1),
                (Dimension::Length, 2),
                (Dimension::Time, -3),
                (Dimension::ElectricCurrent, -1),
            ]),
        );
        let channel = Channel::new(
            "MyChannel",
            Some(1024.0),
            Some(voltage_unit.clone()),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(channel.get_name(), "MyChannel");
        assert_eq!(
            channel.get_sample_rate().unwrap().value,
            expected_rate.value
        );
        assert_eq!(channel.get_sample_rate().unwrap().unit, expected_rate.unit);
        assert_eq!(channel.get_unit().unwrap(), &voltage_unit);
    }

    #[test]
    fn test_channel_display() {
        let channel = Channel::new(
            "H1:GWOSC-4KHZ_RAMP_C00",
            Some(4096.0),
            Some(Unit::new("m", 1.0, UnitProduct::new(Dimension::Length))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            format!("{}", channel),
            "Channel(name='H1:GWOSC-4KHZ_RAMP_C00', sample_rate=4096Hz, unit=m)"
        );
    }
}
