
// Pin assignments (all in arduino notation)
const uint8_t BIN1 = 2;
const uint8_t BIN2 = 3;
const uint8_t BPWM = 4;
const uint8_t AIN1 = 7;
const uint8_t AIN2 = 8;
const uint8_t APWM = 9;
const uint8_t STDBY = 6;

const uint8_t LED1 = 14;
const uint8_t LED2 = 12;

const uint8_t MOTOR_PWM_HI = 0x40;
const uint8_t MOTOR_PWM_LO = 0x0;

void setup() {
  analogWrite(LED1, 0);
  analogWrite(LED2, 0);
  digitalWrite(STDBY,LOW);
  pinMode(STDBY,OUTPUT);
  Serial.begin(9600);
}

// Command format:
// CXX\n where "C" is a one character command code and "XX" is a two
// character hex value.
// Commands include:
// l - left LED (0=off, 255=full on)
// r - right LED (0=off, 255=full on)
// - - stepper counterclockwise (value=# of steps)
// + - stepper clockwise (value=# of steps)
// Each command receieves a response of "OK\n" when complete.
// Unrecognized commands receive "ERR\n".

const char* ERR = "ERR\n";
const char* OK = "OK\n";

uint8_t fromHex(unsigned char c) {
  if (c <= '9' && c >= '0') return c-'0';
  if (c <= 'F' && c >= 'A') return 10+(c-'A');
  if (c <= 'f' && c >= 'a') return 10+(c-'a');
  return 0;
}

// Step pattern is represented as A2A1B1B2 in the low four bits 
const uint8_t STEP_PATTERN_LEN = 4;
uint8_t step_pattern[STEP_PATTERN_LEN] = { 0x5, 0x9, 0xa, 0x6 };
uint16_t curstep = 0;

void rotate(int16_t steps) {
  analogWrite(APWM, MOTOR_PWM_HI);
  analogWrite(BPWM, MOTOR_PWM_HI);
  digitalWrite(STDBY, HIGH);
  delay(2);
  while (steps != 0) {
    if (steps > 0) {
      curstep++; steps--;
    } else {
      curstep--; steps++;
    }
    uint8_t pat = step_pattern[curstep % STEP_PATTERN_LEN];
    digitalWrite(STDBY, LOW+20); // avoid braking during pin setting
    digitalWrite(AIN2, ((pat & 0x08)==0)?LOW:HIGH);
    digitalWrite(AIN1, ((pat & 0x04)==0)?LOW:HIGH);
    digitalWrite(BIN1, ((pat & 0x02)==0)?LOW:HIGH);
    digitalWrite(BIN2, ((pat & 0x01)==0)?LOW:HIGH);
    digitalWrite(STDBY, HIGH);
    delay(2);
  }
  analogWrite(APWM, MOTOR_PWM_LO);
  analogWrite(BPWM, MOTOR_PWM_LO);
}

void doCommand(unsigned char* cmd, uint8_t clen) {
  if (clen < 1) return;
  int16_t val = 0;
  for (uint8_t idx = 1; idx < 5 && idx < clen; idx++) {
    val = val << 4; val |= fromHex(cmd[idx]);
  }
  char command = cmd[0];
  switch (command) {
    case 'l':
    case 'L':
      analogWrite(LED1,val);
      break;   
    case 'r':
    case 'R':
      analogWrite(LED2,val);
      break;
    case '+':
      rotate(val);
      break;
    case '-':
      rotate(-val);
      break;
    case ' ':
    case '\n':
    case '\t':
    case '\r':
      return;
    default:
      Serial.write(ERR);
      return;
  }
  Serial.write(OK);
}

#define BUFSZ 20
unsigned char buf[BUFSZ];
uint8_t bidx = 0;

void loop() {
  if (Serial.available() > 0) {
    unsigned char c = Serial.read();
    if (c == '\r' || c == '\n') {
      doCommand(buf, bidx);
      bidx = 0;
    } else {
      if (bidx < BUFSZ) buf[bidx++] = c;
    }
  }
}
