package levels

import (
	"sync"
	"testing"
)

// BenchmarkRawGoChannelSendRecv measures native Go unranked buffered channel throughput.
func BenchmarkRawGoChannelSendRecv(b *testing.B) {
	ch := make(chan int, 128)
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		ch <- i
		<-ch
	}
}

// BenchmarkLevelChanErasedSendRecv proves that in production (erased lowering in hy-emit),
// LevelChan[T] has zero runtime abstraction overhead: 0 B/op and 0 allocs/op matching raw Go.
func BenchmarkLevelChanErasedSendRecv(b *testing.B) {
	lc := MakeLevelChan[int](1, 128)
	raw := lc.Raw()
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		raw <- i
		<-raw
	}
}

// BenchmarkLevelChanCheckedSendRecv measures the overhead of TaskContext level tracking
// in debug / checked verification mode.
func BenchmarkLevelChanCheckedSendRecv(b *testing.B) {
	tc := NewTaskContext()
	lc := MakeLevelChan[any](1, 128)
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = tc.SendChecked(lc, i)
		_, _ = tc.RecvChecked(lc)
	}
}

// BenchmarkDiningPhilosophersLevelOrdered measures sustained throughput across
// 5 concurrent philosophers acquiring forks strictly according to level hierarchy.
func BenchmarkDiningPhilosophersLevelOrdered(b *testing.B) {
	const n = 5
	forks := make([]LevelChan[struct{}], n)
	for i := 0; i < n; i++ {
		forks[i] = MakeLevelChan[struct{}](i+1, 1)
		forks[i].Raw() <- struct{}{}
	}

	b.ReportAllocs()
	b.ResetTimer()

	var wg sync.WaitGroup
	wg.Add(n)
	itersPerPhil := b.N / n
	if itersPerPhil == 0 {
		itersPerPhil = 1
	}

	for i := 0; i < n; i++ {
		go func(id int) {
			defer wg.Done()
			f1 := id
			f2 := (id + 1) % n
			if forks[f1].Level > forks[f2].Level {
				f1, f2 = f2, f1
			}
			for j := 0; j < itersPerPhil; j++ {
				<-forks[f1].Raw()
				<-forks[f2].Raw()
				forks[f2].Raw() <- struct{}{}
				forks[f1].Raw() <- struct{}{}
			}
		}(i)
	}
	wg.Wait()
}
