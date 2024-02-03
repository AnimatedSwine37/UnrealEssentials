import hashlib
import sys

def main():
    if len(sys.argv) < 2:
        print("Missing an argument for filename")
        return
    file = open(sys.argv[1], "rb")
    buffer = file.read()
    # get SHA-1 hash of buffer
    hasher = hashlib.sha1()
    hasher.update(buffer)
    print("SHA1 value for import file: " + hasher.hexdigest())

if __name__ == "__main__":
    main()