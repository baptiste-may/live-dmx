use serialport::{available_ports, SerialPortType};

use super::types::PhysicalInterface;

pub fn default_physical_interfaces() -> Vec<PhysicalInterface> {
    #[cfg(target_os = "windows")]
    let ports = ["COM3", "COM4", "COM5"];
    #[cfg(target_os = "macos")]
    let ports = [
        "/dev/cu.usbserial-EN0001",
        "/dev/cu.usbserial-FT0002",
        "/dev/cu.usbmodem101",
    ];
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let ports = ["/dev/ttyUSB0", "/dev/ttyUSB1", "/dev/ttyUSB2"];

    vec![
        PhysicalInterface {
            id: "physical-1".to_string(),
            name: "Physical 1".to_string(),
            device: "Enttec DMX USB Pro".to_string(),
            port: ports[0].to_string(),
            status: "connected".to_string(),
            fps: 44,
        },
        PhysicalInterface {
            id: "physical-2".to_string(),
            name: "Physical 2".to_string(),
            device: "FTDI Open DMX".to_string(),
            port: ports[1].to_string(),
            status: "connected".to_string(),
            fps: 40,
        },
        PhysicalInterface {
            id: "physical-3".to_string(),
            name: "Physical 3".to_string(),
            device: "DMXking ultraDMX MAX".to_string(),
            port: ports[2].to_string(),
            status: "connected".to_string(),
            fps: 44,
        },
    ]
}

pub fn discover_physical_interfaces() -> Result<Vec<PhysicalInterface>, String> {
    let ports = available_ports().map_err(|e| e.to_string())?;
    let mut detected = Vec::new();

    for p in ports {
        let port_lower = p.port_name.to_lowercase();

        // Skip Bluetooth serial devices across all platforms
        if matches!(p.port_type, SerialPortType::BluetoothPort) || port_lower.contains("bluetooth")
        {
            continue;
        }

        // On macOS, prefer outgoing callout devices (/dev/cu.*) over dial-in (/dev/tty.*)
        // because opening /dev/tty.* blocks waiting for carrier detect (DCD).
        #[cfg(target_os = "macos")]
        if port_lower.starts_with("/dev/tty.") {
            continue;
        }

        let (device_name, is_target_serial) = match &p.port_type {
            SerialPortType::UsbPort(usb) => {
                let product = usb.product.clone().or_else(|| usb.manufacturer.clone());
                (
                    product.unwrap_or_else(|| format!("USB Serial ({})", p.port_name)),
                    true,
                )
            }
            SerialPortType::PciPort => ("PCI Serial Port".to_string(), false),
            SerialPortType::BluetoothPort => ("Bluetooth Serial".to_string(), false),
            SerialPortType::Unknown => (format!("Serial Port ({})", p.port_name), false),
        };

        // Match USB serial devices across Linux (/dev/ttyUSB*, /dev/ttyACM*),
        // macOS (/dev/cu.usbserial*, /dev/cu.usbmodem*), and Windows (COM1..n).
        if is_target_serial
            || port_lower.contains("usb")
            || port_lower.contains("acm")
            || port_lower.contains("cu.")
            || port_lower.starts_with("com")
            || port_lower.starts_with(r"\\.\com")
        {
            let next_idx = detected.len() + 1;
            detected.push(PhysicalInterface {
                id: format!("physical-{}", next_idx),
                name: format!("Physical {}", next_idx),
                device: device_name,
                port: p.port_name,
                status: "connected".to_string(),
                fps: 44,
            });
        }
    }

    Ok(detected)
}
