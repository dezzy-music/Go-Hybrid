//go:build ignore

package main

import "hybrid.spike/w023"

// Negative test: Attempting to ascend backwards from L2 to L1 must fail `go build`.
func main() {
	w2 := levels.LevelWitness[levels.L2]{}
	// Error: cannot use levels.Ascending[levels.L2, levels.L1]{} as levels.Ascending[levels.L1, levels.L2]
	// Ascent1to2 requires LevelWitness[levels.L1], but caller holds LevelWitness[levels.L2].
	_ = levels.StepAscent(w2, levels.Ascent1to2)
}
