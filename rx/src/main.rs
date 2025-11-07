use ::log::info;
use esp_idf_svc::hal::units::Hertz;

fn main() -> anyhow::Result<()> {
    use esp_idf_svc::hal::adc::{AdcContConfig, AdcContDriver, AdcMeasurement, Attenuated};
    use esp_idf_svc::hal::peripherals::Peripherals;

    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // let config = AdcContConfig::default().sample_freq(Hertz::from(80000));
    let config = AdcContConfig::default().sample_freq(Hertz::from(1000));

    let adc_1_channel_0 = Attenuated::db11(peripherals.pins.gpio2);
    let mut adc = AdcContDriver::new(peripherals.adc1, &config, adc_1_channel_0)?;

    adc.start()?;

    //Default to just read 100 measurements per each read
    let mut samples = [AdcMeasurement::default(); 100];

    loop {
        if let Ok(num_read) = adc.read(&mut samples, 10) {
            info!("Read {} measurement.", num_read);
            let len = samples.len();
            let sum: u64 = samples.map(|e| e.data() as u64).into_iter().sum();
            let avg = sum as f64 / len as f64;
            info!("avg: {avg}");
            // for index in 0..num_read {
            //     info!("{}", samples[index].data());
            // }
        }
    }
}
