
# Some trouble shooting
Since I decided to encapsulate all initialisation logic within the new() method of the device_state object, the default stack size was not sufficient for the heavy load of operations new() was executing. I decided to increase the stack size in sdkconfig.defaults and deactivate the watchdog that was causing the init_display to fail.
