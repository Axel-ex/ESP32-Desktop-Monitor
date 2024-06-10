# Rust Desktop monitor
Small clock, temperature / humidity / wifi signal monitor using esp_idf_svc framework to program ESP32 with Rust!

# Prerequisites
### Software

To build this project, you will need a few resources installed on your machine. A clear walktrough of the dependencies to install can be found here: https://github.com/esp-rs/esp-idf-template.
Once install you should be able to run 
```shell
cargo generate esp-rs/esp-idf-template cargo
```
This command will prompt you for some informations regarding the architecture of the ESP32 you are using and will set project configuration accordingly (toolchain, dependencies...). If everything is configurated like it should, connect your board to the serial port and simply run:

```shell
cargo run
```

### Hardware

For this project i am using:
- ESP32 DevkitV1 (ESP32 in esp-idf-template).
- DHT22 Temperature / Humidity Sensor
- 128 * 64 OLED SH1106 OLED display.

OLED display: vcc(3v3), gnd(gnd), sda(pin21), sck(pin22)

DHT22: vcc(vcc), gnd(gnd), data(pin27)


# Code explanation
### Wifi.rs

#### `wifi_init()`
Initializes the WiFi capabilities of the ESP32 by setting up necessary components such as the WiFi module, SystemEventLoop, and Non-Volatile Storage (NVS).
- **modem:** The hardware component responsible for WiFi communication.
- **SystemEventLoop:** Central to the ESP-IDF framework, this component is responsible for handling and polling system events asynchronously. It manages WiFi events, such as connection and disconnection notifications, which allows the system to respond to changes in WiFi status.
- **Non-Volatile Storage (NVS):** Used for storing persistent data, such as WiFi credentials. This storage ensures that the ESP32 can reconnect to the network automatically upon reboot without requiring user intervention.

#### `wifi_connect()`
Configures and connects the ESP32 to a WiFi network using predefined SSID and password.
- **Configuration:** Uses WIFI_SSID and WIFI_PASSWORD to set up the WiFi connection parameters.
- **Network Interface:** Waits for the network interface to be up before proceeding with further operations. This ensures that the device is fully connected to the network and ready for communication.

### lib.rs

#### `DeviceState`
Initializes the device state, sets up peripherals, and provides functions for display and interaction.
  - **OLED Display:** Used to visualize time, temperature, and WiFi signal strength.
  - **DHT22 Sensor:** Reads temperature and humidity data.
  - **LED:** Provides visual indication.
  - **WiFi Module:** Connects to WiFi networks and scans for signal strength.
  - **SNTP Client:** Synchronizes time with an NTP server.

#### `DeviceState::init_display()`
Initializes the OLED display. the process first involves creating an I2c configuration out of which we can create the I2cDriver needed for the GraphicsMode building process.
- **i2c0**: The I2C interface connected to the OLED display.
- **sda**: The pin for the I2C serial data line.
- **scl**: The pin for the I2C serial clock line.

#### `DeviceState::sync_sntp()`
Synchronizes the device's internal clock with an SNTP (Simple Network Time Protocol) server.

#### `DeviceState::display_wifi_info()`
Scans for nearby WiFi networks, updates the WiFi signal tracker, and displays signal strength.
- **WiFi Module:** Scans for available networks and retrieves signal strength.
- **WiFi Signal Tracker:** Tracks WiFi signal strength over time.

# Troubleshooting
Since I decided to encapsulate all initialisation logic within the new() method of the device_state object, the default stack size was not sufficient for the heavy load of operations new() was executing. I decided to increase the stack size in sdkconfig.defaults and increase the timeout value of the watchdog that was causing the init logic to fail due to the heavy computation. Another approach could be to set the watchdog at runtime and feed it regularly within the `DeviceState::new()`to avoid timeouts (see https://docs.esp-rs.org/esp-idf-svc/esp_idf_svc/hal/task/watchdog/index.html)

# Resources
- The Rust Programming language (**THE** book): https://doc.rust-lang.org/stable/book/
- The Rust on ESP Book: https://docs.esp-rs.org/book/overview/using-the-standard-library.html
- The Embedded Rust Book: https://docs.rust-embedded.org/book/
- esp-idf-svc API reference (Rust): https://docs.esp-rs.org/esp-idf-svc/esp_idf_svc/
- esp_idf API reference (C): https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/index.html
- youtube channels: 
	- with std: https://www.youtube.com/@floodplainnl
Y,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
	- No std: https://www.youtube.com/@therustybits, https://www.youtube.com/@LowLevelLearning

# Contributing
How to?
    Fork the repository.
    Make your changes.
    Create a pull request.
That's it! Your contributions are welcome and appreciated.

# Licence
By contributing to this project, you agree that your contributions will be licensed under the LICENSE.
