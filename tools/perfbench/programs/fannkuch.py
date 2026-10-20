# Python twin of fannkuch.ail (CPython 3, one thread).
import sys


def fannkuch(n):
    perm1 = list(range(n))
    count = [0] * n
    maxflips = 0
    checksum = 0
    permcount = 0
    r = n
    done = False
    while not done:
        while r != 1:
            count[r - 1] = r
            r -= 1
        perm = []
        for i in range(n):
            perm.append(perm1[i])
        flips = 0
        k = perm[0]
        while k != 0:
            i = 0
            j = k
            while i < j:
                t = perm[i]
                perm[i] = perm[j]
                perm[j] = t
                i += 1
                j -= 1
            flips += 1
            k = perm[0]
        if flips > maxflips:
            maxflips = flips
        if permcount % 2 == 0:
            checksum += flips
        else:
            checksum -= flips
        advanced = False
        while not advanced:
            if r == n:
                done = True
                break
            first = perm1[0]
            for i in range(r):
                perm1[i] = perm1[i + 1]
            perm1[r] = first
            count[r] -= 1
            if count[r] > 0:
                advanced = True
            else:
                r += 1
        permcount += 1
    return checksum, maxflips


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 8
    checksum, maxflips = fannkuch(n)
    print(checksum)
    print(f"Pfannkuchen({n}) = {maxflips}")


main()
