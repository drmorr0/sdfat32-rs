import sys

def crc7(data):
    crc = 0
    for i in range(len(data)):
        d = int(data[i], 16)
        crc &= 0xff
        for j in range(8):
            crc <<= 1
            if ((d & 0x80) ^ (crc & 0x80)) != 0:
                crc ^= 0x09
            d <<= 1
            print(crc, d)
        print('-----')
    return ((crc << 1) | 1) & 0xff

if __name__ == '__main__':
    print(crc7(sys.argv[1:]))
