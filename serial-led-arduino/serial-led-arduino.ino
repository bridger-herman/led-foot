#define RED 10
#define GREEN 9
#define BLUE 6
#define WHITE 5
#define MAX_VALUE 255

#define NUMPINS 4
#define BUFSIZE 6 // 2 bytes for RED and GREEN each, one byte for BLUE and WHITE

const int PINS[] = {RED, GREEN, BLUE, WHITE};
unsigned char buf[BUFSIZE];
int bytesRead = 0;

// 16 bit PWM: https://arduino.stackexchange.com/a/12719
// Red and Green are chosen to be the 16 bit outputs because of luminance:
// L = 0.21*R + 0.72*G + 0.072*B
/* Configure digital pins 9 and 10 as 16-bit PWM outputs. */
void setupPWM16() {
    DDRB |= _BV(PB1) | _BV(PB2);        /* set pins as outputs */
    TCCR1A = _BV(COM1A1) | _BV(COM1B1)  /* non-inverting PWM */
        | _BV(WGM11);                   /* mode 14: fast PWM, TOP=ICR1 */
    TCCR1B = _BV(WGM13) | _BV(WGM12)
        | _BV(CS10);                    /* no prescaling */
    ICR1 = 0xffff;                      /* TOP counter value */
}

/* 16-bit version of analogWrite(). Works only on pins 9 and 10. */
void analogWrite16(uint8_t pin, uint16_t val)
{
    switch (pin) {
        case  9: OCR1A = val; break;
        case 10: OCR1B = val; break;
    }
}


void setup() {
  Serial.begin(9600);

  for (int i = 0; i < NUMPINS; i++) {
    pinMode(PINS[i], OUTPUT);
  }
  memset(buf, BUFSIZE*sizeof(unsigned char), 0);

  setupPWM16();

  Serial.println("I"); // Successfully initialized
}

void setRGBW(int value) {
  analogWrite16(RED, value);
  analogWrite16(GREEN, value);
  analogWrite(BLUE, value);
  analogWrite(WHITE, value);
}

void setRGBW(int r, int g, int b, int w) {
  analogWrite16(RED, r);
  analogWrite16(GREEN, g);
  analogWrite(BLUE, b);
  analogWrite(WHITE, w);
}

void panic() {
  for (int i = 0; i < 1000; i += 100) {
    setRGBW(i%MAX_VALUE, 0, 0, 0);
    delay(10);
  }
}

void printBuffer(unsigned char* buf) {
  String cur;
  cur.concat(buf[1]);
  cur.concat(' ');
  cur.concat(buf[2]);
  cur.concat(' ');
  cur.concat(buf[3]);
  cur.concat(' ');
  cur.concat(buf[4]);
  Serial.println(cur);
}

void loop() {
  if (Serial.available() >= BUFSIZE*sizeof(unsigned char)) {
    bytesRead = Serial.readBytes(buf, BUFSIZE);
    if (bytesRead == BUFSIZE) {
      // Convert from bytes to shorts
      int redValue = ((int) buf[0] << 8) | (int) buf[1];
      int greenValue = ((int) buf[2] << 8) | (int) buf[3];
      
      setRGBW(redValue, greenValue, buf[4], buf[5]);
      bytesRead = 0;      
      Serial.println("C"); // Successfully changed
      memset(buf, BUFSIZE*sizeof(unsigned char), 0);
    }
  }
}
