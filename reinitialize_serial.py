# Sometimes the Raspberry Pi serial connection to the Arduino gets stuck. This
# script "flushes" the output.
#
# Needs the `pyserial` package.

import serial
s = serial.Serial('/dev/ttyACM0')
s.write([0] * 9) # magic number of bytes for Arduino
s.read()