// N-body: five bodies under gravity, symplectic advance (benchmarks-game shape).
package main

import (
	"fmt"
	"math"
	"os"
	"strconv"
)

const NB = 5
const PI = 3.141592653589793

// These are variables so the products below round in float64 at run time,
// like the C twin; Go folds constant expressions exactly.
var solarMass float64 = 4.0 * PI * PI
var days float64 = 365.24

// Fields per body: 0-2 position, 3-5 velocity, 6 mass.
var s [NB * 7]float64

func initBodies() {
	init := [NB * 7]float64{
		0.0, 0.0, 0.0, 0.0, 0.0, 0.0, solarMass,
		4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
		1.66007664274403694e-03 * days, 7.69901118419740425e-03 * days,
		-6.90460016972063023e-05 * days, 9.54791938424326609e-04 * solarMass,
		8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
		-2.76742510726862411e-03 * days, 4.99852801234917238e-03 * days,
		2.30417297573763929e-05 * days, 2.85885980666130812e-04 * solarMass,
		1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
		2.96460137564761618e-03 * days, 2.37847173959480950e-03 * days,
		-2.96589568540237556e-05 * days, 4.36624404335156298e-05 * solarMass,
		1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
		2.68067772490389322e-03 * days, 1.62824170038242295e-03 * days,
		-9.51592254519715870e-05 * days, 5.15138902046611451e-05 * solarMass,
	}
	for i := 0; i < NB*7; i++ {
		s[i] = init[i]
	}
}

func offsetMomentum() {
	px, py, pz := 0.0, 0.0, 0.0
	for b := 0; b < NB; b++ {
		m := s[b*7+6]
		px += s[b*7+3] * m
		py += s[b*7+4] * m
		pz += s[b*7+5] * m
	}
	s[3] = -px / solarMass
	s[4] = -py / solarMass
	s[5] = -pz / solarMass
}

func energy() float64 {
	e := 0.0
	for i := 0; i < NB; i++ {
		bi := i * 7
		vx, vy, vz := s[bi+3], s[bi+4], s[bi+5]
		e += 0.5 * s[bi+6] * (vx*vx + vy*vy + vz*vz)
		for j := i + 1; j < NB; j++ {
			bj := j * 7
			dx := s[bi] - s[bj]
			dy := s[bi+1] - s[bj+1]
			dz := s[bi+2] - s[bj+2]
			d := math.Sqrt(dx*dx + dy*dy + dz*dz)
			e -= s[bi+6] * s[bj+6] / d
		}
	}
	return e
}

func advance(dt float64) {
	for i := 0; i < NB; i++ {
		bi := i * 7
		for j := i + 1; j < NB; j++ {
			bj := j * 7
			dx := s[bi] - s[bj]
			dy := s[bi+1] - s[bj+1]
			dz := s[bi+2] - s[bj+2]
			d2 := dx*dx + dy*dy + dz*dz
			mag := dt / (d2 * math.Sqrt(d2))
			mi := s[bi+6] * mag
			mj := s[bj+6] * mag
			s[bi+3] -= dx * mj
			s[bi+4] -= dy * mj
			s[bi+5] -= dz * mj
			s[bj+3] += dx * mi
			s[bj+4] += dy * mi
			s[bj+5] += dz * mi
		}
	}
	for b := 0; b < NB; b++ {
		bb := b * 7
		s[bb] += dt * s[bb+3]
		s[bb+1] += dt * s[bb+4]
		s[bb+2] += dt * s[bb+5]
	}
}

func main() {
	var steps int64 = 20000
	if len(os.Args) > 1 {
		steps, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	initBodies()
	offsetMomentum()
	fmt.Printf("%.9f\n", energy())
	for k := int64(0); k < steps; k++ {
		advance(0.01)
	}
	fmt.Printf("%.9f\n", energy())
}
