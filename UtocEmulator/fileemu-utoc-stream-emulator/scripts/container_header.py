import io
import sys
import struct

def make_tarray(file, entry):
    length = struct.unpack("<I", file.read(4))[0]
    entries = struct.unpack("<" + str(length) + str(entry), file.read(struct.calcsize(entry) * length))
    return list(entries)

class FPackageStoreEntry:
    def __init__(self, export_bundle_size, export_count, export_bundle_count, load_order, imported_package_count):
        self.export_bundle_size = export_bundle_size
        self.export_count = export_count
        self.export_bundle_count = export_bundle_count
        self.load_order = load_order
        self.imported_package_count = imported_package_count

def make_store_entry(file, i):
    info = struct.unpack("<Q6I", file.read(32))
    store_entry = FPackageStoreEntry(info[0], info[1], info[2], info[3], info[5])
    print(
        "file " + str(i) +
        ", bundle size: " + str(store_entry.export_bundle_size) + 
        ", exports: " + str(store_entry.export_count) + 
        ", export bundles: " + str(store_entry.export_bundle_count) +
        ", load order: " + str(store_entry.load_order) +
        ", imports: " + str(store_entry.imported_package_count)
    )
    return store_entry.export_bundle_count

def main():
    if len(sys.argv) < 2:
        print("Missing filename for container header")
        return
    file = open(sys.argv[1], "rb")
    # use little endian (x86 + ARM)
    header = struct.unpack("<QI", file.read(12)) # container_id, package_names
    print("container hash: " + hex(header[0]) + " package count: " + hex(header[1]))
    names = make_tarray(file, "B") # TArray<u8> Names
    if names != False:
        print("names: " + str(names))
    name_hashes = make_tarray(file, "B") # TArray<u8> NameHashes
    if name_hashes != False:
        print("name hashes: " + str(name_hashes))
    package_ids = make_tarray(file, "Q") # TArray<FPackageId> PackageIds
    if package_ids != False:
        print("package ids: " + str(package_ids))
    entry_bytes = make_tarray(file, "B") # TArray<u8> StoreEntries
    entry_buffer = io.BytesIO(bytes(entry_bytes))
    multiple_export_bundles = []
    for i in range(len(package_ids)):
        multiple_export_bundle = make_store_entry(entry_buffer, hex(package_ids[i]))
        # Exports in Blank project with more than 1 export bundle
        # MasterSubmixDefault.uasset
        # MasterReverbSubmixDefault.uasset
        # In both of those files, their LocalExportIndex is a value of 1. At least for now, assume that export bundle count is
        # max(LocalExportIndex) + 1. In most files, this is zero, so we get 1 export bundle
        if (multiple_export_bundle > 1):
            multiple_export_bundles.append("WARNING: File " + hex(package_ids[i]) + " has " + str(multiple_export_bundle) + " export bundles")
    for i in multiple_export_bundles:
        print(i)

if __name__ == "__main__":
    main()