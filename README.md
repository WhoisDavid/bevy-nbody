# bevy-nbody

Experiments with a simple n-body simulation in 3D with [Bevy](https://bevyengine.org/).

In particular, pretty cool ephemerides data from JPL Horizons allowing you to get initial conditions to simulate the Solar System:

![](assets/solar-system.gif)

```
Usage: nbody [--startup <startup>] [--speed <speed>] [-d]

N-body 3D simulation with Bevy

Several startup options:
- planets of the Solar System - and Pluto :'( - with data from JPL Horizons as-of 2021-04-18.
- figure-8 stable three-body solution
- random bodies

Options:
  --startup         startup system [solar (default)|figure8|random]
  --speed           speed of the simulation [default: 1.0x]
  -d, --debug       enable diagnostics
  --help            display usage information
```
