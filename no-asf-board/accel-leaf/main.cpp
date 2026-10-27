// SPDX-License-Identifier: MIT
// Accelerometer leaf: CODAL's MMA8453 (the MMA8653 driver at 7-bit 0x1C),
// MPU6050 (0x68) and FXOS8700 (0x1E) drivers on the board's I2C bus (SERCOM2:
// SDA PA12, SCL PA13),
// linked from the same compiled sources as the full-board base with this main
// instead of the MakeCode runtime. MakeCode's accelerometer package selects
// only LIS3DH on this board (PXT_SUPPORT_* in the request's accelhw.cpp), so
// these drivers are run directly, as pxt would construct them. TC0 is the
// system timer the drivers' waits use. MMA8453 INT1 is on PB15 (active low),
// MPU6050 INT on PB14 (active high), FXOS8700 INT1 on PB13 (active low).
#include "ZPin.h"
#include "ZI2C.h"
#include "SAMDTCTimer.h"
#include "Timer.h"
#include "MMA8453.h"
#include "MPU6050.h"
#include "FXOS8700.h"

using namespace codal;

extern "C" {
// x, y, z in milli-g as each driver reports them.
volatile int32_t bw_accel_mma[3];
volatile int32_t bw_accel_mpu[3];
volatile int32_t bw_accel_fxos[3];
// MMA8453 and MPU6050 WHO_AM_I as their drivers report it; FXOS8700 detected.
volatile int32_t bw_accel_whoami[3];
volatile uint32_t bw_accel_done;
}

// The driver's own sample: getSample() polls INT and reads when it is active.
static void take(Accelerometer &acc, volatile int32_t *out)
{
    for (int i = 0; i < 20; i++) {
        Sample3D s = acc.getSample();
        out[0] = s.x;
        out[1] = s.y;
        out[2] = s.z;
        if (s.x || s.y || s.z)
            return;
        target_wait(5);
    }
}

int main()
{
    static SAMDTCTimer tc0(TC0, TC0_IRQn);
    static Timer timer(tc0);
    static ZPin sda(DEVICE_ID_IO_P0 + 0, PIN_PA12, PIN_CAPABILITY_DIGITAL);
    static ZPin scl(DEVICE_ID_IO_P0 + 1, PIN_PA13, PIN_CAPABILITY_DIGITAL);
    static ZPin mmaInt(DEVICE_ID_IO_P0 + 2, PIN_PB15, PIN_CAPABILITY_DIGITAL);
    static ZPin mpuInt(DEVICE_ID_IO_P0 + 3, PIN_PB14, PIN_CAPABILITY_DIGITAL);
    static ZPin fxosInt(DEVICE_ID_IO_P0 + 4, PIN_PB13, PIN_CAPABILITY_DIGITAL);
    static ZI2C i2c(sda, scl);
    // The board's NORTH_EAST_UP space leaves ENU samples unchanged.
    static CoordinateSpace space(NORTH_EAST_UP);

    static MMA8453 mma(i2c, mmaInt, space);
    bw_accel_whoami[0] = mma.whoAmI();
    take(mma, bw_accel_mma);

    static MPU6050 mpu(i2c, mpuInt, space);
    bw_accel_whoami[1] = mpu.whoAmI();
    take(mpu, bw_accel_mpu);

    bw_accel_whoami[2] = FXOS8700::isDetected(i2c, FXOS8700_DEFAULT_ADDR);
    static FXOS8700 fxos(i2c, fxosInt, space);
    take(fxos, bw_accel_fxos);

    bw_accel_done = 1;
    for (;;)
        __WFI();
}
