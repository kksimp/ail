// Fan-out, fan-in: four workers take jobs from one channel and send results
// to another. A WaitGroup closes the results channel when every worker is
// done, so `main` reads results until it closes.
package main

import (
	"fmt"
	"sort"
	"sync"
)

type Done struct {
	job, worker, steps int
}

func steps(n int) int {
	count := 0
	for n != 1 {
		if n%2 == 0 {
			n /= 2
		} else {
			n = 3*n + 1
		}
		count++
	}
	return count
}

func feed(jobs chan<- int, n int) {
	for i := 1; i <= n; i++ {
		jobs <- i
	}
	close(jobs)
}

func work(id int, jobs <-chan int, results chan<- Done, wg *sync.WaitGroup) {
	defer wg.Done()
	for n := range jobs {
		results <- Done{n, id, steps(n)}
	}
}

func main() {
	jobs := make(chan int, 4)
	results := make(chan Done, 4)
	var wg sync.WaitGroup
	go feed(jobs, 12)
	for id := 1; id <= 4; id++ {
		wg.Add(1)
		go work(id, jobs, results, &wg)
	}
	go func() {
		wg.Wait()
		close(results)
	}()
	var all []Done
	for d := range results {
		all = append(all, d)
	}
	sort.Slice(all, func(i, j int) bool { return all[i].job < all[j].job })
	longest := all[0]
	for _, d := range all {
		fmt.Printf("job %d: %d steps\n", d.job, d.steps)
		if d.steps > longest.steps {
			longest = d
		}
	}
	fmt.Printf("longest %d\n", longest.job)
}
