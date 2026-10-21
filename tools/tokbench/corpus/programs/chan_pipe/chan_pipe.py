import queue
import threading


def produce(ch, n):
    for i in range(n):
        ch.put(i * i)
    ch.put(None)


def main():
    ch = queue.Queue(maxsize=4)
    worker = threading.Thread(target=produce, args=(ch, 10))
    worker.start()
    total = 0
    while (v := ch.get()) is not None:
        total += v
    worker.join()
    print(f"sum of squares 0..10: {total}")


if __name__ == "__main__":
    main()
