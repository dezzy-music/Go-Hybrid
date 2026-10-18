//go:build ignore

package main

import "hybrid.spike/w023"

// Negative test: Passing non-monotonic transition to step ascent must fail `go build`.
func main() {
	w3 := levels.LevelWitness[levels.L3]{}
	// Error: cannot step from L3 to L2; Ascent2to3 requires LevelWitness[levels.L2].
	_ = levels.StepAscent(w3, levels.Ascent2to3)
}
