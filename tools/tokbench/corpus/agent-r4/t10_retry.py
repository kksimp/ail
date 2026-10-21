import time, urllib.request
def fetch(url):
    delay = 0.1
    for _ in range(3):
        try:
            return urllib.request.urlopen(url).read().decode()
        except OSError:
            time.sleep(delay)
            delay *= 2
    return urllib.request.urlopen(url).read().decode()

print(fetch("http://localhost:1/"))
