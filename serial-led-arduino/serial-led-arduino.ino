// NOTE: probably only works on Arduino Mega 2560.

// Define pins that colors are plugged into
#define RED 12
#define GREEN 11
#define BLUE 6
#define WHITE 5
#define MAX_VALUE 255

// Define pins that relays are plugged into
#define LIVING_ROOM 28
#define OFFICE 26
#define BEDROOM 24

#define NUMPINS 4

// 1 magic byte for determining whether a color command is being sent, or a room relay command
// Then, either:
//   Color command: 8 more bytes, 2 bytes for each color channel
//       (they're shorts, represented in Little-Endian format)
//   Room relay command: up to 3 bytes plus up to 8 bytes of zero-padding
//       Contains an array of the currently active rooms
//       (0x1C, 0x1A, 0x18 for LIVING_ROOM, OFFICE, and BEDROOM respectively)
#define BUFSIZE 9

// Magic numbers for color or room relay commands
#define COLOR_CMD 0xC0
#define ROOM_CMD 0xF0

const int PINS[] = {RED, GREEN, BLUE, WHITE};
unsigned char buf[BUFSIZE];
int bytesRead = 0;

// 16 bit PWM: https://arduino.stackexchange.com/a/12719
// With help from https://arduino.stackexchange.com/questions/4877/16-bit-pwm-on-a-mega
// Using reference diagram https://www.arduino.cc/en/uploads/Hacking/PinMap2560big.png
// I could only get these to work: PE3, PH3, PB5, PB6
// I think it has something to do with me not setting the timers up properly.
void setupPWM16() {  
    // Setup outputs (digital pins 5, 6, 11, and 12 on Mega 2560, for W, B, G, and R)
    DDRB |= _BV(PE3) | _BV(PH3) | _BV(PB5) | _BV(PB6);

    // Set up timer 1 for 16 bit PWM (for PB5 and PB6)
    TCCR1A = _BV(COM1A1) | _BV(COM1B1)  /* non-inverting PWM */
        | _BV(WGM11);                   /* mode 14: fast PWM, TOP=ICR1 */
    TCCR1B = _BV(WGM13) | _BV(WGM12)
        | _BV(CS10);                    /* no prescaling */
    ICR1 = 0xffff;                      /* TOP counter value */

    // Set up timer 3 for 16 bit PWM (for PE3)
    TCCR3A = _BV(COM1A1) | _BV(COM1B1)  /* non-inverting PWM */
        | _BV(WGM11);                   /* mode 14: fast PWM, TOP=ICR3 */
    TCCR3B = _BV(WGM13) | _BV(WGM12)
        | _BV(CS10);                    /* no prescaling */
    ICR3 = 0xffff;                      /* TOP counter value */

    // Set up timer 4 for 16 bit PWM (for PH3)
    TCCR4A = _BV(COM1A1) | _BV(COM1B1)  /* non-inverting PWM */
        | _BV(WGM11);                   /* mode 14: fast PWM, TOP=ICR4 */
    TCCR4B = _BV(WGM13) | _BV(WGM12)
        | _BV(CS10);                    /* no prescaling */
    ICR4 = 0xffff;                      /* TOP counter value */

    pinMode(LED_BUILTIN, OUTPUT);
}

void blinkAlert(int howMany) {
  for (int i = 0; i < howMany; i++) {
    digitalWrite(LED_BUILTIN, HIGH);
    delay(50);
    digitalWrite(LED_BUILTIN, LOW);
    delay(50);
  }
  delay(200);
}

// 16-bit version of analogWrite()
// Specific mappings based on Mega 2560 data sheet
void analogWrite16(uint8_t pin, uint16_t val)
{
    switch (pin) {
        case 5: OCR3A = val; break;
        case 6: OCR4A = val; break;
        case 11: OCR1A = val; break;
        case 12: OCR1B = val; break;
    }
}

void setRGBW(int value) {
  analogWrite16(RED, value);
  analogWrite16(GREEN, value);
  analogWrite16(BLUE, value);
  analogWrite16(WHITE, value);
}

void setRGBW(int r, int g, int b, int w) {
  analogWrite16(RED, r);
  analogWrite16(GREEN, g);
  analogWrite16(BLUE, b);
  analogWrite16(WHITE, w);
}

void allRooms(int state) {
  digitalWrite(LIVING_ROOM, state);
  digitalWrite(OFFICE, state);
  digitalWrite(BEDROOM, state);

}

void setup() {
  Serial.begin(9600);

  // Set the LEDs to be output pins
  for (int i = 0; i < NUMPINS; i++) {
    pinMode(PINS[i], OUTPUT);
  }
  memset(buf, BUFSIZE*sizeof(unsigned char), 0);

  setupPWM16();

  // Set output pins for relays
  pinMode(LIVING_ROOM, OUTPUT);
  pinMode(OFFICE, OUTPUT);
  pinMode(BEDROOM, OUTPUT);

  allRooms(LOW);

  Serial.println("I"); // Successfully initialized
}

void colorCmd(unsigned char buf[BUFSIZE]) {
  // Convert from bytes to shorts
  int redValue = ((int) buf[1] << 8) | (int) buf[2];
  int greenValue = ((int) buf[3] << 8) | (int) buf[4];
  int blueValue = ((int) buf[5] << 8) | (int) buf[6];
  int whiteValue = ((int) buf[7] << 8) | (int) buf[8];
  
  // If it's completely black, turn off the relays, independently of what their
  // state is from the room commands
  if (redValue == 0 && greenValue == 0 && blueValue == 0 && whiteValue == 0) {
    allRooms(LOW);
  } else {
    allRooms(HIGH);
  }
  
  setRGBW(redValue, greenValue, blueValue, whiteValue);
}

void loop() {
  if (Serial.available() >= BUFSIZE*sizeof(unsigned char)) {
    bytesRead = Serial.readBytes(buf, BUFSIZE);
    if (bytesRead == BUFSIZE) {
      if (buf[0] == ROOM_CMD) {
        Serial.println("R"); // Successfully changed room status
      } else if (buf[0] = COLOR_CMD) {
        colorCmd(buf);
        Serial.println("C"); // Successfully changed color
      } else {
        Serial.println("X"); // Invalid command
      }

      bytesRead = 0;
      memset(buf, BUFSIZE*sizeof(unsigned char), 0);
    }
  }
}
