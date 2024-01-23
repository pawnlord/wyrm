# converts a dynamic number to an integer


num = input()

bs = [num[i:i+2] for i in range(0, len(num), 2)]

# convert from hex
nums = [int(b, 16) for b in bs]
final = 0
for i, num in enumerate(nums):
    s = bin((num & 0x7F) * (1 << (i * 7)))
    print((66 - len(s))*" " + s)
    final += (num & 0x7F) * (1 << (i * 7))

print(final)
