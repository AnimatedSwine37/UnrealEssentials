from cityhash import CityHash64
import sys

def check_args():
    return True if (len(sys.argv) == 2) else False
    

def main():
    if check_args():
        # ok lets go
        asset_name = sys.argv[1]
        encoded = asset_name.lower()
        encoded_utf8 = encoded.encode("utf-8")
        encoded_utf16 = encoded.encode("utf-16")[2:]
        print("little endian UTF-8: " + CityHash64(encoded_utf8).to_bytes(8, "little").hex())
        print("big endian UTF-8: " + CityHash64(encoded_utf8).to_bytes(8, "big").hex())
        print("little endian UTF-16: " + CityHash64(encoded_utf16).to_bytes(8, "little").hex())
        print("big endian UTF-16: " + CityHash64(encoded_utf16).to_bytes(8, "big").hex())
    else:
        print("ERROR: Value required to hash")

if __name__ == "__main__":
    main()